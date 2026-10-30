import React from "react";
import { useActivePath, useLink } from "../link";
import { cn } from "../lib/utils";
import { CountBadge } from "../ui/count-badge";

/** Classes shared by route tabs and the Radix tabs in `ui/tabs.tsx`, so both look the same. */
export const tabListClass = "flex w-full items-end gap-1 border-b border-ink/10 overflow-x-auto";
export const tabTriggerClass = cn(
  "relative -mb-px inline-flex items-center gap-2 whitespace-nowrap rounded-t-md px-4 py-2.5",
  "text-sm font-medium text-ink/55 border-b-2 border-transparent transition-colors",
  "hover:text-ink/90 hover:bg-ink/[0.03]",
  "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-brand-sky/50",
);
export const tabActiveClass = "text-ink font-semibold border-brand-sky bg-brand-sky/[0.08]";

export interface RouteTab {
  label: string;
  to: string;
  icon?: React.ComponentType<{ className?: string }>;
  count?: number;
}

/** Tabs that switch between child routes; the active one follows the current path. */
export function RouteTabs({
  tabs,
  activePath,
  className,
}: {
  tabs: RouteTab[];
  activePath?: string;
  className?: string;
}) {
  const Link = useLink();
  const ctxPath = useActivePath();
  const pathname = (activePath ?? ctxPath).replace(/\/$/, "");

  return (
    <nav className={cn(tabListClass, "mb-6", className)} role="tablist">
      {tabs.map(({ label, to, icon: Icon, count }) => {
        const cleanTo = to.replace(/\/$/, "");
        const active = pathname.endsWith(cleanTo) || pathname.includes(`${cleanTo}/`);
        return (
          <Link
            key={to}
            to={to}
            role="tab"
            aria-selected={active}
            className={cn(tabTriggerClass, active && tabActiveClass)}
          >
            {Icon && <Icon className="h-4 w-4" />}
            {label}
            <CountBadge n={count} showZero />
          </Link>
        );
      })}
    </nav>
  );
}
