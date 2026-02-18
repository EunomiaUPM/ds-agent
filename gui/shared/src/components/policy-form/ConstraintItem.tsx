/**
 * ConstraintItem.tsx
 *
 * Editable constraint row for ODRL policy constraints.
 * Provides select fields for left operand and operator, and a text input for right operand.
 *
 * Part of the policy form component hierarchy:
 * PolicySection → PolicyItem → ConstraintItem
 *
 * @example
 * <ConstraintItem
 *   constraint={{ leftOperand: "region", operator: "eq", rightOperand: "EU" }}
 *   onUpdate={(operand, value) => updateConstraint(operand, value)}
 *   onRemove={() => removeConstraint()}
 * />
 */

import React, { FC } from "react";
import { Trash } from "lucide-react";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "shared/src/components/ui/select";
import { Input } from "shared/src/components/ui/input";
import { Button } from "shared/src/components/ui/button";
import { leftOperands, operators } from "shared/src/odrl_actions";
import { OperandType } from "shared/src/hooks/usePolicyForm";
import { OdrlConstraint } from "shared/src/data/orval/model";

// =============================================================================
// TYPES
// =============================================================================

/**
 * Props for the ConstraintItem component.
 */
export interface ConstraintItemProps {
  /** The ODRL constraint data */
  constraint: OdrlConstraint;

  /**
   * Callback when any operand value changes.
   * @param operand - Which operand changed (leftOperand, operator, or rightOperand)
   * @param value - The new value
   */
  onUpdate: (operand: OperandType, value: string) => void;

  /** Callback to remove this constraint */
  onRemove: () => void;
}

// =============================================================================
// COMPONENT
// =============================================================================

/**
 * Single constraint editor with three fields:
 * - Left Operand (select from predefined list)
 * - Operator (select: eq, neq, lt, gt, etc.)
 * - Right Operand (free text input)
 *
 * @param props - ConstraintItem properties
 * @returns An editable constraint row
 */
export const ConstraintItem: FC<ConstraintItemProps> = ({ constraint, onUpdate, onRemove }) => {
  return (
    <div className="flex flex-col gap-2">
      <div className="constraint-create mb-2 flex gap-3 justify-end items-end">
        {/* Left Operand Select */}
        <Select
          value={constraint.leftOperand}
          onValueChange={(value: string) => onUpdate("leftOperand", value)}
        >
          <div className="flex flex-col">
            <p className="text-xs text-gray-400 mb-1">Left Operand:</p>
            <SelectTrigger className="w-[180px]">
              <SelectValue placeholder="Select item" />
            </SelectTrigger>
            <SelectContent>
              {leftOperands.map((leftOperand) => (
                <SelectItem key={leftOperand} value={leftOperand}>
                  {leftOperand}
                </SelectItem>
              ))}
            </SelectContent>
          </div>
        </Select>

        {/* Operator Select */}
        <Select
          value={constraint.operator}
          onValueChange={(value: string) => onUpdate("operator", value)}
        >
          <div className="flex flex-col">
            <p className="text-xs text-gray-400 mb-1">Operator:</p>
            <SelectTrigger className="w-[140px]">
              <SelectValue placeholder="Select operator" />
            </SelectTrigger>
            <SelectContent>
              {operators.map((operator) => (
                <SelectItem key={operator} value={operator}>
                  {operator}
                </SelectItem>
              ))}
            </SelectContent>
          </div>
        </Select>

        {/* Right Operand Input */}
        <div className="flex flex-col">
          <p className="text-xs text-gray-400 mb-1">Right Operand:</p>
          <Input
            placeholder="Type value"
            value={constraint.rightOperand as string}
            onChange={(ev) => onUpdate("rightOperand", ev.currentTarget.value)}
          />
        </div>

        {/* Remove Button */}
        <div>
          <Button variant="icon_destructive" size="icon" onClick={onRemove}>
            <Trash className="mb-0.5" />
          </Button>
        </div>
      </div>
    </div>
  );
};
