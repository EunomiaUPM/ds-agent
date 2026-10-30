/**
 * PageHeader.tsx
 *
 * Sticky page title with optional badge, description and right-aligned actions.
 */

import Heading from "../ui/heading";
import { cn } from "../lib/utils";
import React from "react";

export interface PageHeaderProps extends Omit<React.HTMLAttributes<HTMLDivElement>, "title"> {
  title: React.ReactNode;
  /** Badge shown next to the title (state, role...). */
  badge?: React.ReactNode;
  /** Small line above the title, e.g. the kind of record. */
  eyebrow?: React.ReactNode;
  /** One line under the title. */
  description?: React.ReactNode;
  /** Buttons aligned to the right of the title. */
  actions?: React.ReactNode;
}

export function PageHeader({
  title,
  badge,
  eyebrow,
  description,
  actions,
  className,
  children,
  ...props
}: PageHeaderProps) {
  return (
    <header
      className={cn(
        "pt-4 mb-5 pb-4 border-b border-ink/10 sticky top-12 z-30 bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/80 transition-colors",
        className,
      )}
      {...props}
    >
      <div className="flex flex-wrap items-start justify-between gap-x-6 gap-y-3">
        <div className="min-w-0 flex-1">
          {eyebrow && (
            <div className="mb-1 text-xs font-medium uppercase tracking-wide text-ink/50">
              {eyebrow}
            </div>
          )}
          <Heading
            level="h1"
            className="flex flex-wrap gap-2 items-center font-display text-2xl mb-0 text-ink/90"
          >
            <span className="min-w-0 break-words">{title}</span>
            {badge}
          </Heading>
          {description && <p className="mt-1 text-sm text-ink/60">{description}</p>}
        </div>
        {actions && <div className="flex shrink-0 flex-wrap items-center gap-2">{actions}</div>}
      </div>
      {children}
    </header>
  );
}
