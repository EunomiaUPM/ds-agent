import React, { useMemo, useState } from "react";
import { X } from "lucide-react";
import { cn } from "shared/src/lib/utils";

export interface TagOption {
  value: string;
  label: string;
}

export interface TagInputProps {
  value: string[];
  onChange: (value: string[]) => void;
  /** Suggested values; typed text that matches none is added as typed. */
  options?: TagOption[];
  placeholder?: string;
  className?: string;
}

/** A list of values as removable chips; Enter or comma adds what is typed. */
export function TagInput({ value, onChange, options = [], placeholder, className }: TagInputProps) {
  const [draft, setDraft] = useState("");
  const [focused, setFocused] = useState(false);

  const labelOf = (v: string) => options.find((o) => o.value === v)?.label ?? v;
  const suggestions = useMemo(() => {
    const q = draft.trim().toLowerCase();
    return options
      .filter((o) => !value.includes(o.value))
      .filter((o) => !q || o.label.toLowerCase().includes(q) || o.value.toLowerCase().includes(q))
      .slice(0, 8);
  }, [draft, options, value]);

  const add = (raw: string) => {
    const item = raw.trim();
    if (item && !value.includes(item)) onChange([...value, item]);
    setDraft("");
  };

  const onKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === "Enter" || e.key === ",") {
      e.preventDefault();
      const exact = suggestions.find((o) => o.label.toLowerCase() === draft.trim().toLowerCase());
      add(exact?.value ?? draft);
    } else if (e.key === "Backspace" && !draft && value.length > 0) {
      onChange(value.slice(0, -1));
    }
  };

  return (
    <div className={cn("relative", className)}>
      <div className="flex min-h-9 flex-wrap items-center gap-1.5 rounded-md border border-ink/20 bg-transparent px-2 py-1.5 focus-within:ring-2 focus-within:ring-brand-sky/40">
        {value.map((v) => (
          <span
            key={v}
            title={v}
            className="inline-flex max-w-full items-center gap-1 rounded bg-ink/[0.08] px-1.5 py-0.5 text-xs text-ink/85"
          >
            <span className="truncate">{labelOf(v)}</span>
            <button
              type="button"
              onClick={() => onChange(value.filter((x) => x !== v))}
              className="text-ink/50 hover:text-ink"
              aria-label={`Remove ${labelOf(v)}`}
            >
              <X className="h-3 w-3" />
            </button>
          </span>
        ))}
        <input
          value={draft}
          onChange={(e) => setDraft(e.target.value)}
          onKeyDown={onKeyDown}
          onFocus={() => setFocused(true)}
          // Leave time for a click on a suggestion to land.
          onBlur={() => setTimeout(() => setFocused(false), 120)}
          placeholder={value.length === 0 ? placeholder : undefined}
          className="min-w-[8rem] flex-1 bg-transparent text-sm outline-none placeholder:text-ink/40"
        />
      </div>
      {focused && suggestions.length > 0 && (
        <ul className="absolute z-50 mt-1 max-h-56 w-full overflow-y-auto rounded-md border border-ink/15 bg-background py-1 shadow-lg">
          {suggestions.map((o) => (
            <li key={o.value}>
              <button
                type="button"
                onMouseDown={(e) => e.preventDefault()}
                onClick={() => add(o.value)}
                className="flex w-full flex-col items-start px-3 py-1.5 text-left hover:bg-ink/[0.06]"
              >
                <span className="text-sm text-ink/90">{o.label}</span>
                <span className="font-mono text-[11px] text-ink/45">{o.value}</span>
              </button>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
