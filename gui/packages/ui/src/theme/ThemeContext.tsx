import { createContext, ReactNode, useCallback, useEffect, useMemo, useState } from "react";

/** User-selectable preference; "system" follows the OS setting. */
export type ThemePreference = "light" | "dark" | "system";

/** Theme actually painted on screen once "system" is resolved. */
export type ResolvedTheme = "light" | "dark";

export interface ThemeContextType {
  theme: ThemePreference;
  resolvedTheme: ResolvedTheme;
  setTheme: (theme: ThemePreference) => void;
  /** Flips between light and dark, resolving "system" first. */
  toggleTheme: () => void;
}

export const THEME_STORAGE_KEY = "eunomia.theme";

export const ThemeContext = createContext<ThemeContextType | null>(null);

const prefersDark = () =>
  typeof window !== "undefined" && window.matchMedia("(prefers-color-scheme: dark)").matches;

function readStoredPreference(fallback: ThemePreference): ThemePreference {
  try {
    const stored = localStorage.getItem(THEME_STORAGE_KEY);
    if (stored === "light" || stored === "dark" || stored === "system") return stored;
  } catch {
    // private mode or blocked storage: fall back to defaultTheme
  }
  return fallback;
}

/** Mirrors the theme onto <html> so the CSS variables in tokens.css apply. */
function applyTheme(resolved: ResolvedTheme) {
  const root = document.documentElement;
  root.classList.toggle("dark", resolved === "dark");
  root.style.colorScheme = resolved;
}

export interface ThemeContextProviderProps {
  children: ReactNode;
  defaultTheme?: ThemePreference;
}

export const ThemeContextProvider = ({
  children,
  defaultTheme = "system",
}: ThemeContextProviderProps) => {
  const [theme, setThemeState] = useState<ThemePreference>(() =>
    readStoredPreference(defaultTheme),
  );
  const [systemTheme, setSystemTheme] = useState<ResolvedTheme>(() =>
    prefersDark() ? "dark" : "light",
  );

  useEffect(() => {
    const media = window.matchMedia("(prefers-color-scheme: dark)");
    const onChange = (e: MediaQueryListEvent) => setSystemTheme(e.matches ? "dark" : "light");
    media.addEventListener("change", onChange);
    return () => media.removeEventListener("change", onChange);
  }, []);

  const resolvedTheme: ResolvedTheme = theme === "system" ? systemTheme : theme;

  useEffect(() => {
    applyTheme(resolvedTheme);
  }, [resolvedTheme]);

  const setTheme = useCallback((next: ThemePreference) => {
    setThemeState(next);
    try {
      localStorage.setItem(THEME_STORAGE_KEY, next);
    } catch {
      // preference simply does not survive the reload
    }
  }, []);

  const toggleTheme = useCallback(() => {
    setTheme(resolvedTheme === "dark" ? "light" : "dark");
  }, [resolvedTheme, setTheme]);

  const value = useMemo(
    () => ({ theme, resolvedTheme, setTheme, toggleTheme }),
    [theme, resolvedTheme, setTheme, toggleTheme],
  );

  return <ThemeContext.Provider value={value}>{children}</ThemeContext.Provider>;
};
