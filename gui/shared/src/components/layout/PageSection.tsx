/**
 * PageSection.tsx
 *
 * Section wrapper component for organizing page content into logical groups.
 * Provides consistent spacing, optional section titles, and action slots.
 *
 * @example
 * // Basic section with title
 * <PageSection title="Overview">
 *   <InfoList items={[...]} />
 * </PageSection>
 *
 * @example
 * // Section with title and action button
 * <PageSection
 *   title="Policies"
 *   action={<Button size="sm">Add Policy</Button>}
 * >
 *   <PolicyList policies={policies} />
 * </PageSection>
 */

import Heading from "shared/src/components/ui/heading";
import { cn } from "shared/src/lib/utils";
import React from "react";

// =============================================================================
// TYPES
// =============================================================================

/**
 * Props for the PageSection component.
 */
export interface PageSectionProps extends React.HTMLAttributes<HTMLDivElement> {
  /** Optional section title displayed as a small uppercase heading */
  title?: string;

  /** Optional action element (button, link) displayed next to the title */
  action?: React.ReactNode;
}

// =============================================================================
// COMPONENT
// =============================================================================

/**
 * Section wrapper for organizing page content.
 *
 * Use this component to:
 * - Group related content under a section heading
 * - Maintain consistent vertical spacing between sections
 * - Add section-level actions (e.g., "Add" buttons)
 *
 * @param props - PageSection properties
 * @returns A styled section container
 */
export function PageSection({ title, action, className, children, ...props }: PageSectionProps) {
  return (
    <div className={cn("mb-6", className)} {...props}>
      {/* Section header with title and optional action */}
      <div className="flex items-center justify-start gap-4 mb-2">
        {title && (
          <Heading
            level="h6"
            className="mt-2 text-white/80 uppercase tracking-wide text-xs font-semibold"
          >
            {title}
          </Heading>
        )}
        {action && <div className="mt-1">{action}</div>}
      </div>
      {/* Section content */}
      {children}
    </div>
  );
}
