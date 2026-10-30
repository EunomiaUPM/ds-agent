import React from "react";
import { useLink } from "../link";
import { cn } from "../lib/utils";

export interface StatTileProps {
  label: string;
  value?: number;
  hint?: React.ReactNode;
  icon: React.ComponentType<{ className?: string }>;
  to?: string;
}

/** One headline number with its label; a link when `to` is set. */
export function StatTile({ label, value, hint, icon: Icon, to }: StatTileProps) {
  const Link = useLink();
  const body = (
    <>
      <div className="flex items-center justify-between">
        <span className="text-sm text-ink/60">{label}</span>
        <Icon className="h-4 w-4 text-ink/40" />
      </div>
      <div className="mt-2 text-3xl font-semibold tabular-nums tracking-tight">
        {value === undefined ? <span className="text-ink/25">—</span> : value.toLocaleString()}
      </div>
      {hint && <div className="mt-1 text-xs text-ink/50">{hint}</div>}
    </>
  );
  const className = cn(
    "block rounded-xl border border-ink/10 bg-ink/[0.02] p-4 transition-colors",
    to && "hover:border-ink/20 hover:bg-ink/[0.04]",
  );
  return to ? (
    <Link to={to} className={className}>
      {body}
    </Link>
  ) : (
    <div className={className}>{body}</div>
  );
}
