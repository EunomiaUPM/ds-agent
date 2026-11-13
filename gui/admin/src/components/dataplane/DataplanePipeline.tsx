import React, { useMemo } from "react";
import dayjs from "dayjs";
import { AlertTriangle, Check, Pause, X } from "lucide-react";
import { cn } from "@eunomia/ui";
import { DataplaneTransferLogDto } from "shared/src/data/orval/model";
import { formatDurationMs, isFailureLog } from "./utils/dataplaneState";

const PULL_STEPS = ["Init", "Configuring", "Auth", "Ready", "Started"];
const PUSH_STEPS = ["Init", "Configuring", "Auth", "Ready", "Subscribing", "Started"];

type DetourTone = "danger" | "warn" | "pause" | "success";

interface DataplanePipelineProps {
  mode: string;
  currentState: string;
  logs: DataplaneTransferLogDto[];
  tpState?: string;
  createdAt?: string;
  updatedAt?: string | null;
}

export function DataplanePipeline({
  mode,
  currentState,
  logs,
  tpState,
  createdAt,
  updatedAt,
}: DataplanePipelineProps) {
  const steps = mode.toUpperCase() === "PUSH" ? PUSH_STEPS : PULL_STEPS;

  const { visited, stageTimes, anchorIndex, detour } = useMemo(() => {
    const visitedSet = new Set<string>([currentState.toLowerCase()]);
    const times = new Map<string, string>();
    for (const log of logs) {
      if (log.previous_state) visitedSet.add(log.previous_state.toLowerCase());
      const next = log.new_state.toLowerCase();
      visitedSet.add(next);
      if (!times.has(next)) times.set(next, log.created_at);
    }

    const stepLowers = steps.map((s) => s.toLowerCase());
    const curLower = currentState.toLowerCase();
    const directIdx = stepLowers.indexOf(curLower);

    let resolvedDetour: { label: string; tone: DetourTone } | undefined;
    let idx = directIdx;

    if (directIdx === -1) {
      let lastVisited = -1;
      for (let i = stepLowers.length - 1; i >= 0; i -= 1) {
        if (visitedSet.has(stepLowers[i])) {
          lastVisited = i;
          break;
        }
      }
      idx = lastVisited >= 0 ? lastVisited : 0;

      const lastLog = logs[logs.length - 1];
      const failed =
        curLower === "error" ||
        (curLower === "terminated" &&
          ((lastLog && isFailureLog(lastLog)) || tpState?.toUpperCase() === "TERMINATED"));

      if (curLower === "error" || (curLower === "terminated" && failed)) {
        resolvedDetour = { label: currentState, tone: "danger" };
      } else if (curLower === "terminated") {
        resolvedDetour = { label: "Terminated (done)", tone: "success" };
      } else if (curLower === "unsubscribing") {
        resolvedDetour = { label: "Unsubscribing", tone: "warn" };
      } else {
        resolvedDetour = { label: currentState, tone: "pause" };
      }
    }

    return {
      visited: visitedSet,
      stageTimes: times,
      anchorIndex: idx,
      detour: resolvedDetour,
    };
  }, [steps, currentState, logs, tpState]);

  const elapsedMs = useMemo(() => {
    const start = logs[0]?.created_at ?? createdAt;
    const end = logs[logs.length - 1]?.created_at ?? updatedAt ?? createdAt;
    if (!start || !end) return 0;
    return Math.max(0, new Date(end).getTime() - new Date(start).getTime());
  }, [logs, createdAt, updatedAt]);

  return (
    <div className="rounded-lg border border-ink/10 bg-ink/[0.015] px-4 py-3">
      <div className="mb-3 flex flex-wrap items-center justify-between gap-2">
        <span className="text-xs font-medium uppercase tracking-wide text-ink/50">
          Dataplane lifecycle · {mode.toUpperCase()}
        </span>
        {elapsedMs > 0 && (
          <span className="font-mono text-xs tabular-nums text-ink/50">
            Elapsed: {formatDurationMs(elapsedMs)}
          </span>
        )}
      </div>

      <ol className="flex w-full items-start overflow-x-auto pb-1">
        {steps.map((step, i) => {
          const stepLower = step.toLowerCase();
          const isAnchor = i === anchorIndex;
          const reached = visited.has(stepLower);
          const done = i < anchorIndex || (isAnchor && !detour && i === steps.length - 1);
          const skipped = logs.length > 0 && i < anchorIndex && !reached;
          const active = isAnchor && !done && !detour;
          const tone = isAnchor && detour ? detour.tone : undefined;
          const ts = stageTimes.get(stepLower);

          return (
            <li key={step} className="flex min-w-[92px] flex-1 items-start">
              <div
                className="flex flex-col items-center gap-1.5 text-center"
                title={ts ? `${step} at ${dayjs(ts).format("YYYY-MM-DD HH:mm:ss.SSS")}` : step}
              >
                <span
                  className={cn(
                    "flex h-6 w-6 items-center justify-center rounded-full border text-[0.6875rem] font-semibold",
                    done && !skipped && !tone && "border-brand-sky bg-brand-sky text-background",
                    skipped && "border-dashed border-ink/25 text-ink/30",
                    active && "border-brand-sky text-brand-sky ring-4 ring-brand-sky/15",
                    tone === "danger" && "border-danger-500 bg-danger-500 text-white",
                    tone === "warn" && "border-warn-500 bg-warn-500 text-white",
                    tone === "pause" && "border-pause-500 bg-pause-500 text-white",
                    tone === "success" && "border-success-600 bg-success-600 text-white",
                    !done && !active && !tone && !skipped && "border-ink/20 text-ink/35",
                  )}
                >
                  {tone === "danger" ? (
                    <X className="h-3.5 w-3.5" />
                  ) : tone === "warn" ? (
                    <AlertTriangle className="h-3 w-3" />
                  ) : tone === "pause" ? (
                    <Pause className="h-3 w-3" />
                  ) : tone === "success" || (done && !skipped) ? (
                    <Check className="h-3.5 w-3.5" />
                  ) : (
                    i + 1
                  )}
                </span>
                <span
                  className={cn(
                    "text-[0.6875rem] font-medium uppercase tracking-wide",
                    active || tone ? "text-ink" : done && !skipped ? "text-ink/75" : "text-ink/35",
                  )}
                >
                  {step}
                </span>
                {isAnchor && detour && (
                  <span
                    className={cn(
                      "whitespace-nowrap text-[0.6875rem] font-semibold uppercase",
                      detour.tone === "danger"
                        ? "text-danger-600 dark:text-danger-400"
                        : detour.tone === "success"
                          ? "text-success-700 dark:text-success-400"
                          : detour.tone === "warn"
                            ? "text-warn-700 dark:text-warn-400"
                            : "text-pause-600 dark:text-pause-300",
                    )}
                  >
                    {detour.label}
                  </span>
                )}
              </div>
              {i < steps.length - 1 && (
                <span
                  className={cn(
                    "mx-2 mt-3 h-px flex-1",
                    i < anchorIndex ? "bg-brand-sky/60" : "bg-ink/15",
                  )}
                />
              )}
            </li>
          );
        })}
      </ol>
    </div>
  );
}
