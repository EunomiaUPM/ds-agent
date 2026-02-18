/**
 * info-list.tsx
 *
 * A versatile component for displaying key-value pairs in a consistent format.
 * Supports multiple value types including dates, statuses, roles, URNs, and custom content.
 *
 * The InfoList component automatically formats values based on their type:
 * - Strings are displayed in info badges
 * - Dates are formatted using dayjs
 * - Statuses show colored badges with indicators
 * - URNs are truncated for readability
 * - Custom content is rendered as-is
 *
 * @example
 * // Basic usage with different value types
 * <InfoList items={[
 *   { label: "Name", value: "Dataset A" },
 *   { label: "Created", value: { type: "date", value: new Date() } },
 *   { label: "Status", value: { type: "status", value: "ACTIVE" } },
 *   { label: "ID", value: { type: "urn", value: "urn:uuid:123-456" } },
 * ]} />
 */

import React from "react";
import { Badge, BadgeState, BadgeRole } from "./badge";
import dayjs from "dayjs";
import { cn } from "shared/src/lib/utils";
import { formatUrn } from "shared/src/lib/utils";

// =============================================================================
// TYPES
// =============================================================================

/**
 * Supported value types for info list items.
 *
 * - `string`: Plain text displayed in an info badge
 * - `date`: Date value formatted as "DD/MM/YY HH:mm"
 * - `status`: Process state displayed with colored status badge
 * - `role`: User role displayed with role-colored badge
 * - `urn`: URN/UUID displayed truncated in an info badge
 * - `custom`: Custom React content rendered directly
 */
export type InfoItemValue =
  | string
  | undefined
  | null
  | { type: "date"; value: string | Date }
  | { type: "status"; value: string }
  | { type: "role"; value: string }
  | { type: "urn"; value: string | undefined }
  | { type: "custom"; content: React.ReactNode };

/**
 * Props for a single info list item.
 */
export interface InfoItemProps {
  /** Label displayed above the value */
  label: string;

  /** Value to display (see InfoItemValue for supported types) */
  value: InfoItemValue;

  /** Additional class for the item container */
  className?: string;

  /** Additional class for the label element */
  keyClassName?: string;
}

/**
 * Props for the InfoList component.
 */
export interface InfoListProps {
  /** Array of info items to display */
  items: InfoItemProps[];

  /** Additional class for the list container */
  className?: string;
}

// =============================================================================
// COMPONENTS
// =============================================================================

/**
 * Displays a vertical list of labeled information items.
 *
 * Each item shows a small label above a formatted value.
 * Items are separated by subtle borders.
 *
 * @param props - InfoList properties
 * @returns A styled list of info items
 */
export const InfoList = ({ items, className }: InfoListProps) => {
  return (
    <div className={cn("min-w-full px-0", className)}>
      {items.map((item, index) => (
        <InfoListItem key={index} {...item} />
      ))}
    </div>
  );
};

/**
 * Single info list item component.
 * Renders a label and automatically formats the value based on its type.
 */
const InfoListItem = ({ label, value, className, keyClassName }: InfoItemProps) => {
  // Don't render if no value
  if (value === undefined || value === null) return null;

  /**
   * Renders the value based on its type.
   * String values get info badges, typed objects get specialized rendering.
   */
  const renderValue = () => {
    // Plain string: wrap in info badge
    if (typeof value === "string") {
      return <Badge variant="info">{value}</Badge>;
    }

    // Typed objects: render based on type
    if (typeof value === "object") {
      // Custom content: render directly
      if ("content" in value) return value.content;

      switch (value.type) {
        case "date":
          return <p>{dayjs(value.value).format("DD/MM/YY HH:mm")}</p>;

        case "status":
          return (
            <Badge variant="status" state={value.value as BadgeState}>
              {value.value}
            </Badge>
          );

        case "role":
          return (
            <Badge variant="role" dsrole={value.value as BadgeRole}>
              {value.value}
            </Badge>
          );

        case "urn":
          return <Badge variant="info">{formatUrn(value.value)}</Badge>;

        default:
          return null;
      }
    }
    return null;
  };

  return (
    <div className={cn("flex flex-col py-2 border-b border-white/5 last:border-0", className)}>
      {/* Label */}
      <span
        className={cn(
          "text-[10px] uppercase tracking-wide text-white/50 font-medium mb-1",
          keyClassName,
        )}
      >
        {label}
      </span>
      {/* Value */}
      <div className="text-sm font-medium text-white/90">{renderValue()}</div>
    </div>
  );
};
