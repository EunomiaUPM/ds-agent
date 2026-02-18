/**
 * PolicyEditSection.tsx
 *
 * Accordion section for editing ODRL policy rules inline.
 * Provides full CRUD functionality for permissions, prohibitions, and obligations.
 *
 * Unlike PolicyTemplateSection (for template-based editing), this component
 * allows direct editing of policy content with selects and inputs.
 *
 * @example
 * <PolicyEditSection
 *   type="permission"
 *   items={policy.permission}
 *   onAdd={(type) => addComponent(type)}
 *   onRemove={(type, idx) => removeComponent(type, idx)}
 *   onActionChange={(type, idx, value) => updateAction(type, idx, value)}
 *   onAddConstraint={(type, idx) => addConstraint(type, idx)}
 *   onRemoveConstraint={(type, idx, cIdx) => removeConstraint(type, idx, cIdx)}
 *   onOperandChange={(type, idx, cIdx, operand, value) => updateOperand(...)}
 * />
 */

import React from "react";
import {
  Accordion,
  AccordionContent,
  AccordionItem,
  AccordionTrigger,
} from "shared/src/components/ui/accordion";
import { Button } from "shared/src/components/ui/button";
import { Plus, Trash } from "lucide-react";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "shared/src/components/ui/select";
import { Input } from "shared/src/components/ui/input";
import { leftOperands, odrlActions, operators } from "shared/src/odrl_actions";

// =============================================================================
// TYPES
// =============================================================================

/** Policy component type identifier */
export type ComponentType = "permission" | "obligation" | "prohibition";

/** Constraint operand field identifier */
export type OperandType = "leftOperand" | "rightOperand" | "operator";

/**
 * Props for the PolicyEditSection component.
 */
export interface PolicyEditSectionProps {
  /** Type of policy section to render */
  type: ComponentType;

  /** List of policy items (permissions, obligations, or prohibitions) */
  items: any[];

  /** Callback when a new policy item is added */
  onAdd: (type: ComponentType) => void;

  /** Callback when a policy item is removed */
  onRemove: (type: ComponentType, index: number) => void;

  /** Callback when a policy item's action changes */
  onActionChange: (type: ComponentType, index: number, value: string) => void;

  /** Callback when a constraint is added to a policy item */
  onAddConstraint: (type: ComponentType, index: number) => void;

  /** Callback when a constraint is removed from a policy item */
  onRemoveConstraint: (type: ComponentType, index: number, constraintIndex: number) => void;

  /** Callback when a constraint's operand value changes */
  onOperandChange: (
    type: ComponentType,
    index: number,
    constraintIndex: number,
    operand: OperandType,
    value: string,
  ) => void;
}

// =============================================================================
// STYLES
// =============================================================================

/**
 * Color-coded styles for each policy type.
 */
const SECTION_STYLES = {
  permission: {
    accordionItem: "bg-success-500/10 border border-success-600/20",
    trigger: "bg-success-400/25",
    removeIcon: "text-foreground",
  },
  obligation: {
    accordionItem: "bg-warn-500/10 border border-warn-600/20",
    trigger: "bg-warn-400/25",
    removeIcon: "text-danger",
  },
  prohibition: {
    accordionItem: "bg-danger-500/10 border border-danger-600/20",
    trigger: "bg-danger-500/25",
    removeIcon: "text-foreground",
  },
} as const;

// =============================================================================
// COMPONENT
// =============================================================================

/**
 * Full-featured policy section editor.
 *
 * Provides:
 * - Add/remove policy items
 * - Action selection from ODRL actions
 * - Constraint management with operand selectors
 *
 * @param props - PolicyEditSection properties
 * @returns A collapsible section with inline policy editing
 */
export const PolicyEditSection: React.FC<PolicyEditSectionProps> = ({
  type,
  items,
  onAdd,
  onRemove,
  onActionChange,
  onAddConstraint,
  onRemoveConstraint,
  onOperandChange,
}) => {
  const currentStyle = SECTION_STYLES[type];

  return (
    <Accordion type="single" collapsible className="w-full">
      <AccordionItem value="item-1" className={currentStyle.accordionItem}>
        {/* Section Header */}
        <AccordionTrigger
          className={`text-white/70 flex ${currentStyle.trigger} uppercase overflow-hidden rounded-md data-[state=open]:rounded-b-none`}
        >
          <div className="flex items-center w-full">
            <p className="text-current">{type}</p>
          </div>
        </AccordionTrigger>

        {/* Section Content */}
        <AccordionContent className="relative">
          {/* Add new item button */}
          <Button
            type="button"
            className="border-b border-white/15"
            variant="outline"
            size="xs"
            onClick={() => onAdd(type)}
          >
            <Plus />
            Add {type}
          </Button>

          {/* Policy items list */}
          {items.map((item, i) => (
            <div key={`${type}-${i}-${item.action || "new"}`}>
              <div className="policy-item-create">
                {/* Item header with action label and remove button */}
                <div className="flex justify-between">
                  <p className="mb-2">Action:</p>
                  <Button
                    type="button"
                    variant="icon_destructive"
                    size="xs"
                    className="ml-4 border"
                    onClick={() => onRemove(type, i)}
                  >
                    <Trash className={`mb-0.5 ${currentStyle.removeIcon}`} />
                    Remove {type}
                  </Button>
                </div>

                {/* Action selector */}
                <Select
                  onValueChange={(value: string) => onActionChange(type, i, value)}
                  value={item.action}
                >
                  <SelectTrigger className="w-[240px]">
                    <SelectValue placeholder="Select action" />
                  </SelectTrigger>
                  <SelectContent>
                    {odrlActions.map((odrlAction) => (
                      <SelectItem value={odrlAction} key={odrlAction}>
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
                  <div className="flex flex-col gap-2" key={j}>
                    <div className="constraint-create flex gap-3 items-end">
                      {/* Left operand selector */}
                      <Select
                        onValueChange={(value: string) =>
                          onOperandChange(type, i, j, "leftOperand", value)
                        }
                        value={constraint.leftOperand}
                      >
                        <div className="flex flex-col">
                          <p className="text-xs text-gray-400 mb-1">Left Operand:</p>
                          <SelectTrigger className="w-[180px]">
                            <SelectValue placeholder="Select item" />
                          </SelectTrigger>
                          <SelectContent>
                            {leftOperands.map((leftOperand) => (
                              <SelectItem value={leftOperand} key={leftOperand}>
                                {leftOperand}
                              </SelectItem>
                            ))}
                          </SelectContent>
                        </div>
                      </Select>

                      {/* Operator selector */}
                      <Select
                        onValueChange={(value: string) =>
                          onOperandChange(type, i, j, "operator", value)
                        }
                        value={constraint.operator}
                      >
                        <div className="flex flex-col">
                          <p className="text-xs text-gray-400 mb-1">Operator:</p>
                          <SelectTrigger className="w-[140px]">
                            <SelectValue placeholder="Select operator" />
                          </SelectTrigger>
                          <SelectContent>
                            {operators.map((operator) => (
                              <SelectItem value={operator} key={operator}>
                                {operator}
                              </SelectItem>
                            ))}
                          </SelectContent>
                        </div>
                      </Select>

                      {/* Right operand input */}
                      <div className="flex flex-col">
                        <p className="text-xs text-gray-400 mb-1">Right Operand:</p>
                        <Input
                          placeholder="Type value"
                          value={constraint.rightOperand}
                          onChange={(ev) =>
                            onOperandChange(type, i, j, "rightOperand", ev.currentTarget.value)
                          }
                        />
                      </div>

                      {/* Remove constraint button */}
                      <div>
                        <Button
                          type="button"
                          variant="icon_destructive"
                          size="icon"
                          onClick={() => onRemoveConstraint(type, i, j)}
                        >
                          <Trash className="mb-0.5" />
                        </Button>
                      </div>
                    </div>
                  </div>
                ))}

                {/* Add constraint button */}
                <Button
                  type="button"
                  size="xs"
                  variant="outline"
                  className="mt-3"
                  onClick={() => onAddConstraint(type, i)}
                >
                  <Plus />
                  Add constraint
                </Button>
              </div>
            </div>
          ))}
        </AccordionContent>
      </AccordionItem>
    </Accordion>
  );
};
