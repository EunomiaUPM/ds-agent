import React from "react";

/** Renders a plain object as a tidy key → value list */
export function ConfigGrid({ data }: { data: Record<string, unknown> }) {
  const entries = Object.entries(data);
  if (!entries.length) return <span className="text-muted-foreground text-xs italic">—</span>;

  return (
    <div className="divide-y divide-white/5">
      {entries.map(([key, value]) => (
        <div key={key} className="flex items-start gap-3 py-1.5">
          <span className="font-mono text-xs text-muted-foreground min-w-[140px] shrink-0">
            {key}
          </span>
          <span className="font-mono text-xs break-all">
            {typeof value === "object" && value !== null
              ? JSON.stringify(value)
              : String(value ?? "—")}
          </span>
        </div>
      ))}
    </div>
  );
}
