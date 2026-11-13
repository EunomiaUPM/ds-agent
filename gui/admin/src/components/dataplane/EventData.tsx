import React from "react";
import { cn } from "@eunomia/ui";
import { formatEventFieldValue, parseTransferEventData } from "./utils/dataplaneState";

function statusTone(status: number): string {
  if (status === 0 || status >= 500) {
    return "border-danger-500/35 bg-danger-500/15 text-danger-700 dark:text-danger-300";
  }
  if (status >= 400) {
    return "border-warn-500/35 bg-warn-500/15 text-warn-700 dark:text-warn-300";
  }
  if (status >= 200 && status < 300) {
    return "border-success-500/35 bg-success-500/15 text-success-700 dark:text-success-300";
  }
  return "border-ink/15 bg-ink/[0.06] text-ink/80";
}

/** Renders `TransferEventDto.data` as scannable HTTP/Kafka telemetry badges and key-value pills. */
export function EventData({ data }: { data: Record<string, unknown> }) {
  const parsed = parseTransferEventData(data);
  const hasAny =
    parsed.method ||
    parsed.status !== undefined ||
    parsed.reason ||
    parsed.target ||
    parsed.originPath ||
    parsed.kafka ||
    parsed.ingress ||
    parsed.egress ||
    parsed.rest.length > 0;

  if (!hasAny) {
    return <span className="text-xs italic text-ink/40">—</span>;
  }

  return (
    <div className="flex flex-wrap items-center gap-1.5 text-xs">
      {(parsed.method || parsed.status !== undefined) && (
        <span className="inline-flex items-center gap-1">
          {parsed.method && (
            <span className="rounded border border-ink/15 bg-ink/[0.05] px-1.5 py-0.5 font-mono font-semibold text-ink/80">
              {parsed.method}
            </span>
          )}
          {parsed.status !== undefined && (
            <span
              className={cn(
                "rounded border px-1.5 py-0.5 font-mono font-semibold tabular-nums",
                statusTone(parsed.status),
              )}
            >
              {parsed.status === 0 ? "CONN_ERR" : `HTTP ${parsed.status}`}
            </span>
          )}
        </span>
      )}

      {parsed.reason && (
        <span className="rounded border border-warn-500/30 bg-warn-500/10 px-2 py-0.5 font-mono text-warn-800 dark:text-warn-200">
          {parsed.reason}
        </span>
      )}

      {parsed.target && (
        <span
          title={parsed.target}
          className="inline-flex max-w-[360px] items-center gap-1 rounded border border-ink/10 bg-ink/[0.03] px-2 py-0.5 font-mono"
        >
          <span className="shrink-0 text-ink/45">target</span>
          <span className="truncate text-ink/85">{parsed.target}</span>
        </span>
      )}

      {parsed.originPath && (
        <span
          title={parsed.originPath}
          className="inline-flex max-w-[220px] items-center gap-1 rounded border border-ink/10 bg-ink/[0.03] px-2 py-0.5 font-mono"
        >
          <span className="shrink-0 text-ink/45">path</span>
          <span className="truncate text-ink/85">{parsed.originPath}</span>
        </span>
      )}

      {parsed.kafka && (
        <span className="inline-flex items-center gap-1.5 rounded border border-ink/10 bg-ink/[0.03] px-2 py-0.5 font-mono text-ink/80">
          {parsed.kafka.topic && <span>topic:{parsed.kafka.topic}</span>}
          {parsed.kafka.partition !== undefined && (
            <span className="text-ink/55">p{String(parsed.kafka.partition)}</span>
          )}
          {parsed.kafka.offset !== undefined && (
            <span className="text-ink/55">@{String(parsed.kafka.offset)}</span>
          )}
        </span>
      )}

      {(parsed.ingress || parsed.egress) && (
        <span className="inline-flex items-center gap-1 rounded border border-ink/10 bg-ink/[0.03] px-2 py-0.5 font-mono text-ink/65">
          {parsed.ingress ?? "?"} → {parsed.egress ?? "?"}
        </span>
      )}

      {parsed.rest.map(([key, value]) => {
        const display = formatEventFieldValue(value);
        return (
          <span
            key={key}
            title={`${key}: ${display}`}
            className="inline-flex max-w-[220px] items-center gap-1 rounded border border-ink/10 bg-ink/[0.03] px-2 py-0.5 font-mono"
          >
            <span className="shrink-0 text-ink/45">{key}</span>
            <span className="truncate text-ink/80">{display}</span>
          </span>
        );
      })}
    </div>
  );
}
