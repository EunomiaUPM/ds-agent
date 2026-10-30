import React from "react";
import { Check, Pause, X } from "lucide-react";
import { cn } from "../lib/utils";

export interface StateStepperProps {
  /** The happy path of the protocol, in order. */
  steps: string[];
  /** Current state of the process; may be off the path (TERMINATED, SUSPENDED). */
  current: string;
  /** States the process went through, from its message history; unknown while it loads. */
  visited?: Set<string>;
  /** Off-path states and the path step they hang from. */
  detours?: Record<string, { at?: string; tone: "danger" | "warn" }>;
}

/** The protocol path as steps: done, current, skipped or pending, plus a terminal detour. */
export function StateStepper({ steps, current, visited, detours = {} }: StateStepperProps) {
  const detour = detours[current];
  // On a detour the process stopped at the last path step it reached.
  const anchor = detour
    ? (detour.at ?? [...steps].reverse().find((s) => visited?.has(s)) ?? steps[0])
    : current;
  const anchorIndex = steps.indexOf(anchor);

  return (
    <ol className="flex w-full items-start overflow-x-auto rounded-lg border border-ink/10 bg-ink/[0.015] px-4 py-3">
      {steps.map((step, i) => {
        const isAnchor = i === anchorIndex;
        const done = i < anchorIndex || (isAnchor && !detour && step === steps[steps.length - 1]);
        const skipped = !!visited && i < anchorIndex && !visited.has(step);
        const active = isAnchor && !done;
        const tone = isAnchor && detour ? detour.tone : undefined;

        return (
          <li key={step} className="flex min-w-[96px] flex-1 items-start">
            <div className="flex flex-col items-center gap-1.5 text-center">
              <span
                className={cn(
                  "flex h-6 w-6 items-center justify-center rounded-full border text-[0.6875rem] font-semibold",
                  done && !skipped && "border-brand-sky bg-brand-sky text-background",
                  skipped && "border-dashed border-ink/25 text-ink/30",
                  active && !tone && "border-brand-sky text-brand-sky ring-4 ring-brand-sky/15",
                  tone === "danger" && "border-danger-500 bg-danger-500 text-white",
                  tone === "warn" && "border-warn-500 bg-warn-500 text-white",
                  !done && !active && !tone && "border-ink/20 text-ink/35",
                )}
              >
                {tone === "danger" ? (
                  <X className="h-3.5 w-3.5" />
                ) : tone === "warn" ? (
                  <Pause className="h-3 w-3" />
                ) : done && !skipped ? (
                  <Check className="h-3.5 w-3.5" />
                ) : (
                  i + 1
                )}
              </span>
              <span
                className={cn(
                  "text-[0.6875rem] font-medium uppercase tracking-wide",
                  active || tone ? "text-ink" : done && !skipped ? "text-ink/70" : "text-ink/35",
                )}
              >
                {step}
              </span>
              {tone && (
                <span
                  className={cn(
                    "text-[0.6875rem] font-semibold uppercase",
                    tone === "danger" ? "text-danger-500" : "text-warn-500",
                  )}
                >
                  {current}
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
  );
}
