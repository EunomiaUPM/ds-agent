import React from "react";
import { cn } from "../lib/utils";

/** A small count next to a tab or title; renders nothing for zero unless `showZero`. */
export function CountBadge({
  n,
  showZero = false,
  className,
}: {
  n?: number;
  showZero?: boolean;
  className?: string;
}) {
  if (n === undefined || (n === 0 && !showZero)) return null;
  return (
    <span
      className={cn("rounded-full bg-ink/10 px-1.5 text-xs font-medium tabular-nums", className)}
    >
      {n}
    </span>
  );
}
