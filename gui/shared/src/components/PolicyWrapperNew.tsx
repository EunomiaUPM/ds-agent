import React, { FC, useState } from "react";
import { Accordion } from "shared/src/components/ui/accordion";
import { Button } from "shared/src/components/ui/button";
import { usePolicyForm } from "shared/src/hooks/usePolicyForm";
import { PolicySection } from "./policy-form/PolicySection";
import { Textarea } from "shared/src/components/ui/textarea";
import { OdrlPolicyInfo } from "shared/src/data/orval/model/odrlPolicyInfo";

/**
 * Wrapper component for creating a new policy.
 */
export const PolicyWrapperNew: FC<{
  onSubmit: (odrlContent: OdrlPolicyInfo, description?: string) => void | Promise<void>;
}> = ({ onSubmit }) => {
  const {
    policy,
    addComponent,
    removeComponent,
    addConstraint,
    removeConstraint,
    updateField,
    updateOperand,
    resetPolicy,
  } = usePolicyForm();

  const [description, setDescription] = useState<string>("");

  const submitHandler = () => {
    onSubmit(policy, description || undefined);
    resetPolicy();
    setDescription("");
  };

  return (
    <div className="h-screen flex flex-col">
      <div className="flex flex-col gap-3 overflow-y-scroll h-[calc(100vh-180px)]  p-8 ">
        <div className="flex flex-col gap-1">
          <label className="text-xs font-medium text-muted-foreground uppercase tracking-wider">
            Description
          </label>
          <Textarea
            placeholder="Optional policy description"
            value={description}
            onChange={(e) => setDescription(e.target.value)}
            className="resize-none"
            rows={2}
          />
        </div>
        <Accordion type="single" collapsible className="w-full">
          <PolicySection
            type="permission"
            items={policy.permission || []}
            accordionValue="item-1"
            accordionStyles="bg-success-500/10 border border-success-600/20"
            triggerStyles="bg-success-400/25"
            onAdd={() => addComponent("permission")}
            onRemove={(index) => removeComponent("permission", index)}
            onUpdateAction={(index, value) => updateField("permission", index, value)}
            onAddConstraint={(index) => addConstraint("permission", index)}
            onRemoveConstraint={(itemIndex, constraintIndex) =>
              removeConstraint("permission", itemIndex, constraintIndex)
            }
            onUpdateConstraint={(itemIndex, constraintIndex, operand, value) =>
              updateOperand("permission", itemIndex, constraintIndex, operand, value)
            }
          />
        </Accordion>
        <Accordion type="single" collapsible className="w-full">
          <PolicySection
            type="obligation"
            items={policy.obligation || []}
            accordionValue="item-1"
            accordionStyles="bg-warn-500/10 border border-warn-600/20"
            triggerStyles="bg-warn-400/25"
            onAdd={() => addComponent("obligation")}
            onRemove={(index) => removeComponent("obligation", index)}
            onUpdateAction={(index, value) => updateField("obligation", index, value)}
            onAddConstraint={(index) => addConstraint("obligation", index)}
            onRemoveConstraint={(itemIndex, constraintIndex) =>
              removeConstraint("obligation", itemIndex, constraintIndex)
            }
            onUpdateConstraint={(itemIndex, constraintIndex, operand, value) =>
              updateOperand("obligation", itemIndex, constraintIndex, operand, value)
            }
          />
        </Accordion>
        <Accordion type="single" collapsible className="w-full">
          <PolicySection
            type="prohibition"
            items={policy.prohibition || []}
            accordionValue="item-1"
            accordionStyles="bg-danger-500/10 border border-danger-600/20 mb-24"
            triggerStyles="bg-danger-500/25"
            onAdd={() => addComponent("prohibition")}
            onRemove={(index) => removeComponent("prohibition", index)}
            onUpdateAction={(index, value) => updateField("prohibition", index, value)}
            onAddConstraint={(index) => addConstraint("prohibition", index)}
            onRemoveConstraint={(itemIndex, constraintIndex) =>
              removeConstraint("prohibition", itemIndex, constraintIndex)
            }
            onUpdateConstraint={(itemIndex, constraintIndex, operand, value) =>
              updateOperand("prohibition", itemIndex, constraintIndex, operand, value)
            }
          />
        </Accordion>
      </div>
      <div className="h-24  bottom-0 bg-background border-t border-white/30 w-full p-6">
        <Button onClick={() => submitHandler()}>Add policy</Button>
      </div>
    </div>
  );
};
