import React, { useState } from "react";
import { Check, Copy } from "lucide-react";
import { cn } from "@eunomia/ui";

export function CopyButton({
  text,
  label,
  className,
}: {
  text: string;
  label?: string;
  className?: string;
}) {
  const [copied, setCopied] = useState(false);

  const handleCopy = (e: React.MouseEvent<HTMLButtonElement>) => {
    e.stopPropagation();
    navigator.clipboard
      ?.writeText(text)
      .then(() => {
        setCopied(true);
        setTimeout(() => setCopied(false), 1500);
      })
      .catch(() => {
        /* Clipboard may be unavailable in insecure contexts. */
      });
  };

  return (
    <button
      type="button"
      onClick={handleCopy}
      title={copied ? "Copied!" : "Copy to clipboard"}
      aria-label={copied ? "Copied to clipboard" : "Copy to clipboard"}
      className={cn(
        "inline-flex shrink-0 items-center gap-1 rounded p-1 text-ink/55 transition-colors hover:bg-ink/[0.06] hover:text-ink",
        className,
      )}
    >
      {copied ? (
        <Check className="h-3.5 w-3.5 text-success-700 dark:text-success-400" />
      ) : (
        <Copy className="h-3.5 w-3.5" />
      )}
      {label && <span className="text-xs font-medium">{copied ? "Copied" : label}</span>}
    </button>
  );
}
