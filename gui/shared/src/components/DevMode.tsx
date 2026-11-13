import React, { useContext, useEffect, useState } from "react";
import { GlobalInfoContext, GlobalInfoContextType } from "shared/src/context/GlobalInfoContext";
import { cn } from "@eunomia/ui";

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

  // Host and port only: the full gateway URL stays in the tooltip.
  const gatewayHost = api_gateway?.replace(/^https?:\/\//, "").split("/")[0];

  return (
    <div
      className={cn(
        "flex items-center gap-1.5 rounded-md border px-2 py-1 text-xs",
        "group-data-[collapsible=icon]:justify-center group-data-[collapsible=icon]:px-1",
        theme.border,
        className,
      )}
      title={`GUI dev mode\nGateway: ${api_gateway || "default"}`}
    >
      <span className={cn("h-2 w-2 shrink-0 rounded-full", theme.bg)} />
      <span className="font-medium text-foreground group-data-[collapsible=icon]:hidden">Dev</span>
      {gatewayHost && (
        <span className="truncate font-mono text-muted-foreground group-data-[collapsible=icon]:hidden">
          {gatewayHost}
        </span>
      )}
    </div>
  );
};
