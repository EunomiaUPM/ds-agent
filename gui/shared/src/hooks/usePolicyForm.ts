import { useState } from "react";
import { OdrlInfo, OdrlPermission } from "../data/orval/model";

export type ComponentType = "permission" | "obligation" | "prohibition";
export type OperandType = "leftOperand" | "rightOperand" | "operator";

/*
  Custom React hook to manage the state of an ODRL policy form.
  It provides functions to add/remove components and constraints,
  as well as to update field values and reset the policy.
*/
export const usePolicyForm = (initialPolicy?: OdrlInfo) => {
  const [newPolicy, setNewPolicy] = useState<OdrlInfo>(
    initialPolicy || {
      obligation: [],
      permission: [],
      prohibition: [],
    },
  );

  // Handlers for managing policy components and constraints
  // Add a new component (permission, obligation, prohibition)
  const addComponentHandler = (componentType: ComponentType) => {
    const newComponent: OdrlPermission = {
      action: "",
      constraint: [],
    };
    const _newPolicy = { ...newPolicy };
    
    const targetArray = _newPolicy[componentType];
    if (targetArray) {
        (targetArray as OdrlPermission[]).push(newComponent);
    } else {
        _newPolicy[componentType] = [newComponent];
    }
    setNewPolicy(_newPolicy);
  };

  // Remove a component by index
  const removeComponentHandler = (componentType: ComponentType, index: number) => {
    const _newPolicy = { ...newPolicy };
    if (_newPolicy[componentType]) {
        // @ts-ignore
        _newPolicy[componentType]!.splice(index, 1);
    }
    setNewPolicy(_newPolicy);
  };

  // Add a new constraint to a specific component
  const addConstraintHandler = (componentType: ComponentType, componentIndex: number) => {
    const _newPolicy = { ...newPolicy };
    if (_newPolicy[componentType] && _newPolicy[componentType]![componentIndex]) {
        if (!_newPolicy[componentType]![componentIndex].constraint) {
            _newPolicy[componentType]![componentIndex].constraint = [];
        }
        _newPolicy[componentType]![componentIndex].constraint!.push({
            leftOperand: "",
            operator: "",
            rightOperand: "",
        });
    }
    setNewPolicy(_newPolicy);
  };

  // Remove a constraint from a specific component
  const removeConstraintHandler = (
    componentType: ComponentType,
    componentIndex: number,
    constraintIndex: number,
  ) => {
    const _newPolicy = { ...newPolicy };
    if (_newPolicy[componentType] && _newPolicy[componentType]![componentIndex] && _newPolicy[componentType]![componentIndex].constraint) {
        _newPolicy[componentType]![componentIndex].constraint!.splice(constraintIndex, 1);
    }
    setNewPolicy(_newPolicy);
  };

  // Update field values for action and constraints
  const fieldValueChangeHandler = (
    componentType: ComponentType,
    componentIndex: number,
    value: string,
  ) => {
    const _newPolicy = { ...newPolicy };
    if (_newPolicy[componentType] && _newPolicy[componentType]![componentIndex]) {
        // @ts-ignore
        _newPolicy[componentType]![componentIndex].action = value;
    }
    setNewPolicy(_newPolicy);
  };

  // Update operand values for constraints
  const operandValueChangeHandler = (
    componentType: ComponentType,
    componentIndex: number,
    constraintIndex: number,
    operand: OperandType,
    value: string,
  ) => {
    const _newPolicy = { ...newPolicy };
    const targetArray = _newPolicy[componentType];
    if(targetArray && targetArray[componentIndex] && targetArray[componentIndex].constraint) {
        const constraint = targetArray[componentIndex].constraint![constraintIndex];
        if (constraint) {
            // @ts-ignore - constraint type might be strict but we are editing
            constraint[operand] = value;
        }
    }
    setNewPolicy(_newPolicy);
  };

  // Reset the policy to an empty state
  const resetPolicy = () => {
    setNewPolicy({
      permission: [],
      obligation: [],
      prohibition: [],
    });
  };

  return {
    policy: newPolicy,
    addComponent: addComponentHandler,
    removeComponent: removeComponentHandler,
    addConstraint: addConstraintHandler,
    removeConstraint: removeConstraintHandler,
    updateField: fieldValueChangeHandler,
    updateOperand: operandValueChangeHandler,
    resetPolicy,
  };
};
