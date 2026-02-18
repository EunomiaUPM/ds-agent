/**
 * policy.tsx
 *
 * Component library for displaying ODRL policy content.
 * Provides a set of composable components for rendering policy rules,
 * actions, and constraints with color-coded variants.
 *
 * Components:
 * - `Policy`: Root container with variant-based coloring
 * - `PolicyItemContainer`: Wrapper for individual policy items
 * - `PolicyItem`: Row container for key-value pairs
 * - `PolicyItemKey`: Label element (e.g., "action:")
 * - `PolicyItemValue`: Value element (e.g., "use")
 * - `PolicyConstraintsWrapper`: Container for constraint lists
 * - `PolicyConstraintsContainer`: Individual constraint row
 * - `PolicyConstraint`: Single constraint operand/operator
 *
 * @example
 * <Policy variant="permission">
 *   <PolicyItemContainer>
 *     <PolicyItem>
 *       <PolicyItemKey>action:</PolicyItemKey>
 *       <PolicyItemValue>use</PolicyItemValue>
 *     </PolicyItem>
 *   </PolicyItemContainer>
 * </Policy>
 */

import * as React from "react";
import { FC } from "react";
import { cn } from "shared/src/lib/utils";
import { cva, type VariantProps } from "class-variance-authority";

// =============================================================================
// STYLE VARIANTS
// =============================================================================

/**
 * Policy container variants with color-coding for different rule types.
 *
 * - `permission`: Green - allowed actions
 * - `obligation`: Amber/Yellow - required actions
 * - `prohibition`: Red - forbidden actions
 */
const policyVariants = cva("gap-1.5 border px-2 py-1 rounded-md", {
  variants: {
    variant: {
      permission: "bg-success-600/20 border-success-700 text-sucess-100",
      obligation: "bg-warn-700/20 border-warn-700 text-warn-100",
      prohibition: "bg-danger-600/20 border-danger-700 text-danger-100",
    },
  },
});

/** Exported variant type for external use */
export type PolicyVariants = VariantProps<typeof policyVariants>["variant"];

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/**
 * Returns the appropriate heading color class for a variant.
 */
const HeadingColor = ({ variant }: { variant: PolicyVariants }) => {
  switch (variant) {
    case "permission":
      return "text-success-200";
    case "obligation":
      return "text-warn-300";
    case "prohibition":
      return "text-danger-400";
    default:
      return "text-white/80";
  }
};

// =============================================================================
// TYPES
// =============================================================================

/**
 * Props for the Policy root component.
 */
export interface PolicyProps
  extends React.HTMLAttributes<HTMLDivElement>,
    VariantProps<typeof policyVariants> {
  children: React.ReactNode;
}

/**
 * Base props for policy child components.
 */
interface PolicyChildProps {
  children: React.ReactNode;
  props?: React.HTMLAttributes<HTMLDivElement>;
}

/**
 * Props for the PolicyConstraint component.
 */
interface PolicyConstraintProps extends PolicyChildProps {
  /** Type of constraint operand */
  type: "leftOperand" | "operator" | "rightOperand";
  className?: string;
}

// =============================================================================
// COMPONENTS
// =============================================================================

/**
 * Root policy container with variant-based styling.
 * Displays the variant name as a header.
 */
const Policy = React.forwardRef<HTMLDivElement, PolicyProps>(
  ({ className, variant, children, ...props }, ref) => {
    return (
      <div className={cn(policyVariants({ variant, className }))} {...props}>
        <p className={`uppercase mb-0 font-bold ${HeadingColor({ variant })}`}>{variant}</p>
        {children}
      </div>
    );
  },
);
Policy.displayName = "Policy";

/**
 * Container for a single policy item (e.g., one permission).
 * Provides vertical spacing and border separation.
 */
const PolicyItemContainer: FC<PolicyChildProps> = ({ children, ...props }) => {
  return (
    <div
      className={cn(
        "flex flex-col gap-1 py-3 border-b border-white/20 last:border-0 first:pt-1 last:pb-1",
      )}
      {...props}
    >
      {children}
    </div>
  );
};

/**
 * Heading element for policy sections.
 */
const PolicyHeading: FC<PolicyChildProps> = ({ children, ...props }) => {
  return (
    <div className={cn("flex ")} {...props}>
      {children}
    </div>
  );
};

/**
 * Row container for key-value pairs in a policy item.
 */
const PolicyItem: FC<PolicyChildProps> = ({ children, ...props }) => {
  return (
    <div className={cn("flex ")} {...props}>
      {children}
    </div>
  );
};

/**
 * Label element for policy item keys (e.g., "action:").
 */
const PolicyItemKey: FC<PolicyChildProps> = ({ children, ...props }) => {
  return (
    <div className={cn("w-32 font-semibold text-white/60")} {...props}>
      {children}
    </div>
  );
};

/**
 * Value element for policy item values.
 */
const PolicyItemValue: FC<PolicyChildProps> = ({ children, ...props }) => {
  return (
    <div className={cn("uppercase")} {...props}>
      {children}
    </div>
  );
};

/**
 * Container for a single constraint triplet (left, operator, right).
 */
const PolicyConstraintsContainer: FC<PolicyChildProps> = ({ children, ...props }) => {
  return (
    <div className={cn("flex gap-1.5 bg-black/30 w-fit p-1 rounded-md")} {...props}>
      {children}
    </div>
  );
};

/**
 * Individual constraint operand or operator display.
 * Shows the value and its type label below.
 */
const PolicyConstraint: FC<PolicyConstraintProps> = ({ type, className, children, ...props }) => {
  /**
   * Formats constraint text by removing JSON syntax characters.
   */
  const formatString = (text: string) => {
    return text.replace(/[()[\]{},\"]/g, " ");
  };

  const childText = typeof children === "string" ? children : String(children);

  return (
    <div className={cn("constraint-policy-container", className)} {...props}>
      {/* Constraint value */}
      <span
        className={`flex justify-start items-start h-full px-2 py-0.5 w-fit max-w-[165px] rounded-sm gap-1 focus-visible:ring focus-visible:ring-ring/50 focus-visible:ring-[3px] transition-all font-medium break-all border border-white/15 bg-gray-300/5
        ${childText.length >= 16 ? "nowrap" : ""}`}
      >
        <p className="break-all text-white/80">{formatString(childText)}</p>
      </span>
      {/* Type label */}
      <div className="constraint-item text-2xs px-1.5 rounded-sm py-0.5 cursor-pointer bg-black/90 opacity-80 mt-1">
        {type}
      </div>
    </div>
  );
};

/**
 * Wrapper for constraint lists, providing vertical stacking.
 */
const PolicyConstraintsWrapper: FC<PolicyChildProps> = ({ children }) => {
  return <div className="flex flex-col gap-1">{children}</div>;
};

// =============================================================================
// EXPORTS
// =============================================================================

export {
  Policy,
  policyVariants,
  PolicyItemContainer,
  PolicyHeading,
  PolicyItem,
  PolicyItemKey,
  PolicyItemValue,
  PolicyConstraint,
  PolicyConstraintsContainer,
  PolicyConstraintsWrapper,
};
