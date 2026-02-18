/**
 * PageLayout.tsx
 *
 * Root container component for page content.
 * Provides consistent horizontal padding and vertical spacing.
 *
 * Use this as the outermost wrapper for page content, typically
 * containing PageHeader and PageSection components.
 *
 * @example
 * <PageLayout>
 *   <PageHeader title="Datasets" />
 *   <PageSection title="Overview">
 *     <InfoList items={...} />
 *   </PageSection>
 * </PageLayout>
 */

import { cn } from "shared/src/lib/utils";
import React from "react";

// =============================================================================
// TYPES
// =============================================================================

/**
 * Props for the PageLayout component.
 * Extends standard HTML div attributes.
 */
export interface PageLayoutProps extends React.HTMLAttributes<HTMLDivElement> {}

// =============================================================================
// COMPONENT
// =============================================================================

/**
 * Root page container with consistent spacing.
 *
 * Provides:
 * - Horizontal padding (px-3)
 * - Vertical spacing between children (space-y-2)
 * - Bottom padding to prevent content from touching edges
 * - Full width by default
 *
 * @param props - Standard div props plus custom className
 * @returns A styled page container
 */
export function PageLayout({ className, children, ...props }: PageLayoutProps) {
  return (
    <div className={cn("space-y-2 pb-2 px-3 w-full", className)} {...props}>
      {children}
    </div>
  );
}
