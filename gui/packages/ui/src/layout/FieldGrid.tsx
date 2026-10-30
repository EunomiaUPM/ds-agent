import React from "react";
import { cn } from "../lib/utils";

export interface Field {
  label: string;
  value: React.ReactNode;
  /** Columns the field spans on wide screens. */
  span?: 1 | 2 | 3 | 4;
  hidden?: boolean;
}

const COLS: Record<number, string> = {
  1: "",
  2: "md:grid-cols-2",
  3: "md:grid-cols-3",
  4: "md:grid-cols-2 xl:grid-cols-4",
};
const SPAN = { 1: "", 2: "md:col-span-2", 3: "md:col-span-3", 4: "md:col-span-full" };

/** Label/value pairs in a bordered grid: the summary block at the top of detail pages. */
export function FieldGrid({
  fields,
  columns = 3,
  className,
}: {
  fields: Field[];
  columns?: 2 | 3 | 4;
  className?: string;
}) {
  const visible = fields.filter((f) => !f.hidden);
  return (
    // Each cell draws its right and bottom border; the frame clips the outer ones.
    <div className={cn("overflow-hidden rounded-lg border border-ink/10", className)}>
      <dl className={cn("-mb-px -mr-px grid grid-cols-1", COLS[Math.min(columns, visible.length)])}>
        {visible.map((f) => (
          <div
            key={f.label}
            className={cn("min-w-0 border-b border-r border-ink/10 px-4 py-3", SPAN[f.span ?? 1])}
          >
            <dt className="mb-1 text-xs font-medium text-ink/50">{f.label}</dt>
            <dd className="min-w-0 text-sm text-ink/90">
              {f.value ?? <span className="text-ink/35">—</span>}
            </dd>
          </div>
        ))}
      </dl>
    </div>
  );
}
