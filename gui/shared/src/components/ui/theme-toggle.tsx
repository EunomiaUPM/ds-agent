/**
 * theme-toggle.tsx
 *
 * Light / system / dark selector backed by ThemeContext.
 */

import React from "react";
import { Monitor, Moon, Sun } from "lucide-react";
import { cn } from "shared/src/lib/utils";
import { useTheme } from "shared/src/hooks/useTheme";
import type { ThemePreference } from "shared/src/context/ThemeContext";

const OPTIONS: { value: ThemePreference; label: string; icon: React.ComponentType<{ className?: string }> }[] = [
  { value: "light", label: "Light theme", icon: Sun },
  { value: "system", label: "Follow system theme", icon: Monitor },
  { value: "dark", label: "Dark theme", icon: Moon },
];

/** Segmented control with the three theme preferences. */
export const ThemeToggle = ({ className }: { className?: string }) => {
  const { theme, setTheme } = useTheme();

  return (
    <div
      role="radiogroup"
      aria-label="Colour theme"
      className={cn(
        "flex items-center gap-0.5 rounded-full border border-ink/10 bg-ink/5 p-0.5",
        className,
      )}
    >
      {OPTIONS.map(({ value, label, icon: Icon }) => {
        const active = theme === value;
        return (
          <button
            key={value}
            type="button"
            role="radio"
            aria-checked={active}
            aria-label={label}
            title={label}
            onClick={() => setTheme(value)}
            className={cn(
              "flex h-6 w-6 items-center justify-center rounded-full transition-colors",
              active
                ? "bg-ink/15 text-foreground-50 shadow-sm"
                : "text-muted-foreground hover:text-foreground-50 hover:bg-ink/10",
            )}
          >
            <Icon className="h-3.5 w-3.5" />
          </button>
        );
      })}
    </div>
  );
};

/** Single button that flips between light and dark, for tight spots. */
export const ThemeToggleCompact = ({ className }: { className?: string }) => {
  const { resolvedTheme, toggleTheme } = useTheme();
  const goingDark = resolvedTheme === "light";

  return (
    <button
      type="button"
      onClick={toggleTheme}
      aria-label={goingDark ? "Switch to dark theme" : "Switch to light theme"}
      title={goingDark ? "Switch to dark theme" : "Switch to light theme"}
      className={cn(
        "flex h-7 w-7 items-center justify-center rounded-full border border-ink/10",
        "bg-ink/5 text-muted-foreground transition-colors hover:bg-ink/10 hover:text-foreground-50",
        className,
      )}
    >
      {goingDark ? <Moon className="h-4 w-4" /> : <Sun className="h-4 w-4" />}
    </button>
  );
};
