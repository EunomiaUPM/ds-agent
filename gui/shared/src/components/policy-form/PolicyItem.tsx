/**
 * PolicyItem.tsx
 *
 * Single policy item (permission/obligation/prohibition) editor.
 * Allows editing the action and managing constraints within a policy rule.
 *
 * Part of the policy form component hierarchy:
 * PolicySection → PolicyItem → ConstraintItem
 *
 * @example
 * <PolicyItem
 *   item={{ action: "use", constraint: [...] }}
 *   typeLabel="permission"
 *   onRemove={() => removeItem()}
 *   onUpdateAction={(value) => updateAction(value)}
 *   onAddConstraint={() => addConstraint()}
 *   onRemoveConstraint={(idx) => removeConstraint(idx)}
 *   onUpdateConstraint={(idx, operand, value) => updateConstraint(idx, operand, value)}
 * />
 */

import React, { FC } from "react";
import { Plus, Trash } from "lucide-react";
import { Button } from "shared/src/components/ui/button";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "shared/src/components/ui/select";
import { odrlActions } from "shared/src/odrl_actions";
import { ConstraintItem } from "./ConstraintItem";
import { OperandType } from "shared/src/hooks/usePolicyForm";
import { OdrlPermission, OdrlProhibition, OdrlObligation } from "shared/src/data/orval/model";

// =============================================================================
// TYPES
// =============================================================================

/**
 * Props for the PolicyItem component.
 */
export interface PolicyItemProps {
  /** The ODRL permission/obligation/prohibition data */
  item: OdrlPermission | OdrlProhibition | OdrlObligation;

  /** Display label for the type (e.g., "permission", "obligation") */
  typeLabel: string;

  /** Callback to remove this policy item */
  onRemove: () => void;

  /** Callback when the action value changes */
  onUpdateAction: (value: string) => void;

  /** Callback to add a new constraint to this item */
  onAddConstraint: () => void;

  /** Callback to remove a constraint by index */
  onRemoveConstraint: (index: number) => void;

  /**
   * Callback to update a constraint's operand.
   * @param index - Constraint index within this item
   * @param operand - Which operand to update
   * @param value - New value
   */
  onUpdateConstraint: (index: number, operand: OperandType, value: string) => void;
}

// =============================================================================
// COMPONENT
// =============================================================================

/**
 * Editable policy item with action selector and constraint list.
 *
 * Features:
 * - Action selection from ODRL actions list
 * - Dynamic constraint list with add/remove
 * - Remove button for the entire item
 *
 * @param props - PolicyItem properties
 * @returns An editable policy item with constraints
 */
export const PolicyItem: FC<PolicyItemProps> = ({
  item,
  typeLabel,
  onRemove,
  onUpdateAction,
  onAddConstraint,
  onRemoveConstraint,
  onUpdateConstraint,
}) => {
  return (
    <div>
      <div className="policy-item-create">
        {/* Header with action label and remove button */}
        <div className="flex justify-between">
          <p className="mb-2">Action:</p>
          <Button variant="icon_destructive" size="xs" className="ml-4" onClick={onRemove}>
            <Trash className="mb-0.5" />
            Remove {typeLabel}
          </Button>
        </div>

        {/* Action selector */}
        <Select value={item.action} onValueChange={onUpdateAction}>
          <SelectTrigger className="w-[240px]">
            <SelectValue placeholder="Select action" />
          </SelectTrigger>
          <SelectContent>
            {odrlActions.map((odrlAction) => (
              <SelectItem key={odrlAction} value={odrlAction}>
                {odrlAction}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>

        {/* Constraints section */}
        <div className="h-6" />
        <p className="mb-2">Constraints:</p>

        {/* Constraint list */}
        {item.constraint?.map((constraint: any, j: number) => (
          <ConstraintItem
            key={j}
            constraint={constraint}
            onRemove={() => onRemoveConstraint(j)}
            onUpdate={(operand, value) => onUpdateConstraint(j, operand, value)}
          />
        ))}

        {/* Add constraint button */}
        <Button size="xs" variant="outline" className="mt-3" onClick={onAddConstraint}>
          <Plus />
          Add constraint
        </Button>
      </div>
    </div>
  );
};
