import { useContext } from "react";
import { ThemeContext, ThemeContextType } from "shared/src/context/ThemeContext";

/** Reads the theme context; throws when used outside ThemeContextProvider. */
export function useTheme(): ThemeContextType {
  const ctx = useContext(ThemeContext);
  if (!ctx) {
    throw new Error("useTheme must be used within a ThemeContextProvider");
  }
  return ctx;
}
