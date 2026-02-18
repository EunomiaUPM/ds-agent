/**
 * badge.tsx
 *
 * Versatile badge component with multiple visual variants for different contexts.
 * Supports status indicators, role badges, size variations, and custom styling.
 *
 * Variants:
 * - `default`: Standard badge with subtle styling
 * - `info`: Monospace uppercase badge for IDs and technical values
 * - `infoLighter`: Lighter version of info badge
 * - `role`: Role indicator with role-specific coloring
 * - `status`: Process state badge with colored dot indicator
 *
 * @example
 * // Simple badge
 * <Badge>Label</Badge>
 *
 * @example
 * // Status badge with state coloring
 * <Badge variant="status" state="ACTIVE">Active</Badge>
 *
 * @example
 * // Role badge
 * <Badge variant="role" dsrole="Provider">Provider</Badge>
 */

import * as React from "react";
import { Slot } from "@radix-ui/react-slot";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "shared/src/lib/utils";

// =============================================================================
// STYLE VARIANTS
// =============================================================================

/**
 * Badge style variants using class-variance-authority.
 *
 * Provides consistent styling across different badge types while
 * supporting state-based coloring for process statuses and roles.
 */
const badgeVariants = cva(
  // Base styles applied to all badges
  "px-1.5 py-0.5 w-fit inline-flex justify-start items-center bg-white/5 font-medium rounded-[4px] border border-white/10 whitespace-nowrap shrink-0 gap-1 transition-all",
  {
    variants: {
      /**
       * Visual variant determining the badge's purpose and styling.
       */
      variant: {
        default: "bg-brand-snow/15 text-brand-snow border-white/10",
        info: "font-mono uppercase bg-background-800 text-secondary-400 border-white/10",
        infoLighter: "font-mono uppercase bg-white/10 text-secondary-400 border-white/10",
        role: "text-white uppercase border-white/10",
        status: "bg-opacity-30 border-white/10 text-foreground-300",
      },

      /**
       * Process state for status badges.
       * Maps directly to DSP protocol states.
       */
      state: {
        default: "",
        danger: "",
        warn: "",
        // Active/processing states
        ACTIVE: "bg-process text-process-300 [&>span]:bg-process-400",
        INACTIVE: "bg-paused text-paused-300 [&>span]:bg-paused-400",
        ACCEPTED: "bg-process text-process-300 [&>span]:bg-process-400",
        VERIFIED: "bg-process text-process-300 [&>span]:bg-process-400",
        STARTED: "bg-process text-process-300 [&>span]:bg-process-400",
        // Pending states
        OFFERED: "bg-warn text-warn-300 [&>span]:bg-warn-400",
        REQUESTED: "bg-warn text-warn-300 [&>span]:bg-warn-400",
        AGREED: "bg-process text-process-300 [&>span]:bg-process-400",
        // Success states
        FINALIZED: "bg-success text-success-300 [&>span]:bg-success-400",
        COMPLETED: "bg-success text-success-300 [&>span]:bg-success-400",
        // Paused states
        SUSPENDED: "bg-pause text-pause-300 [&>span]:bg-pause-400",
        PAUSE: "bg-pause text-pause-300 [&>span]:bg-pause-400",
        BY_PROVIDER: "bg-pause text-pause-300 [&>span]:bg-pause-400",
        BY_CONSUMER: "bg-pause text-pause-300 [&>span]:bg-pause-400",
        ON_REQUEST: "bg-pause text-pause-300 [&>span]:bg-pause-400",
        STOP: "bg-pause text-pause-300 [&>span]:bg-pause-400",
        STOPPED: "bg-pause text-pause-300 [&>span]:bg-pause-400",
        // Error state
        TERMINATED: "bg-danger text-danger-300 [&>span]:bg-danger-400",
      },

      /**
       * Role type for role badges.
       * Each role has a distinct color theme.
       */
      dsrole: {
        Provider: "bg-roles-provider/30 border-roles-provider/40",
        Consumer: "bg-roles-consumer/30 border-roles-consumer/40",
        Business: "bg-roles-business/30 border-roles-business/40",
        Customer: "bg-roles-customer/30 border-roles-customer/40",
      },

      /**
       * Size variant for different contexts.
       */
      size: {
        default: "text-xs px-2 py-0.5",
        lg: "text-sm font-bold px-2 py-1",
        sm: "text-[10px] px-1.5 py-0.5 leading-none",
      },
    },
    defaultVariants: {
      variant: "default",
      state: "default",
      size: "default",
    },
  },
);

// =============================================================================
// TYPES
// =============================================================================

/** Available badge states for status variant */
export type BadgeState = VariantProps<typeof badgeVariants>["state"];

/** Available role types for role variant */
export type BadgeRole = VariantProps<typeof badgeVariants>["dsrole"];

/**
 * Props for the Badge component.
 * Extends span attributes with variant options.
 */
export interface BadgeProps
  extends React.ComponentProps<"span">,
    VariantProps<typeof badgeVariants> {
  /** Render as a different element using Radix Slot */
  asChild?: boolean;
}

// =============================================================================
// COMPONENT
// =============================================================================

/**
 * Badge component for displaying status, roles, or labels.
 *
 * Features:
 * - Multiple visual variants for different contexts
 * - State-based coloring for process statuses
 * - Role-based coloring for participant types
 * - Size variations for different layouts
 * - Slot support for rendering as child element
 *
 * @param props - Badge properties including variant, state, size, and role
 * @returns A styled badge element
 */
function Badge({
  className,
  variant,
  state,
  size,
  dsrole,
  asChild = false,
  children,
  ...props
}: BadgeProps) {
  const Comp = asChild ? Slot : "span";

  // Status badges show a colored dot indicator
  const showDot = variant === "status";

  return (
    <Comp
      data-slot="badge"
      className={cn(badgeVariants({ variant, size, state, dsrole }), className)}
      {...props}
    >
      {/* Status dot indicator */}
      {showDot && <span className={cn("w-2 h-2 rounded-full mr-1 mb-[2px]")} />}
      {children}
    </Comp>
  );
}

export { Badge, badgeVariants };
