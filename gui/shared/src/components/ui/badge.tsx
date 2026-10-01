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

const normalizeStatus = (status?: string): BadgeState => {
  switch ((status || "").toLowerCase()) {
    case "active":
    case "accepted":
    case "verified":
    case "started":
    case "approved":
    case "agreed":
    case "in_progress":
    case "running":
    case "valid":
      return "process";

    case "offered":
    case "requested":
    case "pending":
    case "processing":
    case "waiting":
    case "expiring":
    case "warning":
      return "warn";

    case "finalized":
    case "completed":
    case "success":
    case "confirmed":
    case "delivered":
    case "delivered_confirmed":
    case "ok":
      return "success";

    case "inactive":
    case "suspended":
    case "pause":
    case "paused":
    case "by_provider":
    case "by_consumer":
    case "on_request":
    case "stop":
    case "stopped":
    case "idle":
      return "pause";

    case "terminated":
    case "rejected":
    case "failed":
    case "error":
    case "revoked":
    case "dead_letter":
    case "dlq":
    case "expired":
      return "danger";

    default:
      return "default";
  }
};

/**
 * Badge style variants using class-variance-authority.
 * Consolidated palette supporting states, roles, and enterprise semantics.
 */
const badgeVariants = cva(
  "px-2 py-0.5 w-fit inline-flex justify-start items-center font-medium rounded-md border whitespace-nowrap shrink-0 gap-1.5 transition-all text-xs tracking-tight",
  {
    variants: {
      variant: {
        default: "bg-ink/10 text-brand-snow border-ink/15 hover:bg-ink/15",
        secondary: "bg-secondary-100 dark:bg-secondary-800/40 text-secondary-700 dark:text-secondary-200 border-secondary-600/40",
        outline: "bg-transparent text-foreground-200 border-ink/20 hover:border-ink/30",
        destructive: "bg-danger-500/15 text-danger-700 dark:text-danger-300 border-danger-500/30",
        danger: "bg-danger-500/15 text-danger-700 dark:text-danger-300 border-danger-500/30",
        success: "bg-success-500/15 text-success-700 dark:text-success-300 border-success-500/30",
        warning: "bg-warn-500/15 text-warn-700 dark:text-warn-300 border-warn-500/30",
        warn: "bg-warn-500/15 text-warn-700 dark:text-warn-300 border-warn-500/30",
        info: "font-mono uppercase bg-primary-100 dark:bg-primary-900/40 text-brand-sky border-primary-600/30",
        infoLighter: "font-mono uppercase bg-ink/10 text-secondary-700 dark:text-secondary-300 border-ink/15",
        role: "text-ink uppercase border-ink/15",
        status: "border-ink/15 text-foreground-300 uppercase tracking-wide",
        detail:
          "text-xs bg-brand-sky/15 text-brand-sky border-brand-sky/30 !px-1.5 !py-0.5 max-w-[140px] !whitespace-normal",
        code: "bg-sunken/60 border border-danger-200 dark:border-danger-900/40 rounded font-mono text-danger-700 dark:text-danger-400 !py-0 px-1.5",
        wizard:
          "bg-violet-100 dark:bg-violet-900/50 border border-violet-600/60 text-violet-700 dark:text-violet-200 uppercase tracking-wide px-3 py-0.5",
        wizardSuccess:
          "bg-success-100 dark:bg-success-900/50 border border-success-600/60 text-success-800 dark:text-success-200 uppercase tracking-wide px-3 py-0.5",
      },

      state: {
        default: "",
        process: "bg-process-400/15 text-process-800 dark:text-process-300 border-process-400/30 [&>span]:bg-process-400",
        warn: "bg-warn-500/15 text-warn-700 dark:text-warn-300 border-warn-500/30 [&>span]:bg-warn-400",
        success: "bg-success-500/15 text-success-700 dark:text-success-300 border-success-500/30 [&>span]:bg-success-400",
        pause: "bg-pause-500/15 text-pause-700 dark:text-pause-300 border-pause-500/30 [&>span]:bg-pause-400",
        danger: "bg-danger-500/15 text-danger-700 dark:text-danger-300 border-danger-500/30 [&>span]:bg-danger-400",
      },

      dsrole: {
        Provider: "bg-roles-provider/15 text-roles-provider border-roles-provider/30",
        Consumer: "bg-roles-consumer/15 text-roles-consumer border-roles-consumer/30",
        Business: "bg-roles-bussiness/15 text-roles-bussiness border-roles-bussiness/30",
        Customer: "bg-roles-customer/15 text-roles-customer border-roles-customer/30",
      },

      size: {
        xs: "text-xs px-1.5 py-0 leading-tight font-medium rounded",
        sm: "text-xs px-2 py-0.5 leading-tight font-medium rounded",
        default: "text-xs px-2.5 py-0.5 font-medium rounded-md",
        lg: "text-sm px-3 py-1 font-semibold rounded-md",
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
    Omit<VariantProps<typeof badgeVariants>, "state"> {
  /** Accept API status directly */
  state?: string;
  /** Explicit dot indicator */
  dot?: boolean;
  asChild?: boolean;
}

// =============================================================================
// COMPONENT
// =============================================================================

/**
 * Consolidated Badge component for statuses, roles, tags, and micro-labels.
 */
function Badge({
  className,
  variant,
  state,
  size,
  dsrole,
  dot,
  asChild = false,
  children,
  ...props
}: BadgeProps) {
  const Comp = asChild ? Slot : "span";

  // Infer role if not explicitly passed
  let resolvedRole = dsrole;
  if (!resolvedRole && variant === "role" && typeof children === "string") {
    const roleStr = children.trim().toLowerCase();
    if (roleStr === "provider") resolvedRole = "Provider";
    else if (roleStr === "consumer") resolvedRole = "Consumer";
    else if (roleStr === "business") resolvedRole = "Business";
    else if (roleStr === "customer") resolvedRole = "Customer";
  }

  const showDot = variant === "status" || dot === true;
  const stateStyle = normalizeStatus(state);

  return (
    <Comp
      data-slot="badge"
      className={cn(
        badgeVariants({ variant, size, state: stateStyle, dsrole: resolvedRole }),
        className,
      )}
      {...props}
    >
      {showDot && (
        <span className="w-1.5 h-1.5 rounded-full mr-1 shrink-0 inline-block bg-current shadow-[0_0_6px_currentColor]" />
      )}
      {children}
    </Comp>
  );
}

export { Badge, badgeVariants };
