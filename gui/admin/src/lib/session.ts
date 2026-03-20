const SESSION_KEY = "eunomia_admin_session";

export const isSessionActive = (): boolean => {
  return localStorage.getItem(SESSION_KEY) === "true";
};

export const setSession = (): void => {
  localStorage.setItem(SESSION_KEY, "true");
};

export const clearSession = (): void => {
  localStorage.removeItem(SESSION_KEY);
};
