import React, { useState } from "react";
import { Check, Copy } from "lucide-react";
import { cn } from "../lib/utils";

export interface IdentifierProps {
  value?: string | null;
  className?: string;
  /** Characters kept from the end of a URN/DID; the full value stays in the tooltip. */
  tail?: number;
  copyable?: boolean;
}

/** A long URN/DID shown short in monospace, full on hover, with a copy button. */
export function Identifier({ value, className, tail = 8, copyable = true }: IdentifierProps) {
  const [copied, setCopied] = useState(false);
  if (!value) return <span className="text-ink/35">—</span>;

  const copy = (e: React.MouseEvent) => {
    e.preventDefault();
    e.stopPropagation();
    navigator.clipboard.writeText(value);
    setCopied(true);
    setTimeout(() => setCopied(false), 1200);
  };

  return (
    <span
      className={cn("group/id inline-flex min-w-0 items-center gap-1", className)}
      title={value}
    >
      <span className="truncate font-mono text-xs text-ink/75">
        {Identifier.short(value, tail)}
      </span>
      {copyable && (
        <button
          type="button"
          onClick={copy}
          aria-label="Copy identifier"
          className="shrink-0 rounded p-0.5 text-ink/35 opacity-0 transition-opacity hover:text-ink group-hover/id:opacity-100 focus-visible:opacity-100"
        >
          {copied ? <Check className="h-3 w-3" /> : <Copy className="h-3 w-3" />}
        </button>
      )}
    </span>
  );
}

/** `urn:agreement:9a31d5ab-…` → `agreement · 9a31d5ab`; long DIDs keep method and tail. */
Identifier.short = (value: string, tail = 8): string => {
  if (value.startsWith("urn:")) {
    const [, nid, ...rest] = value.split(":");
    const nss = rest.join(":");
    return `${nid} · ${nss.slice(0, tail)}`;
  }
  if (value.startsWith("did:") && value.length > 32) {
    const method = value.split(":").slice(0, 2).join(":");
    return `${method}:…${value.slice(-tail)}`;
  }
  return value.length > 40 ? `${value.slice(0, 36)}…` : value;
};
