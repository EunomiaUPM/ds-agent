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

// The browser session, as the gateway's identity provider sees it.
//
// The console never handles tokens. With Keycloak, oauth2-proxy keeps the session in a cookie
// and adds the token on the way to the agents; in `static` mode every request is the configured
// user. The console asks the gateway who it is (`GET /me`); a 401 means there is no session, and
// the browser goes to the proxy's sign-in. The agents enforce access whatever the console shows.
// The former password login against the built-in OAuth module lives in oauth-session.legacy.ts.

import { customInstance } from "../data/orval-mutator";

/** The session's user, as `GET /me` returns it. */
export interface SessionUser {
  userId: string;
  username?: string | null;
  email?: string | null;
  /** Role path, e.g. `/admin/upm`. */
  role: string;
  /** Whether it is the root (`/admin`), who sees and acts on everything. */
  root: boolean;
}

/** Event fired when a call finds no session (401). */
export const UNAUTHORIZED_EVENT = "eunomia:unauthorized";

let currentUser: SessionUser | null = null;

/** The user of the session, once `loadSession` found one. */
export const getSessionUser = (): SessionUser | null => currentUser;

/** A name to show for the session's user. */
export const sessionDisplayName = (user: SessionUser | null): string =>
  user?.username || user?.email || user?.userId || "";

/** Asks the gateway who the session is; `null` when there is none (401). */
export const loadSession = async (): Promise<SessionUser | null> => {
  try {
    const res = await customInstance<{ status: number; data: SessionUser }>("/me", {
      method: "GET",
      _noSessionRedirect: true,
    });
    currentUser = res.status === 200 ? res.data : null;
  } catch {
    currentUser = null;
  }
  return currentUser;
};

/** Sends the browser to the proxy's sign-in, coming back to where it was. */
export const signIn = (): void => {
  if (typeof window === "undefined") return;
  const back = encodeURIComponent(window.location.href);
  window.location.assign(`/oauth2/start?rd=${back}`);
};

/** Ends the proxy's session and comes back to the console. */
export const signOut = (): void => {
  currentUser = null;
  if (typeof window === "undefined") return;
  const back = encodeURIComponent(`${window.location.origin}/admin/`);
  window.location.assign(`/oauth2/sign_out?rd=${back}`);
};
