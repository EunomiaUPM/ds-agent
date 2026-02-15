/**
 * PolicyComponent.tsx
 *
 * Recursive component for displaying ODRL policy rules.
 * Renders permission, prohibition, or obligation items with their
 * associated actions and constraints.
 *
 * The component handles the nested structure of ODRL policies,
 * displaying each rule's action and any associated constraints
 * (left operand, operator, right operand).
 *
 * @example
 * // Display permissions for a policy
 * <PolicyComponent
 *   policyItem={policy.permission}
 *   variant="permission"
 * />
 *
 * @example
 * // Display prohibitions
 * <PolicyComponent
 *   policyItem={policy.prohibition}
 *   variant="prohibition"
 * />
 */

import React, { FC } from "react";
import {
  Policy,
  PolicyConstraint,
  PolicyConstraintsContainer,
  PolicyConstraintsWrapper,
  PolicyItem,
  PolicyItemContainer,
  PolicyItemKey,
  PolicyItemValue,
  PolicyVariants,
} from "shared/src/components/ui/policy";
import { OdrlPermission, OdrlProhibition, OdrlObligation } from "shared/src/data/orval/model";

// =============================================================================
// TYPES
// =============================================================================

/**
 * Props for the PolicyComponent.
 */
export interface PolicyComponentProps {
  /**
   * Array of ODRL permission/prohibition/obligation items to display.
   * Each item contains an action and optional constraints.
   */
  policyItem: (OdrlPermission | OdrlProhibition | OdrlObligation)[] | null | undefined;

  /**
   * Visual variant determining the component's color scheme.
   * - "permission": Green-ish styling
   * - "prohibition": Red-ish styling
   * - "obligation": Blue-ish styling
   */
  variant: PolicyVariants;
}

// =============================================================================
// COMPONENT
// =============================================================================

/**
 * Displays ODRL policy rules with actions and constraints.
 *
 * Renders a list of policy items, each showing:
 * - The action (e.g., "use", "distribute", "modify")
 * - Constraints as operand-operator-operand triplets
 *
 * Empty or null policy items display a placeholder message.
 *
 * @param props - PolicyComponent properties
 * @returns Styled policy rule display
 */
const PolicyComponent: FC<PolicyComponentProps> = ({ policyItem, variant }) => {
  // Empty state: show "No {variant}s" message
  const isEmpty = policyItem?.length === 0 || policyItem == null;

  return (
    <Policy className="" variant={variant}>
      {isEmpty ? (
        <p className="text-xs mt-0.5 text-white/70">No {variant}s</p>
      ) : (
        <div className="flex flex-col">
          {/* Iterate over each policy item */}
          {policyItem?.map((item, i: number) => (
            <PolicyItemContainer key={i}>
              {/* Action row */}
              <PolicyItem>
                <PolicyItemKey>action:</PolicyItemKey>
                <PolicyItemValue>{item.action}</PolicyItemValue>
              </PolicyItem>

              {/* Constraints row */}
              <PolicyItem>
                <PolicyItemKey>constraints:</PolicyItemKey>
                <PolicyConstraintsWrapper>
                  {/* Empty constraints */}
                  {item.constraint == null || item.constraint.length === 0 ? (
                    <p className="text-xs mt-0.5">No constraints</p>
                  ) : (
                    <>
                      {/* Render each constraint as a triplet */}
                      {item.constraint.map((constr: any, j: number) => (
                        <PolicyConstraintsContainer key={j}>
                          <PolicyConstraint type="leftOperand">
                            {JSON.stringify(constr.leftOperand)}
                          </PolicyConstraint>
                          <PolicyConstraint type="operator">
                            {JSON.stringify(constr.operator)}
                          </PolicyConstraint>
                          <PolicyConstraint type="rightOperand">
                            {JSON.stringify(constr.rightOperand)}
                          </PolicyConstraint>
                        </PolicyConstraintsContainer>
                      ))}
                    </>
                  )}
                </PolicyConstraintsWrapper>
              </PolicyItem>
            </PolicyItemContainer>
          ))}
        </div>
      )}
    </Policy>
  );
};

export default PolicyComponent;
