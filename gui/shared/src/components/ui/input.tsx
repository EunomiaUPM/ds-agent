import * as React from "react";
import { Search, X } from "lucide-react";
import { cn } from "shared/src/lib/utils";

export interface InputProps extends React.ComponentProps<"input"> {
  startIcon?: React.ReactNode;
  endIcon?: React.ReactNode;
  containerClassName?: string;
  onClear?: () => void;
}

/**
 * Enterprise shadcn Input component with dark styling, icons, and clear button support.
 */
const Input = React.forwardRef<HTMLInputElement, InputProps>(
  (
    {
      className,
      containerClassName,
      type = "text",
      placeholder,
      startIcon,
      endIcon,
      onClear,
      value,
      ...props
    },
    ref,
  ) => {
    const isSearch = type === "search";
    const hasDecorator = Boolean(startIcon || endIcon || onClear || isSearch);

    const baseInputStyles =
      "w-full rounded-md border border-ink/15 bg-background-800/90 px-3 py-1.5 text-xs text-brand-snow placeholder:text-muted-foreground/60 transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-brand-sky/60 focus-visible:border-brand-sky/60 disabled:cursor-not-allowed disabled:opacity-50 [color-scheme:dark]";

    if (!hasDecorator) {
      return (
        <input
          {...props}
          ref={ref}
          type={type}
          value={value}
          placeholder={placeholder}
          className={cn("flex h-9", baseInputStyles, className)}
        />
      );
    }

    const defaultPlaceholder = isSearch ? "Search..." : undefined;
    const resolvedPlaceholder = placeholder !== undefined ? placeholder : defaultPlaceholder;

    return (
      <div
        className={cn(
          "flex h-9 w-full items-center rounded-md border border-ink/15 bg-background-800/90 px-2.5 text-xs text-brand-snow transition-all focus-within:border-brand-sky/60 focus-within:ring-1 focus-within:ring-brand-sky/60 hover:border-ink/25",
          containerClassName,
        )}
      >
        {startIcon ? (
          <span className="mr-2 flex items-center text-muted-foreground/70 shrink-0">
            {startIcon}
          </span>
        ) : isSearch ? (
          <Search className="mr-2 h-3.5 w-3.5 text-muted-foreground/70 shrink-0" />
        ) : null}

        <input
          {...props}
          ref={ref}
          type={type}
          value={value}
          placeholder={resolvedPlaceholder}
          className={cn(
            "w-full bg-transparent text-xs text-brand-snow placeholder:text-muted-foreground/60 focus:outline-none disabled:cursor-not-allowed disabled:opacity-50 [color-scheme:dark]",
            className,
          )}
        />

        {onClear && value && String(value).length > 0 ? (
          <button
            type="button"
            onClick={onClear}
            className="ml-1 text-muted-foreground/60 hover:text-ink transition-colors shrink-0"
          >
            <X className="h-3.5 w-3.5" />
          </button>
        ) : endIcon ? (
          <span className="ml-2 flex items-center text-muted-foreground/70 shrink-0">
            {endIcon}
          </span>
        ) : null}
      </div>
    );
  },
);
Input.displayName = "Input";

export { Input };

