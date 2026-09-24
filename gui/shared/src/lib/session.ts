/*
 * Copyright (C) 2026 - Universidad Politécnica de Madrid - UPM
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program. If not, see <https://www.gnu.org/licenses/>.
 */

// OAuth 2.0 and OpenID Connect session manager for Eunomia DS-Agent.
// Manages real tokens, OIDC UserInfo introspection, and token lifecycle.

export interface AdminSessionInfo {
  sub: string;
  name?: string | null;
  email?: string | null;
  preferred_username?: string | null;
  roles?: string[];
  role?: string;
  token?: string | null;
  id_token?: string | null;
  refresh_token?: string | null;
  token_type?: string;
  scope?: string | null;
  login_at?: string;
  expires_at?: number;
  client_id?: string;
  iss?: string;
  aud?: string;
}

const SESSION_KEY = "eunomia_admin_session";
const TOKEN_KEY = "eunomia_token";
const ACCESS_TOKEN_KEY = "access_token";
const REFRESH_TOKEN_KEY = "refresh_token";
const ID_TOKEN_KEY = "id_token";
const USER_INFO_KEY = "eunomia_user_info";

// Safely decode JWT claims without external dependencies.
export function decodeJwtPayload<T = Record<string, any>>(token: string): T | null {
  try {
    const parts = token.split(".");
    if (parts.length !== 3) return null;
    let base64 = parts[1].replace(/-/g, "+").replace(/_/g, "/");
    while (base64.length % 4) {
      base64 += "=";
    }
    const jsonStr = decodeURIComponent(
      atob(base64)
        .split("")
        .map((c) => "%" + ("00" + c.charCodeAt(0).toString(16)).slice(-2))
        .join("")
    );
    return JSON.parse(jsonStr) as T;
  } catch {
    return null;
  }
}

// Check if an authenticated session exists and its token has not expired.
export const isSessionActive = (): boolean => {
  if (typeof localStorage === "undefined") return false;
  const token = getSessionToken();
  if (!token) return false;

  const claims = decodeJwtPayload<{ exp?: number }>(token);
  if (claims && claims.exp) {
    const now = Math.floor(Date.now() / 1000);
    if (claims.exp <= now) {
      return false;
    }
  }

  return true;
};

// Retrieve active bearer access token from localStorage.
export const getSessionToken = (): string | null => {
  if (typeof localStorage === "undefined") return null;
  return (
    localStorage.getItem(TOKEN_KEY) ||
    localStorage.getItem(ACCESS_TOKEN_KEY) ||
    localStorage.getItem("pat_token")
  );
};

// Retrieve OpenID Connect ID token from localStorage.
export const getIdToken = (): string | null => {
  if (typeof localStorage === "undefined") return null;
  return localStorage.getItem(ID_TOKEN_KEY);
};

// Retrieve OAuth 2.0 refresh token from localStorage.
export const getRefreshToken = (): string | null => {
  if (typeof localStorage === "undefined") return null;
  return localStorage.getItem(REFRESH_TOKEN_KEY);
};

// Retrieve stored admin profile and OIDC user info.
export const getAdminInfo = (): AdminSessionInfo | null => {
  if (typeof localStorage === "undefined") return null;
  try {
    const raw = localStorage.getItem(USER_INFO_KEY);
    if (raw) return JSON.parse(raw);
  } catch (e) {
    console.error("Error reading admin session info:", e);
  }
  return null;
};

// Persist session tokens and admin profile to localStorage.
export const setSession = (
  token?: string | null,
  adminInfo?: AdminSessionInfo | null,
  refreshToken?: string | null,
  idToken?: string | null
): void => {
  if (typeof localStorage === "undefined") return;
  localStorage.setItem(SESSION_KEY, "true");
  if (token) {
    localStorage.setItem(TOKEN_KEY, token);
    localStorage.setItem(ACCESS_TOKEN_KEY, token);
  }
  if (refreshToken) {
    localStorage.setItem(REFRESH_TOKEN_KEY, refreshToken);
  }
  if (idToken) {
    localStorage.setItem(ID_TOKEN_KEY, idToken);
  }
  if (adminInfo) {
    localStorage.setItem(USER_INFO_KEY, JSON.stringify(adminInfo));
  }
};

// Clear all authentication keys and tokens from localStorage.
export const clearSession = (): void => {
  if (typeof localStorage === "undefined") return;
  localStorage.removeItem(SESSION_KEY);
  localStorage.removeItem(TOKEN_KEY);
  localStorage.removeItem(ACCESS_TOKEN_KEY);
  localStorage.removeItem(REFRESH_TOKEN_KEY);
  localStorage.removeItem(ID_TOKEN_KEY);
  localStorage.removeItem(USER_INFO_KEY);
  localStorage.removeItem("auth_data");
  localStorage.removeItem("eunomia_acting_tenant");
};

// Helper to try OAuth endpoint across common mounting prefixes (/admin/api, /api, /oauth).
async function fetchOAuthEndpoint(
  pathSuffix: string,
  options: RequestInit,
  apiBase: string = ""
): Promise<Response> {
  const baseUrl = apiBase ? apiBase.replace(/\/$/, "") : "";
  const candidates = [
    `${baseUrl}/admin/api/oauth/${pathSuffix}`,
    `${baseUrl}/api/oauth/${pathSuffix}`,
    `${baseUrl}/oauth/${pathSuffix}`,
  ];

  let lastRes: Response | null = null;
  for (const url of candidates) {
    try {
      const res = await fetch(url, options);
      if (res.status !== 404) {
        return res;
      }
      lastRes = res;
    } catch {
      // Continue trying remaining candidate endpoints
    }
  }
  if (lastRes) return lastRes;
  throw new Error(`Unable to reach /${pathSuffix} on any candidate URL.`);
}

// Refresh OAuth access token using stored refresh token.
export async function refreshOAuthToken(apiBase: string = ""): Promise<string | null> {
  const refreshToken = getRefreshToken();
  if (!refreshToken) return null;

  try {
    const res = await fetchOAuthEndpoint(
      "refresh",
      {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ refresh_token: refreshToken }),
      },
      apiBase
    );

    if (!res.ok) {
      clearSession();
      return null;
    }

    const data = await res.json();
    if (data.access_token) {
      const currentAdmin = getAdminInfo();
      const claims = decodeJwtPayload(data.access_token) || {};
      const updatedAdmin: AdminSessionInfo | null = currentAdmin
        ? {
            ...currentAdmin,
            token: data.access_token,
            expires_at: claims.exp || currentAdmin.expires_at,
          }
        : null;

      setSession(data.access_token, updatedAdmin, data.refresh_token || refreshToken, data.id_token);
      return data.access_token;
    }
  } catch (e) {
    console.warn("Error refreshing OAuth token:", e);
  }
  return null;
}

// Authenticate against OAuth 2.0 Token endpoint and fetch OIDC UserInfo.
export async function loginWithOAuth(
  username: string,
  password: string,
  apiBase: string = ""
): Promise<{ success: boolean; error?: string; adminInfo?: AdminSessionInfo }> {
  const cleanUsername = username.trim();
  if (!cleanUsername || !password) {
    return { success: false, error: "Username and password are required." };
  }

  try {
    const res = await fetchOAuthEndpoint(
      "token",
      {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          grant_type: "password",
          client_id: "eunomia-admin-gui",
          username: cleanUsername,
          password,
          scope: "openid profile email admin",
        }),
      },
      apiBase
    );

    if (!res.ok) {
      let errMsg = "Authentication failed. Invalid username or password.";
      try {
        const errJson = await res.json();
        if (errJson.error_description) {
          const desc = String(errJson.error_description).trim();
          errMsg = desc === "Format Error" ? "Invalid username or password." : desc;
        } else if (errJson.error) {
          errMsg = errJson.error;
        }
      } catch {
        // Fallback default message
      }
      return { success: false, error: errMsg };
    }

    const tokenData = await res.json();
    const accessToken = tokenData.access_token;
    const idToken = tokenData.id_token;
    const refreshToken = tokenData.refresh_token;

    // Decode ID token or access token claims for identity
    const idClaims = idToken ? decodeJwtPayload(idToken) : null;
    const accessClaims = decodeJwtPayload(accessToken);
    const claims = idClaims || accessClaims || {};

    const userSub = claims.sub || cleanUsername;
    const userEmail =
      claims.email ||
      (cleanUsername.includes("@") ? cleanUsername : `${cleanUsername}@admin.local`);
    const userRole = claims.role || "Admin";

    let adminInfo: AdminSessionInfo = {
      sub: userSub,
      name: claims.name || cleanUsername,
      email: userEmail,
      preferred_username: cleanUsername,
      roles: [userRole],
      role: userRole,
      token: accessToken,
      id_token: idToken,
      refresh_token: refreshToken,
      token_type: tokenData.token_type || "Bearer",
      scope: tokenData.scope || "openid profile email admin",
      login_at: new Date().toISOString(),
      expires_at:
        claims.exp || Math.floor(Date.now() / 1000) + (tokenData.expires_in || 3600),
      client_id: "eunomia-admin-gui",
      iss: claims.iss,
      aud: claims.aud,
    };

    // Fetch OIDC UserInfo endpoint to enrich profile
    try {
      const uRes = await fetchOAuthEndpoint(
        "userinfo",
        {
          headers: { Authorization: `Bearer ${accessToken}` },
        },
        apiBase
      );
      if (uRes.ok) {
        const uData = await uRes.json();
        adminInfo = {
          ...adminInfo,
          sub: uData.sub || adminInfo.sub,
          name: uData.name || adminInfo.name,
          email: uData.email || adminInfo.email,
          preferred_username: uData.preferred_username || adminInfo.preferred_username,
          roles: uData.roles || (uData.role ? [uData.role] : adminInfo.roles),
          role: uData.role || (uData.roles ? uData.roles[0] : adminInfo.role),
        };
      }
    } catch (e) {
      console.warn("Could not fetch OIDC userinfo, using token claims:", e);
    }

    setSession(accessToken, adminInfo, refreshToken, idToken);
    return { success: true, adminInfo };
  } catch (err: any) {
    return {
      success: false,
      error: `Network error: unable to connect to OAuth service at ${
        apiBase || (typeof window !== "undefined" ? window.location.origin : "")
      }/admin/api/oauth/token. Please verify the backend is running.`,
    };
  }
}

// Revoke current OAuth token on the server and clear local session.
export async function logoutOAuth(apiBase: string = ""): Promise<void> {
  const token = getSessionToken();
  if (token) {
    try {
      await fetchOAuthEndpoint(
        "revoke",
        {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ token }),
        },
        apiBase
      );
    } catch (e) {
      console.warn("Error revoking OAuth token on server:", e);
    }
  }
  clearSession();
}
