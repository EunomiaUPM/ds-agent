import React, { useMemo, useState } from "react";
import dayjs from "dayjs";
import { ArrowRight, RefreshCw, Search, XCircle } from "lucide-react";
import {
  Badge,
  Button,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  FieldGrid,
  Identifier,
  Input,
  RelativeTime,
  SegmentedControl,
  Skeleton,
  cn,
} from "@eunomia/ui";
import { DataplaneTransferLogDto } from "shared/src/data/orval/model";
import {
  dataplaneStateVariant,
  formatDurationMs,
  isFailureLog,
  parseReason,
} from "./utils/dataplaneState";
import { CopyButton } from "./CopyButton";

interface DataplaneLogsTabProps {
  logs: DataplaneTransferLogDto[];
  isLoading?: boolean;
  isRefreshing?: boolean;
  onRefresh?: () => void;
}

interface EnrichedLog {
  log: DataplaneTransferLogDto;
  step: number;
  failed: boolean;
  parsedReason: ReturnType<typeof parseReason>;
  deltaMs: number | null;
  elapsedMs: number;
}

const ORDER_OPTIONS: Array<{ value: "asc" | "desc"; label: string }> = [
  { value: "asc", label: "Oldest first" },
  { value: "desc", label: "Newest first" },
];

/** State transitions recorded by the dataplane, with step deltas, failure flags, and full details. */
export function DataplaneLogsTab({
  logs,
  isLoading,
  isRefreshing,
  onRefresh,
}: DataplaneLogsTabProps) {
  const [query, setQuery] = useState("");
  const [onlyFailures, setOnlyFailures] = useState(false);
  const [order, setOrder] = useState<"asc" | "desc">("asc");
  const [selected, setSelected] = useState<EnrichedLog | null>(null);

  const enriched = useMemo<EnrichedLog[]>(() => {
    const sorted = [...logs].sort(
      (a, b) => new Date(a.created_at).getTime() - new Date(b.created_at).getTime(),
    );
    const firstMs = sorted.length > 0 ? new Date(sorted[0].created_at).getTime() : 0;
    return sorted.map((log, i) => {
      const currentMs = new Date(log.created_at).getTime();
      const prevMs = i > 0 ? new Date(sorted[i - 1].created_at).getTime() : null;
      return {
        log,
        step: i + 1,
        failed: isFailureLog(log),
        parsedReason: parseReason(log.reason),
        deltaMs: prevMs !== null ? Math.max(0, currentMs - prevMs) : null,
        elapsedMs: Math.max(0, currentMs - firstMs),
      };
    });
  }, [logs]);

  const failureCount = useMemo(() => enriched.filter((e) => e.failed).length, [enriched]);
  const latestFailure = useMemo(() => [...enriched].reverse().find((e) => e.failed), [enriched]);
  const totalElapsedMs = enriched.length > 0 ? enriched[enriched.length - 1].elapsedMs : 0;

  const visible = useMemo(() => {
    const needle = query.trim().toLowerCase();
    const filtered = enriched.filter((item) => {
      if (onlyFailures && !item.failed) return false;
      if (!needle) return true;
      return (
        item.log.new_state.toLowerCase().includes(needle) ||
        (item.log.previous_state ?? "").toLowerCase().includes(needle) ||
        item.log.trigger.toLowerCase().includes(needle) ||
        (item.log.reason ?? "").toLowerCase().includes(needle)
      );
    });
    return order === "desc" ? [...filtered].reverse() : filtered;
  }, [enriched, onlyFailures, query, order]);

  if (isLoading && logs.length === 0) {
    return (
      <div className="space-y-3">
        <Skeleton className="h-10 w-full rounded-lg" />
        <Skeleton className="h-44 w-full rounded-lg" />
      </div>
    );
  }

  if (logs.length === 0) {
    return (
      <div className="rounded-lg border border-dashed border-ink/15 px-6 py-12 text-center">
        <p className="text-sm font-medium text-ink/70">No dataplane transition logs yet</p>
        <p className="mt-1 text-xs text-ink/50">
          State transitions (Init, Configuring, Auth, Ready, Started, Terminated) appear here with
          reasons and durations.
        </p>
        {onRefresh && (
          <Button
            type="button"
            variant="outline"
            size="xs"
            onClick={onRefresh}
            disabled={isRefreshing}
            className="mt-4"
          >
            <RefreshCw className={cn("h-3.5 w-3.5", isRefreshing && "animate-spin")} />
            Refresh
          </Button>
        )}
      </div>
    );
  }

  return (
    <div className="space-y-4">
      {latestFailure && (
        <div className="flex items-start gap-3 rounded-lg border border-danger-500/30 bg-danger-500/10 p-3.5">
          <XCircle className="mt-0.5 h-4 w-4 shrink-0 text-danger-600 dark:text-danger-400" />
          <div className="min-w-0 flex-1 space-y-1">
            <div className="flex flex-wrap items-center gap-2 text-xs">
              <span className="font-semibold text-danger-800 dark:text-danger-200">
                Failure recorded at step #{latestFailure.step} ({latestFailure.log.new_state})
              </span>
              {latestFailure.parsedReason.code && (
                <Badge variant="code">{latestFailure.parsedReason.code}</Badge>
              )}
              <span className="font-mono text-ink/60">{latestFailure.log.trigger}</span>
            </div>
            {latestFailure.parsedReason.message && (
              <p className="break-words text-sm text-ink/90">
                {latestFailure.parsedReason.message}
              </p>
            )}
          </div>
        </div>
      )}

      <div className="flex flex-wrap items-center justify-between gap-3">
        <div className="flex flex-wrap items-center gap-2 text-xs text-ink/60">
          <span className="rounded-md border border-ink/10 bg-ink/[0.03] px-2.5 py-1 font-medium tabular-nums text-ink/80">
            {logs.length} {logs.length === 1 ? "transition" : "transitions"}
          </span>
          {totalElapsedMs > 0 && (
            <span className="rounded-md border border-ink/10 bg-ink/[0.03] px-2.5 py-1 font-mono tabular-nums text-ink/70">
              Total: {formatDurationMs(totalElapsedMs)}
            </span>
          )}
          {failureCount > 0 && (
            <button
              type="button"
              onClick={() => setOnlyFailures((v) => !v)}
              className={cn(
                "inline-flex items-center gap-1 rounded-md border px-2.5 py-1 font-medium transition-colors",
                onlyFailures
                  ? "border-danger-500/50 bg-danger-500/15 text-danger-700 dark:text-danger-300"
                  : "border-danger-500/25 bg-danger-500/5 text-danger-700 hover:bg-danger-500/10 dark:text-danger-300",
              )}
            >
              {failureCount} {failureCount === 1 ? "failure" : "failures"}
            </button>
          )}
        </div>

        <div className="flex flex-wrap items-center gap-2">
          <Input
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            onClear={query ? () => setQuery("") : undefined}
            placeholder="Filter state, trigger or reason..."
            startIcon={<Search className="h-3.5 w-3.5" />}
            containerClassName="w-56 sm:w-64"
            className="h-8 text-xs"
          />
          <SegmentedControl
            value={order}
            onChange={setOrder}
            options={ORDER_OPTIONS}
            ariaLabel="Log order"
          />
          {onRefresh && (
            <Button
              type="button"
              variant="outline"
              size="xs"
              onClick={onRefresh}
              disabled={isRefreshing}
            >
              <RefreshCw className={cn("h-3.5 w-3.5", isRefreshing && "animate-spin")} />
              Refresh
            </Button>
          )}
        </div>
      </div>

      {visible.length === 0 ? (
        <div className="rounded-lg border border-dashed border-ink/15 py-10 text-center text-sm text-ink/50">
          No transitions match the active filters.
        </div>
      ) : (
        <div className="overflow-x-auto rounded-lg border border-ink/10">
          <table className="w-full text-sm">
            <thead className="bg-ink/[0.03] text-left text-xs font-medium text-ink/55">
              <tr>
                <th className="w-11 px-3 py-2.5 font-medium">#</th>
                <th className="whitespace-nowrap px-3 py-2.5 font-medium">State transition</th>
                <th className="whitespace-nowrap px-3 py-2.5 font-medium">Trigger</th>
                <th className="px-3 py-2.5 font-medium">Reason</th>
                <th className="whitespace-nowrap px-3 py-2.5 text-right font-medium">Step Δ</th>
                <th className="whitespace-nowrap px-3 py-2.5 text-right font-medium">When</th>
              </tr>
            </thead>
            <tbody>
              {visible.map((item) => {
                const { log, step, failed, parsedReason, deltaMs } = item;
                return (
                  <tr
                    key={log.id ?? step}
                    onClick={() => setSelected(item)}
                    className={cn(
                      "cursor-pointer border-t border-ink/[0.07] align-top transition-colors hover:bg-ink/[0.03]",
                      failed && "bg-danger-500/[0.04] hover:bg-danger-500/[0.08]",
                    )}
                  >
                    <td className="px-3 py-2.5 font-mono text-xs tabular-nums text-ink/45">
                      {step}
                    </td>
                    <td className="whitespace-nowrap px-3 py-2.5">
                      <span className="inline-flex items-center gap-1.5">
                        {log.previous_state ? (
                          <span className="text-xs text-ink/70">{log.previous_state}</span>
                        ) : (
                          <span className="text-xs italic text-ink/45">Initial</span>
                        )}
                        <ArrowRight className="h-3 w-3 text-ink/35" />
                        <Badge
                          variant="status"
                          state={dataplaneStateVariant(log.new_state)}
                          size="sm"
                        >
                          {log.new_state}
                        </Badge>
                      </span>
                    </td>
                    <td className="whitespace-nowrap px-3 py-2.5">
                      <span className="rounded border border-ink/10 bg-ink/[0.04] px-1.5 py-0.5 font-mono text-xs text-ink/80">
                        {log.trigger}
                      </span>
                    </td>
                    <td className="min-w-[260px] max-w-[560px] px-3 py-2.5">
                      {log.reason ? (
                        <div className="flex flex-wrap items-baseline gap-1.5">
                          {parsedReason.code && (
                            <Badge variant="code" size="xs">
                              {parsedReason.code}
                            </Badge>
                          )}
                          <span
                            className={cn(
                              "break-words text-xs leading-relaxed",
                              failed
                                ? "font-medium text-danger-800 dark:text-danger-200"
                                : "text-ink/80",
                            )}
                          >
                            {parsedReason.message}
                          </span>
                        </div>
                      ) : (
                        <span className="text-xs text-ink/30">—</span>
                      )}
                    </td>
                    <td className="whitespace-nowrap px-3 py-2.5 text-right font-mono text-xs tabular-nums text-ink/50">
                      {deltaMs !== null ? `+${formatDurationMs(deltaMs)}` : "—"}
                    </td>
                    <td
                      className="whitespace-nowrap px-3 py-2.5 text-right text-xs text-ink/65"
                      title={dayjs(log.created_at).format("YYYY-MM-DD HH:mm:ss.SSS")}
                    >
                      <div className="font-mono tabular-nums text-ink/80">
                        {dayjs(log.created_at).format("HH:mm:ss.SSS")}
                      </div>
                      <div className="text-[0.6875rem] text-ink/45">
                        <RelativeTime date={log.created_at} />
                      </div>
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      )}

      <Dialog open={!!selected} onOpenChange={(open) => !open && setSelected(null)}>
        <DialogContent className="max-h-[90vh] max-w-2xl overflow-y-auto">
          {selected && (
            <>
              <DialogHeader>
                <DialogTitle className="flex flex-wrap items-center gap-2">
                  <span>Transition #{selected.step}</span>
                  <Badge
                    variant="status"
                    state={dataplaneStateVariant(selected.log.new_state)}
                    size="sm"
                  >
                    {selected.log.new_state}
                  </Badge>
                  {selected.parsedReason.code && (
                    <Badge variant="code">{selected.parsedReason.code}</Badge>
                  )}
                </DialogTitle>
                <DialogDescription className="font-mono text-xs">
                  {selected.log.trigger}
                </DialogDescription>
              </DialogHeader>

              <FieldGrid
                columns={2}
                fields={[
                  {
                    label: "State transition",
                    value: (
                      <span className="inline-flex items-center gap-1.5">
                        <span className="text-xs text-ink/70">
                          {selected.log.previous_state ?? "Initial"}
                        </span>
                        <ArrowRight className="h-3 w-3 text-ink/35" />
                        <Badge
                          variant="status"
                          state={dataplaneStateVariant(selected.log.new_state)}
                          size="sm"
                        >
                          {selected.log.new_state}
                        </Badge>
                      </span>
                    ),
                  },
                  {
                    label: "Trigger",
                    value: <span className="font-mono text-xs">{selected.log.trigger}</span>,
                  },
                  {
                    label: "Timestamp",
                    value: (
                      <span className="font-mono text-xs">
                        {dayjs(selected.log.created_at).format("YYYY-MM-DD HH:mm:ss.SSS")}
                      </span>
                    ),
                  },
                  {
                    label: "Timing",
                    value: (
                      <span className="font-mono text-xs tabular-nums">
                        {selected.deltaMs !== null
                          ? `+${formatDurationMs(selected.deltaMs)} from prev`
                          : "First step"}
                        {selected.elapsedMs > 0
                          ? ` · +${formatDurationMs(selected.elapsedMs)} total`
                          : ""}
                      </span>
                    ),
                  },
                  {
                    label: "Log ID",
                    span: 2,
                    value: <Identifier value={selected.log.id} />,
                  },
                ]}
              />

              <div className="space-y-1.5">
                <div className="flex items-center justify-between">
                  <span className="text-xs font-medium uppercase tracking-wide text-ink/50">
                    Reason
                  </span>
                  {selected.log.reason && <CopyButton text={selected.log.reason} label="Copy" />}
                </div>
                {selected.log.reason ? (
                  <pre
                    className={cn(
                      "whitespace-pre-wrap break-words rounded-lg border p-3 font-mono text-xs leading-relaxed",
                      selected.failed
                        ? "border-danger-500/30 bg-danger-500/10 text-danger-800 dark:text-danger-200"
                        : "border-ink/10 bg-sunken/70 text-ink/85",
                    )}
                  >
                    {selected.log.reason}
                  </pre>
                ) : (
                  <p className="text-xs italic text-ink/45">
                    No reason recorded for this transition.
                  </p>
                )}
              </div>
            </>
          )}
        </DialogContent>
      </Dialog>
    </div>
  );
}
