import React from "react";
import { cn } from "@eunomia/ui";
import { CONTROL_POINTS } from "shared/src/data/policyTemplateApi";
import type { ControlPoint } from "shared/src/data/orval/model";

/** Where a policy piece is meant to apply, in the lifecycle order; informative only. */
export function ControlPointChips({
  points,
  className,
}: {
  points?: ControlPoint[] | null;
  className?: string;
}) {
  const shown = CONTROL_POINTS.filter((p) => points?.includes(p.value));
  if (shown.length === 0) return null;
  return (
    <span className={cn("inline-flex flex-wrap gap-1", className)}>
      {shown.map((p) => (
        <span
          key={p.value}
          title={`${p.hint}. Informative: nothing enforces policies yet.`}
          className="rounded-full border border-brand-sky/25 bg-brand-sky/10 px-2 py-px text-[11px] text-brand-sky"
        >
          {p.label}
        </span>
      ))}
    </span>
  );
}
