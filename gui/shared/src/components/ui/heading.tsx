/**
 * heading.tsx
 *
 * Universal heading component with consistent styling across all heading levels.
 * Automatically applies appropriate typography, spacing, and visual hierarchy.
 *
 * Supports semantic heading levels (h1-h6) plus custom variants for
 * specific contexts like tables, small titles, and subtitles.
 *
 * @example
 * // Page title
 * <Heading level="h1">Dashboard</Heading>
 *
 * @example
 * // Section heading
 * <Heading level="h4">Recent Activities</Heading>
 *
 * @example
 * // Table header
 * <Heading level="table">Status</Heading>
 */

"use client";

import React, { FC } from "react";
import { cn } from "shared/src/lib/utils";
import IntrinsicElements = JSX.IntrinsicElements;

// =============================================================================
// TYPES
// =============================================================================

/**
 * Available heading levels and variants.
 *
 * Standard levels:
 * - `h1`: Page titles (largest)
 * - `h2`: Major sections
 * - `h3`: Subsections
 * - `h4`: Card/panel titles
 * - `h5`: Small section headers
 * - `h6`: Smallest, uppercase labels
 *
 * Custom variants:
 * - `table`: Table column headers
 * - `title-sm`: Small titles for compact spaces
 * - `subtitle`: Descriptive subtitle text
 */
export type HeadingLevel =
  | "h1"
  | "h2"
  | "h3"
  | "h4"
  | "h5"
  | "h6"
  | "table"
  | "title-sm"
  | "subtitle";

/**
 * Props for the Heading component.
 */
export interface HeadingProps {
  /** Heading level or variant determining size and styling */
  level: HeadingLevel;

  /** Content to display */
  children: React.ReactNode;

  /** Additional CSS classes */
  className?: string;

  /** Ref for the heading element */
  ref?: React.Ref<HTMLHeadingElement>;
}

// =============================================================================
// STYLING CONFIGURATION
// =============================================================================

/**
 * Maps heading levels to their rendered HTML elements.
 * Custom variants render as appropriate semantic elements.
 */
const levelToElement: Record<HeadingLevel, keyof IntrinsicElements> = {
  h1: "h1",
  h2: "h2",
  h3: "h3",
  h4: "h4",
  h5: "h5",
  h6: "h6",
  table: "h6",
  "title-sm": "h6",
  subtitle: "h5",
};

/**
 * Size and style classes for each heading level.
 */
const sizeClasses: Record<HeadingLevel, string> = {
  h1: "text-2xl mb-4 font-semibold font-title tracking-tight",
  h2: "text-xl mb-3 font-semibold tracking-tight",
  h3: "text-lg mb-3 font-medium tracking-tight",
  h4: "text-base mb-2 font-medium font-display tracking-normal text-white/90",
  h5: "text-sm text-white/80 mb-2 font-medium tracking-wide",
  h6: "text-xs font-semibold mb-1 uppercase tracking-wider text-muted-foreground",
  table: "text-xs font-semibold uppercase tracking-wider",
  "title-sm": "text-sm font-medium mb-1 leading-snug",
  subtitle: "text-base mb-2 font-normal text-muted-foreground max-w-[65ch]",
};

// =============================================================================
// COMPONENT
// =============================================================================

/**
 * Semantic heading component with consistent typography.
 *
 * Provides a unified interface for all heading needs across the application,
 * ensuring visual consistency while maintaining semantic HTML structure.
 *
 * @param props - Heading properties
 * @returns A styled heading element
 */
const Heading: FC<HeadingProps> = ({ level = "h1", children, ref, className = "" }) => {
  const Component = levelToElement[level] || "h1";
  const baseClasses = "text-foreground-100 text-balance";

  return (
    <Component className={cn(baseClasses, sizeClasses[level], className)}>{children}</Component>
  );
};

export default Heading;
