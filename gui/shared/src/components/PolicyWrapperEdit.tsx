/**
 * PolicyWrapperEdit.tsx
 *
 * Wrapper component for editing an existing ODRL policy.
 * Provides a complete UI for modifying permissions, obligations, and prohibitions
 * with their associated constraints.
 *
 * Uses a declarative pattern with `onChange` callback to notify parent
 * components of policy changes.
 *
 * @example
 * const [policy, setPolicy] = useState<OdrlInfo | null>(null);
 *
 * <PolicyWrapperEdit
 *   policy={existingPolicy}
 *   onChange={setPolicy}
 * />
 *
 * // Access current policy state via `policy` variable
 */

import React, { useEffect, useState, useCallback } from "react";
import Heading from "shared/src/components/ui/heading";
import { Badge } from "shared/src/components/ui/badge";
import { InfoList } from "shared/src/components/ui/info-list";
import {
  ComponentType,
  OperandType,
  PolicyEditSection,
} from "shared/src/components/policy/PolicyEditSection";
import { OdrlConstraint, OdrlInfo, OdrlOffer, OdrlPermission } from "shared/src/data/orval/model";

// =============================================================================
// TYPES
// =============================================================================

/**
 * Props for the PolicyWrapperEdit component.
 */
export interface PolicyWrapperEditProps {
  /** The ODRL policy to edit */
  policy: OdrlOffer;

  /**
   * Callback invoked when the policy changes.
   * Called with the updated policy state after any modification.
   */
  onChange?: (policy: OdrlInfo) => void;
}

// =============================================================================
// COMPONENT
// =============================================================================

/**
 * Full-featured policy editor with declarative state management.
 *
 * Features:
 * - Displays policy metadata (ID, type, target)
 * - Three collapsible sections for permission/obligation/prohibition
 * - Add/remove policy items and constraints
 * - Notifies parent of changes via `onChange` callback
 *
 * @param props - PolicyWrapperEdit properties
 * @returns A complete policy editing interface
 */
export const PolicyWrapperEdit = ({ policy, onChange }: PolicyWrapperEditProps) => {
  // ---------------------------------------------------------------------------
  // State
  // ---------------------------------------------------------------------------

  const [editedPolicy, setEditedPolicy] = useState<OdrlInfo>({
    obligation: [],
    permission: [],
    prohibition: [],
  });

  // ---------------------------------------------------------------------------
  // Effects
  // ---------------------------------------------------------------------------

  /** Initialize state from incoming policy prop */
  useEffect(() => {
    const initialPolicy: OdrlInfo = {
      obligation: policy.obligation || [],
      permission: policy.permission || [],
      prohibition: policy.prohibition || [],
    };
    setEditedPolicy(initialPolicy);

    const policyToEmit = { ...initialPolicy };
    if (policyToEmit.permission && policyToEmit.permission.length === 0) {
      policyToEmit.permission = undefined;
    }
    if (policyToEmit.obligation && policyToEmit.obligation.length === 0) {
      policyToEmit.obligation = undefined;
    }
    if (policyToEmit.prohibition && policyToEmit.prohibition.length === 0) {
      policyToEmit.prohibition = undefined;
    }

    // Also notify parent of initial state
    onChange?.(policyToEmit);
  }, [policy]);

  /** Notify parent when policy changes */
  const updatePolicyAndNotify = useCallback(
    (updater: (prev: OdrlInfo) => OdrlInfo) => {
      setEditedPolicy((prev) => {
        const newPolicy = updater(prev);
        const policyToEmit = { ...newPolicy };

        // Filter out empty arrays for onChange
        if (policyToEmit.permission && policyToEmit.permission.length === 0) {
          policyToEmit.permission = undefined;
        }
        if (policyToEmit.obligation && policyToEmit.obligation.length === 0) {
          policyToEmit.obligation = undefined;
        }
        if (policyToEmit.prohibition && policyToEmit.prohibition.length === 0) {
          policyToEmit.prohibition = undefined;
        }

        onChange?.(policyToEmit);
        return newPolicy;
      });
    },
    [onChange],
  );

  // ---------------------------------------------------------------------------
  // Event Handlers
  // ---------------------------------------------------------------------------

  /** Add a new policy component (permission/obligation/prohibition) */
  const handleAddComponent = useCallback(
    (componentType: ComponentType) => {
      const newComponent: OdrlPermission = {
        action: "",
        constraint: [],
      };
      updatePolicyAndNotify((prev) => ({
        ...prev,
        [componentType]: [...(prev[componentType] || []), newComponent],
      }));
    },
    [updatePolicyAndNotify],
  );

  /** Remove a policy component by index */
  const handleRemoveComponent = useCallback(
    (componentType: ComponentType, index: number) => {
      updatePolicyAndNotify((prev) => ({
        ...prev,
        [componentType]: (prev[componentType] || []).filter((_, i) => i !== index),
      }));
    },
    [updatePolicyAndNotify],
  );

  /** Add a constraint to a policy component */
  const handleAddConstraint = useCallback(
    (componentType: ComponentType, componentIndex: number) => {
      updatePolicyAndNotify((prev) => {
        const updated = [...(prev[componentType] || [])];
        updated[componentIndex] = {
          ...updated[componentIndex],
          constraint: [
            ...(updated[componentIndex].constraint || []),
            { leftOperand: "", operator: "", rightOperand: "" },
          ],
        };
        return { ...prev, [componentType]: updated };
      });
    },
    [updatePolicyAndNotify],
  );

  /** Remove a constraint from a policy component */
  const handleRemoveConstraint = useCallback(
    (componentType: ComponentType, componentIndex: number, constraintIndex: number) => {
      updatePolicyAndNotify((prev) => {
        const updated = [...(prev[componentType] || [])];
        updated[componentIndex] = {
          ...updated[componentIndex],
          constraint: (updated[componentIndex].constraint || []).filter(
            (_: OdrlConstraint, i: number) => i !== constraintIndex,
          ),
        };
        return { ...prev, [componentType]: updated };
      });
    },
    [updatePolicyAndNotify],
  );

  /** Update a policy component's action */
  const handleActionChange = useCallback(
    (componentType: ComponentType, componentIndex: number, value: string) => {
      updatePolicyAndNotify((prev) => {
        const updated = [...(prev[componentType] || [])];
        updated[componentIndex] = { ...updated[componentIndex], action: value };
        return { ...prev, [componentType]: updated };
      });
    },
    [updatePolicyAndNotify],
  );

  /** Update a constraint's operand value */
  const handleOperandChange = useCallback(
    (
      componentType: ComponentType,
      componentIndex: number,
      constraintIndex: number,
      operand: OperandType,
      value: string,
    ) => {
      updatePolicyAndNotify((prev) => {
        const updated = [...(prev[componentType] || [])];
        const constraints = [...(updated[componentIndex].constraint || [])];
        constraints[constraintIndex] = {
          ...constraints[constraintIndex],
          [operand]: value,
        };
        updated[componentIndex] = {
          ...updated[componentIndex],
          constraint: constraints,
        };
        return { ...prev, [componentType]: updated };
      });
    },
    [updatePolicyAndNotify],
  );

  // ---------------------------------------------------------------------------
  // Render
  // ---------------------------------------------------------------------------

  return (
    <div className="w-full">
      <div className="border border-white/30 bg-white/10 p-3 pt-2 rounded-md justify-start">
        {/* Policy Header */}
        <div className="flex mb-4">
          <Heading level="h5" className="flex gap-3">
            <div>Policy with ID</div>
            <Badge variant="info" className="h-6">
              {(policy["@id"] || "").slice(9, 29) + "[...]"}
            </Badge>
          </Heading>
        </div>

        {/* Policy Metadata */}
        <InfoList
          items={[
            { label: "Policy Target", value: policy["@type"] },
            { label: "Target", value: policy.target?.slice(9) },
          ]}
        />

        {/* ODRL Content Editor */}
        <div className="h-5" />
        <Heading level="h6">ODRL CONTENT</Heading>

        <div className="flex flex-col gap-2">
          <div className="flex flex-col gap-4">
            {/* Permission Section */}
            <PolicyEditSection
              type="permission"
              items={editedPolicy.permission || []}
              onAdd={handleAddComponent}
              onRemove={handleRemoveComponent}
              onActionChange={handleActionChange}
              onAddConstraint={handleAddConstraint}
              onRemoveConstraint={handleRemoveConstraint}
              onOperandChange={handleOperandChange}
            />

            {/* Obligation Section */}
            <PolicyEditSection
              type="obligation"
              items={editedPolicy.obligation || []}
              onAdd={handleAddComponent}
              onRemove={handleRemoveComponent}
              onActionChange={handleActionChange}
              onAddConstraint={handleAddConstraint}
              onRemoveConstraint={handleRemoveConstraint}
              onOperandChange={handleOperandChange}
            />

            {/* Prohibition Section */}
            <PolicyEditSection
              type="prohibition"
              items={editedPolicy.prohibition || []}
              onAdd={handleAddComponent}
              onRemove={handleRemoveComponent}
              onActionChange={handleActionChange}
              onAddConstraint={handleAddConstraint}
              onRemoveConstraint={handleRemoveConstraint}
              onOperandChange={handleOperandChange}
            />
          </div>
        </div>
      </div>
    </div>
  );
};
