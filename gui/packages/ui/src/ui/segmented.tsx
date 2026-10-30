import React from "react";
import { cn } from "../lib/utils";

export interface SegmentedOption<T extends string> {
  value: T;
  label: string;
  /** Tooltip explaining the option. */
  hint?: string;
}

/** A small set of mutually exclusive options shown side by side. */
export function SegmentedControl<T extends string>({
  value,
  onChange,
  options,
  className,
  ariaLabel,
}: {
  value: T;
  onChange: (value: T) => void;
  options: SegmentedOption<T>[];
  className?: string;
  ariaLabel?: string;
}) {
  return (
    <div
      role="radiogroup"
      aria-label={ariaLabel}
      className={cn("inline-flex rounded-lg border border-ink/10 bg-ink/[0.03] p-0.5", className)}
    >
      {options.map((o) => (
        <button
          key={o.value}
          type="button"
          role="radio"
          aria-checked={value === o.value}
          title={o.hint}
          onClick={() => onChange(o.value)}
          className={cn(
            "rounded-md px-3 py-1 text-sm transition-colors",
            value === o.value
              ? "bg-background font-medium text-ink shadow-sm ring-1 ring-ink/10"
              : "text-ink/55 hover:text-ink",
          )}
        >
          {o.label}
        </button>
      ))}
    </div>
  );
}
