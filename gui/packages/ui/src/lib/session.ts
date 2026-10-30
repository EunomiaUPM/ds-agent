import { customInstance, UNAUTHORIZED_EVENT } from "./http";

export { UNAUTHORIZED_EVENT };

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
export const signOut = (redirectPath = "/admin/"): void => {
  currentUser = null;
  if (typeof window === "undefined") return;
  const back = encodeURIComponent(`${window.location.origin}${redirectPath}`);
  window.location.assign(`/oauth2/sign_out?rd=${back}`);
};
