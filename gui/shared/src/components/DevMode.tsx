import React, { useContext, useEffect, useState } from "react";
import { GlobalInfoContext, GlobalInfoContextType } from "shared/src/context/GlobalInfoContext";
import { cn } from "shared/src/lib/utils";

const randomThemes = [
  { border: "border-red-500/50", bg: "bg-red-500", text: "text-red-500" },
  { border: "border-orange-500/50", bg: "bg-orange-500", text: "text-orange-500" },
  { border: "border-amber-500/50", bg: "bg-amber-500", text: "text-amber-500" },
  { border: "border-yellow-400/50", bg: "bg-yellow-400", text: "text-yellow-400" },
  { border: "border-lime-400/50", bg: "bg-lime-400", text: "text-lime-400" },
  { border: "border-green-500/50", bg: "bg-green-500", text: "text-green-500" },
  { border: "border-emerald-500/50", bg: "bg-emerald-500", text: "text-emerald-500" },
  { border: "border-teal-400/50", bg: "bg-teal-400", text: "text-teal-400" },
  { border: "border-cyan-400/50", bg: "bg-cyan-400", text: "text-cyan-400" },
  { border: "border-sky-400/50", bg: "bg-sky-400", text: "text-sky-400" },
  { border: "border-blue-400/50", bg: "bg-blue-400", text: "text-blue-400" },
  { border: "border-indigo-400/50", bg: "bg-indigo-400", text: "text-indigo-400" },
  { border: "border-violet-400/50", bg: "bg-violet-400", text: "text-violet-400" },
  { border: "border-purple-400/50", bg: "bg-purple-400", text: "text-purple-400" },
  { border: "border-fuchsia-400/50", bg: "bg-fuchsia-400", text: "text-fuchsia-400" },
  { border: "border-pink-400/50", bg: "bg-pink-400", text: "text-pink-400" },
  { border: "border-rose-400/50", bg: "bg-rose-400", text: "text-rose-400" },
];

const STORAGE_KEY = "dev_mode_gateway_colors";

type ThemeType = { border: string; bg: string; text?: string };

export const DevMode = ({ className }: { className?: string }) => {
  const { api_gateway } = useContext<GlobalInfoContextType | null>(GlobalInfoContext) || {};
  const [theme, setTheme] = useState<ThemeType | null>(null);

  useEffect(() => {
    if (!api_gateway) return;

    try {
      const storedData = localStorage.getItem(STORAGE_KEY);
      const colorMap: Record<string, ThemeType> = storedData ? JSON.parse(storedData) : {};
      if (colorMap[api_gateway]) {
        setTheme(colorMap[api_gateway]);
      } else {
        const randomIndex = Math.floor(Math.random() * randomThemes.length);
        const newTheme = randomThemes[randomIndex];
        colorMap[api_gateway] = newTheme;
        localStorage.setItem(STORAGE_KEY, JSON.stringify(colorMap));

        setTheme(newTheme);
      }
    } catch (error) {
      console.warn("Not able to load localstorage", error);
      setTheme(randomThemes[2]); // default
    }
  }, [api_gateway]);

  if (!theme || import.meta.env.PROD) return null;

  return (
    <div
      className={cn(
        "rounded-xl border p-2.5 bg-ink/[0.03] dark:bg-ink/[0.06] shadow-2xs space-y-1 transition-all text-xs font-mono",
        "group-data-[collapsible=icon]:p-1.5 group-data-[collapsible=icon]:flex group-data-[collapsible=icon]:justify-center",
        theme.border,
        className,
      )}
      title={`GUI Dev Mode ON\nGateway: ${api_gateway || "default"}`}
    >
      <div className="flex items-center justify-between gap-1.5">
        <div className="flex items-center gap-1.5">
          <span className={cn("h-2 w-2 rounded-full shrink-0 animate-pulse", theme.bg)} />
          <span className="font-semibold text-foreground tracking-tight text-[11px] group-data-[collapsible=icon]:hidden">
            GUI Dev Mode
          </span>
        </div>
        <span
          className={cn(
            "text-[9px] uppercase px-1 py-0.2 rounded font-bold group-data-[collapsible=icon]:hidden",
            theme.bg,
            "text-black font-sans",
          )}
        >
          ON
        </span>
      </div>

      {api_gateway && (
        <div className="text-[10px] text-muted-foreground truncate group-data-[collapsible=icon]:hidden">
          <span className="opacity-70 font-sans">Gateway: </span>
          <span className="font-mono select-all text-foreground/80">{api_gateway}</span>
        </div>
      )}
    </div>
  );
};
