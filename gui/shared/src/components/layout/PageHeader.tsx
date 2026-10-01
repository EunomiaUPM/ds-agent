/**
 * PageHeader.tsx
 *
 * Standard header component for page titles with optional badge decoration.
 * Creates a consistent heading style across all pages in the application.
 *
 * @example
 * // Basic usage
 * <PageHeader title="Datasets" />
 *
 * @example
 * // With status badge
 * <PageHeader
 *   title="Contract Negotiation"
 *   badge={<Badge variant="success">Active</Badge>}
 * />
 */

import Heading from "shared/src/components/ui/heading";
import { cn } from "shared/src/lib/utils";
import React from "react";

// =============================================================================
// TYPES
// =============================================================================

/**
 * Props for the PageHeader component.
 */
export interface PageHeaderProps extends React.HTMLAttributes<HTMLDivElement> {
  /** The main title text to display */
  title: string;

  /** Optional badge element displayed next to the title */
  badge?: React.ReactNode;
}

// =============================================================================
// COMPONENT
// =============================================================================

/**
 * Page header component that displays a formatted title with optional badge.
 *
 * Provides consistent styling for page-level headings including:
 * - Bottom border separator
 * - Proper spacing and typography
 * - Badge alignment
 *
 * @param props - PageHeader properties
 * @returns A styled page header element
 */
export function PageHeader({ title, badge, className, children, ...props }: PageHeaderProps) {
  return (
    <header
      className={cn(
        "pt-3 mb-4 pb-3 border-b border-ink/10 sticky top-12 z-30 bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/80 transition-colors",
        className,
      )}
      {...props}
    >
      <Heading level="h4" className="flex gap-2 items-center font-display mb-0 text-ink/90">
        {title}
        {badge}
      </Heading>
      {children}
    </header>
  );
}
