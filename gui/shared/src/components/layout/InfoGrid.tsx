/**
 * InfoGrid.tsx
 *
 * Responsive grid layout for displaying information in a two-column format.
 * On mobile devices, collapses to a single column.
 *
 * Typically used in combination with InfoList components to display
 * key-value pairs in a grid arrangement.
 *
 * @example
 * // Two-column layout for info items
 * <InfoGrid>
 *   <InfoList items={[...leftColumnItems]} />
 *   <InfoList items={[...rightColumnItems]} />
 * </InfoGrid>
 */

import { cn } from "shared/src/lib/utils";
import React from "react";

// =============================================================================
// TYPES
// =============================================================================

/**
 * Props for the InfoGrid component.
 * Extends standard HTML div attributes.
 */
export interface InfoGridProps extends React.HTMLAttributes<HTMLDivElement> {}

// =============================================================================
// COMPONENT
// =============================================================================

/**
 * Responsive two-column grid for information display.
 *
 * Layout behavior:
 * - Desktop (md+): Two equal-width columns
 * - Mobile: Single column (stacked)
 *
 * @param props - Standard div props including className and children
 * @returns A responsive grid container
 */
export function InfoGrid({ className, children, ...props }: InfoGridProps) {
  return (
    <div className={cn("grid grid-cols-1 md:grid-cols-2 gap-3 mb-4", className)} {...props}>
      {children}
    </div>
  );
}
