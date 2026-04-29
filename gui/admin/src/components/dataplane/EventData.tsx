import React from "react";

/** Renders TransferEventDto.data as a compact 2-column grid */
export function EventData({ data }: { data: Record<string, unknown> }) {
  const entries = Object.entries(data);
  if (!entries.length) return <span className="text-muted-foreground text-xs italic">—</span>;

  return (
    <div className="grid grid-cols-2 gap-x-4 gap-y-0.5">
      {entries.map(([key, value]) => {
        const isStatus = key === "status";
        const statusNum = isStatus ? Number(value) : 0;
        const statusColor =
          statusNum >= 500
            ? "text-danger-300"
            : statusNum >= 400
              ? "text-warn-300"
              : statusNum >= 200
                ? "text-success-300"
                : "";

        const displayValue =
          typeof value === "object" && value !== null
            ? JSON.stringify(value)
            : String(value ?? "—");

        return (
          <div key={key} className="flex items-baseline gap-1 min-w-0">
            <span className="font-mono text-[10px] text-muted-foreground shrink-0">{key}</span>
            <span
              title={displayValue}
              className={`font-mono text-[10px] truncate ${isStatus ? statusColor : ""}`}
            >
              {displayValue}
            </span>
          </div>
        );
      })}
    </div>
  );
}
