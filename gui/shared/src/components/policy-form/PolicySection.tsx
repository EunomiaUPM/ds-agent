/**
 * PolicySection.tsx
 *
 * Accordion section for a policy type (permission/obligation/prohibition).
 * Contains a list of PolicyItem components and an "Add" button.
 *
 * Part of the policy form component hierarchy:
 * PolicySection - PolicyItem - ConstraintItem
 *
 * @example
 * <PolicySection
 *   type="permission"
 *   items={permissions}
 *   accordionValue="permission-section"
 *   accordionStyles="bg-success-500/10 border-success-600/20"
 *   triggerStyles="bg-success-400/25"
 *   onAdd={() => addPermission()}
 *   onRemove={(idx) => removePermission(idx)}
 *   onUpdateAction={(idx, value) => updateAction(idx, value)}
 *   onAddConstraint={(idx) => addConstraint(idx)}
 *   onRemoveConstraint={(itemIdx, constraintIdx) => removeConstraint(itemIdx, constraintIdx)}
 *   onUpdateConstraint={(itemIdx, constraintIdx, operand, value) => updateConstraint(...)}
 * />
 */

import React, { FC } from "react";
import {
  AccordionContent,
  AccordionItem,
  AccordionTrigger,
} from "shared/src/components/ui/accordion";
import { Button } from "shared/src/components/ui/button";
import { Plus } from "lucide-react";
import { PolicyItem } from "./PolicyItem";
import { ComponentType, OperandType } from "shared/src/hooks/usePolicyForm";
import { OdrlPermission, OdrlProhibition, OdrlObligation } from "shared/src/data/orval/model";

// =============================================================================
// TYPES
// =============================================================================

/**
 * Props for the PolicySection component.
 */
export interface PolicySectionProps {
  /** Type of policy section */
  type: ComponentType;

  /** List of policy items to display */
  items: (OdrlPermission | OdrlProhibition | OdrlObligation)[];

  /** Unique value for accordion state */
  accordionValue: string;

  /** CSS classes for the accordion item container */
  accordionStyles: string;

  /** CSS classes for the accordion trigger */
  triggerStyles: string;

  /** Callback to add a new policy item */
  onAdd: () => void;

  /** Callback to remove a policy item by index */
  onRemove: (index: number) => void;

  /** Callback to update a policy item's action */
  onUpdateAction: (index: number, value: string) => void;

  /** Callback to add a constraint to a policy item */
  onAddConstraint: (index: number) => void;

  /** Callback to remove a constraint from a policy item */
  onRemoveConstraint: (itemIndex: number, constraintIndex: number) => void;

  /** Callback to update a constraint's operand */
  onUpdateConstraint: (
    itemIndex: number,
    constraintIndex: number,
    operand: OperandType,
    value: string,
  ) => void;
}

// =============================================================================
// COMPONENT
// =============================================================================

/**
 * Collapsible section for editing policy items of a specific type.
 *
 * Features:
 * - Collapsible accordion UI
 * - Color-coded by policy type
 * - Add button for new items
 * - Delegates item editing to PolicyItem components
 *
 * @param props - PolicySection properties
 * @returns An accordion section with policy items
 */
export const PolicySection: FC<PolicySectionProps> = ({
  type,
  items,
  accordionValue,
  accordionStyles,
  triggerStyles,
  onAdd,
  onRemove,
  onUpdateAction,
  onAddConstraint,
  onRemoveConstraint,
  onUpdateConstraint,
}) => {
  return (
    <AccordionItem value={accordionValue} className={accordionStyles}>
      {/* Accordion header with type label */}
      <AccordionTrigger
        className={`${triggerStyles} text-white/70 flex uppercase overflow-hidden rounded-md data-[state=open]:rounded-b-none`}
      >
        <div className="flex items-center w-full">
          <p className="text-current">{type}</p>
        </div>
      </AccordionTrigger>

      {/* Accordion content with items list */}
      <AccordionContent className="relative">
        {/* Add new item button */}
        <Button className="border-b border-white/15" variant="outline" size="xs" onClick={onAdd}>
          <Plus />
          Add {type}
        </Button>

        {/* Policy items list */}
        {items.map((item, i) => (
          <PolicyItem
            key={i}
            item={item}
            typeLabel={type}
            onRemove={() => onRemove(i)}
            onUpdateAction={(value) => onUpdateAction(i, value)}
            onAddConstraint={() => onAddConstraint(i)}
            onRemoveConstraint={(constraintIndex) => onRemoveConstraint(i, constraintIndex)}
            onUpdateConstraint={(constraintIndex, operand, value) =>
              onUpdateConstraint(i, constraintIndex, operand, value)
            }
          />
        ))}
      </AccordionContent>
    </AccordionItem>
  );
};
