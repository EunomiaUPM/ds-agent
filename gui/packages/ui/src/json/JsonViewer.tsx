import React, { Suspense, useMemo, useState } from "react";
import { Check, Copy } from "lucide-react";
import { cn } from "../lib/utils";
import { useTheme } from "../hooks/useTheme";

const MonacoJson = React.lazy(() => import("./MonacoJson"));

export interface JsonViewerProps {
  value: unknown;
  className?: string;
  /** Tailwind max-height class for the editor, such as `max-h-[60vh]` or `max-h-96`. */
  maxHeightClass?: string;
}

/** Pretty-printed JSON in a read-only Monaco editor framed like a VS Code tab, with a copy button. */
export function JsonViewer({ value, className, maxHeightClass = "max-h-[60vh]" }: JsonViewerProps) {
  const { resolvedTheme } = useTheme();
  const [copied, setCopied] = useState(false);
  const text = useMemo(
    () => (typeof value === "string" ? JsonViewer.prettify(value) : JSON.stringify(value, null, 2)),
    [value],
  );
  const maxHeight = JsonViewer.pixels(maxHeightClass);

  const copy = () => {
    navigator.clipboard.writeText(text);
    setCopied(true);
    setTimeout(() => setCopied(false), 1500);
  };

  const dark = resolvedTheme === "dark";
  const lines = text.split("\n").length;

  return (
    <div
      className={cn(
        "overflow-hidden rounded-md border",
        dark ? "border-[#3c3c3c] bg-[#1e1e1e]" : "border-[#e5e5e5] bg-white",
        className,
      )}
    >
      <div
        className={cn(
          "flex h-[30px] items-center justify-between border-b pl-3 pr-1.5 font-sans text-[12px]",
          dark
            ? "border-[#3c3c3c] bg-[#252526] text-[#cccccc]"
            : "border-[#e5e5e5] bg-[#f3f3f3] text-[#616161]",
        )}
      >
        <span className="flex items-center gap-1.5">
          <span
            className={cn(
              "font-mono text-[11px] font-semibold",
              dark ? "text-[#cbcb41]" : "text-[#a16207]",
            )}
          >
            {"{}"}
          </span>
          JSON
          <span className="opacity-60">
            · {lines} {lines === 1 ? "line" : "lines"}
          </span>
        </span>
        <button
          type="button"
          onClick={copy}
          aria-label="Copy JSON"
          title="Copy"
          className={cn(
            "inline-flex h-[22px] items-center gap-1 rounded px-1.5 transition-colors",
            dark ? "hover:bg-[#3c3c3c]" : "hover:bg-[#e1e1e1]",
          )}
        >
          {copied ? <Check className="h-3.5 w-3.5" /> : <Copy className="h-3.5 w-3.5" />}
          {copied ? "Copied" : "Copy"}
        </button>
      </div>
      <Suspense
        fallback={
          <pre
            className={cn(
              "overflow-auto py-1.5 pl-[52px] pr-4 text-[12px] leading-[18px]",
              dark ? "text-[#d4d4d4]" : "text-black",
            )}
            style={{ maxHeight, fontFamily: 'Menlo, Monaco, Consolas, "Courier New", monospace' }}
          >
            {text}
          </pre>
        }
      >
        <MonacoJson text={text} dark={dark} maxHeight={maxHeight} />
      </Suspense>
    </div>
  );
}

/** Re-indents a JSON string, leaving non-JSON text untouched. */
JsonViewer.prettify = (raw: string): string => {
  try {
    return JSON.stringify(JSON.parse(raw), null, 2);
  } catch {
    return raw;
  }
};

/** The pixels a Tailwind max-height class stands for (`max-h-[60vh]`, `max-h-96`, `max-h-[30rem]`). */
JsonViewer.pixels = (heightClass: string): number => {
  const vh = typeof window === "undefined" ? 800 : window.innerHeight;
  const arbitrary = heightClass.match(/max-h-\[(\d+(?:\.\d+)?)(vh|px|rem)\]/);
  if (arbitrary) {
    const n = Number(arbitrary[1]);
    return arbitrary[2] === "vh" ? (n / 100) * vh : arbitrary[2] === "rem" ? n * 16 : n;
  }
  const scale = heightClass.match(/max-h-(\d+)$/);
  if (scale) return Number(scale[1]) * 4;
  return 0.6 * vh;
};
