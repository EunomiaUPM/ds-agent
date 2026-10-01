import * as React from "react";

import { cn } from "shared/src/lib/utils";

/**
 * Textarea component for multi-line text input.
 */
const Textarea = React.forwardRef<HTMLTextAreaElement, React.ComponentProps<"textarea">>(
  ({ className, ...props }, ref) => {
    return (
      <textarea
        className={cn(
          "flex min-h-[60px] w-full rounded-md border border-ink/15 bg-background-800/90 px-3 py-2 text-xs text-brand-snow shadow-sm placeholder:text-muted-foreground/60 transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-brand-sky/60 focus-visible:border-brand-sky/60 disabled:cursor-not-allowed disabled:opacity-50 [color-scheme:dark]",
          className,
        )}
        ref={ref}
        {...props}
      />
    );
  },
);
Textarea.displayName = "Textarea";

export { Textarea };
