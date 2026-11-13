import React, { useMemo, useState } from "react";
import dayjs from "dayjs";
import { AlertTriangle, BarChart3, Bug, Info, RefreshCw, Search, XCircle } from "lucide-react";
import {
  Badge,
  Button,
  ChartConfig,
  ChartContainer,
  ChartTooltip,
  ChartTooltipContent,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  FieldGrid,
  Identifier,
  Input,
  JsonTable,
  RelativeTime,
  SegmentedControl,
  Skeleton,
  cn,
} from "@eunomia/ui";
import { Bar, BarChart, CartesianGrid, XAxis, YAxis } from "recharts";
import { TransferEventDto } from "shared/src/data/orval/model";
import { CopyButton } from "./CopyButton";
import { EventData } from "./EventData";

interface TransferEventsTabProps {
  events: TransferEventDto[];
  isLoading?: boolean;
  isRefreshing?: boolean;
  onRefresh?: () => void;
}

type LevelFilter = "All" | "Error" | "Warn" | "Info" | "Debug";

const SORT_OPTIONS: Array<{ value: "desc" | "asc"; label: string }> = [
  { value: "desc", label: "Newest first" },
  { value: "asc", label: "Oldest first" },
];

const chartConfig = {
  info: { label: "Info / Debug", color: "#38bdf8" },
  warn: { label: "Warn", color: "#f59e0b" },
  error: { label: "Error", color: "#ef4444" },
} satisfies ChartConfig;

interface ChartPoint {
  time: string;
  info: number;
  warn: number;
  error: number;
}

function chooseBucketSizeMs(spanMs: number): number {
  if (spanMs <= 3 * 60_000) return 10_000;
  if (spanMs <= 90 * 60_000) return 60_000;
  return 15 * 60_000;
}

function useEventsChartData(events: TransferEventDto[]): {
  points: ChartPoint[];
  bucketLabel: string;
} {
  return useMemo(() => {
    const timestamps = events
      .map((e) => new Date(e.created_at).getTime())
      .filter((t) => Number.isFinite(t));
    if (timestamps.length === 0) return { points: [], bucketLabel: "per minute" };

    const minRaw = Math.min(...timestamps);
    const maxRaw = Math.max(...timestamps);
    const bucketMs = chooseBucketSizeMs(maxRaw - minRaw);

    const bucketMap = new Map<number, { info: number; warn: number; error: number }>();
    for (const ev of events) {
      const t = new Date(ev.created_at).getTime();
      if (!Number.isFinite(t)) continue;
      const b = Math.floor(t / bucketMs) * bucketMs;
      const entry = bucketMap.get(b) ?? { info: 0, warn: 0, error: 0 };
      if (ev.level === "Error") entry.error += 1;
      else if (ev.level === "Warn") entry.warn += 1;
      else entry.info += 1;
      bucketMap.set(b, entry);
    }

    const firstBucket = Math.floor(minRaw / bucketMs) * bucketMs;
    const lastBucket = Math.floor(maxRaw / bucketMs) * bucketMs;
    const padStart = firstBucket - bucketMs;
    const minVisibleSpan = bucketMs * 6;
    const padEnd = Math.max(lastBucket + bucketMs, padStart + minVisibleSpan);

    const fmt = bucketMs < 60_000 ? "HH:mm:ss" : "HH:mm";
    const points: ChartPoint[] = [];
    for (let t = padStart; t <= padEnd; t += bucketMs) {
      const counts = bucketMap.get(t) ?? { info: 0, warn: 0, error: 0 };
      points.push({
        time: dayjs(t).format(fmt),
        ...counts,
      });
    }

    const bucketLabel =
      bucketMs === 10_000 ? "per 10s" : bucketMs === 60_000 ? "per minute" : "per 15 min";

    return { points, bucketLabel };
  }, [events]);
}

function LevelBadge({ level }: { level?: string }) {
  if (level === "Error") {
    return (
      <Badge variant="status" state="TERMINATED" size="sm">
        <XCircle className="h-3 w-3" />
        Error
      </Badge>
    );
  }
  if (level === "Warn") {
    return (
      <Badge variant="status" state="OFFERED" size="sm">
        <AlertTriangle className="h-3 w-3" />
        Warn
      </Badge>
    );
  }
  if (level === "Info") {
    return (
      <Badge variant="status" state="STARTED" size="sm">
        <Info className="h-3 w-3" />
        Info
      </Badge>
    );
  }
  return (
    <Badge variant="secondary" size="sm">
      <Bug className="h-3 w-3" />
      {level ?? "Debug"}
    </Badge>
  );
}

/** Diagnostic events emitted by the data proxy and relays, with severity filters and inspection. */
export function TransferEventsTab({
  events,
  isLoading,
  isRefreshing,
  onRefresh,
}: TransferEventsTabProps) {
  const [levelFilter, setLevelFilter] = useState<LevelFilter>("All");
  const [query, setQuery] = useState("");
  const [order, setOrder] = useState<"desc" | "asc">("desc");
  const [showChart, setShowChart] = useState(true);
  const [selected, setSelected] = useState<TransferEventDto | null>(null);

  const { points: chartData, bucketLabel } = useEventsChartData(events);

  const counts = useMemo(() => {
    const map = { All: events.length, Error: 0, Warn: 0, Info: 0, Debug: 0 };
    for (const ev of events) {
      if (ev.level === "Error") map.Error += 1;
      else if (ev.level === "Warn") map.Warn += 1;
      else if (ev.level === "Info") map.Info += 1;
      else if (ev.level === "Debug") map.Debug += 1;
    }
    return map;
  }, [events]);

  const components = useMemo(
    () => Array.from(new Set(events.map((e) => e.component).filter(Boolean))),
    [events],
  );

  const visible = useMemo(() => {
    const needle = query.trim().toLowerCase();
    const filtered = events.filter((ev) => {
      if (levelFilter !== "All" && ev.level !== levelFilter) return false;
      if (!needle) return true;
      const dataStr = ev.data ? JSON.stringify(ev.data).toLowerCase() : "";
      return (
        ev.message.toLowerCase().includes(needle) ||
        ev.component.toLowerCase().includes(needle) ||
        ev.level.toLowerCase().includes(needle) ||
        dataStr.includes(needle)
      );
    });
    return [...filtered].sort((a, b) => {
      const diff = new Date(a.created_at).getTime() - new Date(b.created_at).getTime();
      return order === "asc" ? diff : -diff;
    });
  }, [events, levelFilter, query, order]);

  if (isLoading && events.length === 0) {
    return (
      <div className="space-y-4">
        <Skeleton className="h-36 w-full rounded-lg" />
        <Skeleton className="h-44 w-full rounded-lg" />
      </div>
    );
  }

  if (events.length === 0) {
    return (
      <div className="rounded-lg border border-dashed border-ink/15 px-6 py-12 text-center">
        <p className="text-sm font-medium text-ink/70">No data plane events recorded yet</p>
        <p className="mx-auto mt-1 max-w-md text-xs text-ink/50">
          HTTP proxy requests, SSRF or auth rejections, finite-quota deliveries, and Kafka relay
          records appear here once data traffic flows.
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

  const filterLevels: LevelFilter[] =
    counts.Debug > 0 ? ["All", "Error", "Warn", "Info", "Debug"] : ["All", "Error", "Warn", "Info"];

  return (
    <div className="space-y-4">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div className="flex flex-wrap items-center gap-1.5">
          {filterLevels.map((lvl) => {
            const count = counts[lvl];
            const active = levelFilter === lvl;
            return (
              <button
                key={lvl}
                type="button"
                onClick={() => setLevelFilter(lvl)}
                className={cn(
                  "inline-flex items-center gap-1.5 rounded-md border px-2.5 py-1 text-xs font-medium transition-colors",
                  active
                    ? "border-brand-sky/50 bg-brand-sky/15 text-ink"
                    : "border-ink/10 bg-ink/[0.02] text-ink/65 hover:border-ink/20 hover:text-ink",
                )}
              >
                <span>{lvl}</span>
                <span className="rounded-full bg-ink/10 px-1.5 text-[0.6875rem] tabular-nums">
                  {count}
                </span>
              </button>
            );
          })}
          {components.map((comp) => (
            <span
              key={comp}
              className="hidden rounded-md border border-ink/10 bg-ink/[0.03] px-2 py-1 font-mono text-xs text-ink/55 sm:inline-flex"
            >
              {comp}
            </span>
          ))}
        </div>

        <div className="flex flex-wrap items-center gap-2">
          <Input
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            onClear={query ? () => setQuery("") : undefined}
            placeholder="Search message, target, status..."
            startIcon={<Search className="h-3.5 w-3.5" />}
            containerClassName="w-56 sm:w-64"
            className="h-8 text-xs"
          />
          <SegmentedControl
            value={order}
            onChange={setOrder}
            options={SORT_OPTIONS}
            ariaLabel="Events sort order"
          />
          <Button
            type="button"
            variant={showChart ? "secondary" : "outline"}
            size="xs"
            onClick={() => setShowChart((v) => !v)}
          >
            <BarChart3 className="h-3.5 w-3.5" />
            Chart
          </Button>
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

      {showChart && chartData.length > 0 && (
        <div className="rounded-lg border border-ink/10 bg-ink/[0.015] p-4">
          <div className="mb-3 flex flex-wrap items-center justify-between gap-2">
            <span className="text-xs font-medium uppercase tracking-wide text-ink/50">
              Activity · Events {bucketLabel}
            </span>
            <div className="flex items-center gap-3 text-xs text-ink/60">
              <span className="inline-flex items-center gap-1.5">
                <span className="h-2 w-2 rounded-sm bg-[#38bdf8]" />
                Info ({counts.Info + counts.Debug})
              </span>
              <span className="inline-flex items-center gap-1.5">
                <span className="h-2 w-2 rounded-sm bg-[#f59e0b]" />
                Warn ({counts.Warn})
              </span>
              <span className="inline-flex items-center gap-1.5">
                <span className="h-2 w-2 rounded-sm bg-[#ef4444]" />
                Error ({counts.Error})
              </span>
            </div>
          </div>
          <ChartContainer config={chartConfig} className="h-[150px] w-full">
            <BarChart data={chartData} margin={{ top: 6, right: 12, left: 0, bottom: 0 }}>
              <CartesianGrid vertical={false} strokeDasharray="3 3" />
              <XAxis
                dataKey="time"
                tickLine={false}
                axisLine={false}
                tickMargin={6}
                tick={{ fontSize: 11 }}
              />
              <YAxis
                tickLine={false}
                axisLine={false}
                tickMargin={6}
                tick={{ fontSize: 11 }}
                allowDecimals={false}
                width={26}
              />
              <ChartTooltip content={<ChartTooltipContent />} />
              <Bar dataKey="info" stackId="lvl" fill="var(--color-info)" barSize={14} />
              <Bar dataKey="warn" stackId="lvl" fill="var(--color-warn)" barSize={14} />
              <Bar
                dataKey="error"
                stackId="lvl"
                fill="var(--color-error)"
                radius={[3, 3, 0, 0]}
                barSize={14}
              />
            </BarChart>
          </ChartContainer>
        </div>
      )}

      {visible.length === 0 ? (
        <div className="rounded-lg border border-dashed border-ink/15 py-10 text-center text-sm text-ink/50">
          No events match the active filters.
        </div>
      ) : (
        <div className="overflow-x-auto rounded-lg border border-ink/10">
          <table className="w-full text-sm">
            <thead className="bg-ink/[0.03] text-left text-xs font-medium text-ink/55">
              <tr>
                <th className="whitespace-nowrap px-3 py-2.5 font-medium">When</th>
                <th className="whitespace-nowrap px-3 py-2.5 font-medium">Level</th>
                <th className="whitespace-nowrap px-3 py-2.5 font-medium">Component</th>
                <th className="px-3 py-2.5 font-medium">Message &amp; telemetry</th>
              </tr>
            </thead>
            <tbody>
              {visible.map((event, i) => {
                const isError = event.level === "Error";
                const isWarn = event.level === "Warn";
                const hasData = Boolean(
                  event.data &&
                    typeof event.data === "object" &&
                    !Array.isArray(event.data) &&
                    Object.keys(event.data as Record<string, unknown>).length > 0,
                );
                return (
                  <tr
                    key={event.id ?? i}
                    onClick={() => setSelected(event)}
                    className={cn(
                      "cursor-pointer border-t border-ink/[0.07] align-top transition-colors hover:bg-ink/[0.03]",
                      isError && "bg-danger-500/[0.04] hover:bg-danger-500/[0.08]",
                      isWarn && "bg-warn-500/[0.04] hover:bg-warn-500/[0.08]",
                    )}
                  >
                    <td
                      className="whitespace-nowrap px-3 py-2.5 text-xs"
                      title={dayjs(event.created_at).format("YYYY-MM-DD HH:mm:ss.SSS")}
                    >
                      <div className="font-mono tabular-nums text-ink/85">
                        {dayjs(event.created_at).format("HH:mm:ss.SSS")}
                      </div>
                      <div className="text-[0.6875rem] text-ink/45">
                        <RelativeTime date={event.created_at} />
                      </div>
                    </td>
                    <td className="whitespace-nowrap px-3 py-2.5">
                      <LevelBadge level={event.level} />
                    </td>
                    <td className="whitespace-nowrap px-3 py-2.5">
                      <span className="rounded border border-ink/10 bg-ink/[0.04] px-1.5 py-0.5 font-mono text-xs text-ink/75">
                        {event.component || "—"}
                      </span>
                    </td>
                    <td className="min-w-[300px] px-3 py-2.5">
                      <div className="space-y-1.5">
                        <p className="break-words text-xs font-medium leading-relaxed text-ink/90">
                          {event.message || <span className="italic text-ink/40">—</span>}
                        </p>
                        {hasData && <EventData data={event.data as Record<string, unknown>} />}
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
        <DialogContent className="max-h-[90vh] max-w-3xl overflow-y-auto">
          {selected && (
            <>
              <DialogHeader>
                <DialogTitle className="flex flex-wrap items-center gap-2">
                  <LevelBadge level={selected.level} />
                  <span>{selected.component || "Transfer event"}</span>
                </DialogTitle>
                <DialogDescription className="font-mono text-xs">
                  {dayjs(selected.created_at).format("YYYY-MM-DD HH:mm:ss.SSS")}
                </DialogDescription>
              </DialogHeader>

              <FieldGrid
                columns={2}
                fields={[
                  { label: "Level", value: <LevelBadge level={selected.level} /> },
                  {
                    label: "Component",
                    value: <span className="font-mono text-xs">{selected.component || "—"}</span>,
                  },
                  {
                    label: "Occurred",
                    value: (
                      <span className="font-mono text-xs">
                        {dayjs(selected.created_at).format("YYYY-MM-DD HH:mm:ss.SSS")} (
                        <RelativeTime date={selected.created_at} />)
                      </span>
                    ),
                  },
                  { label: "Event ID", value: <Identifier value={selected.id} /> },
                ]}
              />

              <div className="space-y-1.5">
                <div className="flex items-center justify-between">
                  <span className="text-xs font-medium uppercase tracking-wide text-ink/50">
                    Message
                  </span>
                  {selected.message && <CopyButton text={selected.message} label="Copy" />}
                </div>
                <pre className="whitespace-pre-wrap break-words rounded-lg border border-ink/10 bg-sunken/70 p-3 font-mono text-xs leading-relaxed text-ink/85">
                  {selected.message}
                </pre>
              </div>

              {selected.data !== undefined && selected.data !== null && (
                <div className="space-y-1.5">
                  <span className="text-xs font-medium uppercase tracking-wide text-ink/50">
                    Event payload
                  </span>
                  <JsonTable
                    value={selected.data}
                    expandDepth={2}
                    emptyText="The event has no extra data."
                  />
                </div>
              )}
            </>
          )}
        </DialogContent>
      </Dialog>
    </div>
  );
}
