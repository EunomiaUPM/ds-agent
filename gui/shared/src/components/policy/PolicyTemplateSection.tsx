/**
 * PolicyTemplateSection.tsx
 *
 * Accordion section for editing policy templates.
 * Renders ODRL rules from a template and allows editing constraint values
 * based on the operand options defined in the template.
 *
 * Unlike PolicyEditSection (for direct editing), this component uses
 * pre-defined template options to generate form fields.
 *
 * @example
 * <PolicyTemplateSection
 *   type="permission"
 *   items={template.content.permission}
 *   operandOptions={template.operand_options}
 *   control={form.control}
 * />
 */

import React from "react";
import {
  Accordion,
  AccordionContent,
  AccordionItem,
  AccordionTrigger,
} from "shared/src/components/ui/accordion";
import { FormField } from "shared/src/components/ui/form";
import { Input } from "shared/src/components/ui/input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "shared/src/components/ui/select";
import { Control, ControllerRenderProps } from "react-hook-form";

// =============================================================================
// TYPES
// =============================================================================

/**
 * Props for the PolicyTemplateSection component.
 */
export interface PolicyTemplateSectionProps {
  /** Type of policy section (permission, obligation, prohibition) */
  type: "permission" | "obligation" | "prohibition";

  /** Policy items from the template content */
  items: any[] | undefined;

  /** Operand options from the template defining available form inputs */
  operandOptions: any;

  /** React Hook Form control for form state management */
  control: Control<any>;
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
  },
  obligation: {
    accordionItem: "bg-warn-500/10 border border-warn-600/20",
    trigger: "bg-warn-400/25",
  },
  prohibition: {
    accordionItem: "bg-danger-500/10 border border-danger-600/20",
    trigger: "bg-danger-500/25",
  },
} as const;

// =============================================================================
// COMPONENT
// =============================================================================

/**
 * Template-based policy section editor.
 *
 * Renders policy rules as read-only with editable constraint values
 * based on template operand options (text, date, or select).
 *
 * @param props - PolicyTemplateSection properties
 * @returns A collapsible section with template-based constraint inputs
 */
export const PolicyTemplateSection: React.FC<PolicyTemplateSectionProps> = ({
  type,
  items,
  operandOptions,
  control,
}) => {
  const currentStyle = SECTION_STYLES[type];

  // ---------------------------------------------------------------------------
  // Render Helpers
  // ---------------------------------------------------------------------------

  /**
   * Renders the appropriate form input based on the operand's form type.
   * Supports: text, date, and select inputs.
   */
  const renderOperandInput = (
    field: ControllerRenderProps,
    operand: string,
  ): React.ReactElement => {
    const options = operandOptions[operand];
    if (!options) return <></>;

    const { formType, defaultValue, options: selectOptions } = options;

    switch (formType) {
      case "date":
      case "text":
        return <Input type={formType} defaultValue={defaultValue} {...field} />;

      case "select":
        return (
          <Select defaultValue={defaultValue} onValueChange={field.onChange}>
            <SelectTrigger className="min-w-[200px] w-fit">
              <SelectValue placeholder="Select an option" />
            </SelectTrigger>
            <SelectContent>
              {selectOptions?.map((opt: any) => (
                <SelectItem className="text-white/70 min-w-fit" key={opt.value} value={opt.value}>
                  {/* Display localized label if available, fallback to value */}
                  {opt.label?.find((l: any) => l["@language"] === "en")?.["@value"]}
                  {opt.value}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        );

      default:
        return <></>;
    }
  };

  /**
   * Wraps the operand input in a FormField for react-hook-form integration.
   */
  const renderFormField = (operand: string) => (
    <FormField
      render={({ field }) => renderOperandInput(field, operand)}
      name={operand}
      key={operand}
      control={control}
    />
  );

  // ---------------------------------------------------------------------------
  // Render
  // ---------------------------------------------------------------------------

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
          {/* Empty state */}
          {(!items || items.length === 0) && <p>No policy defined</p>}

          {/* Policy items from template */}
          {(items || []).map((item, i) => (
            <div key={i} className="border-b border-white/20 last:border-0 first:mt-0 mt-3">
              <div className="policy-item-template">
                {/* Action display (read-only) */}
                <div className="flex gap-3">
                  <p className="mb-2 opacity-80 font-bold">Action:</p>
                  <p>{item.action?.toUpperCase()}</p>
                </div>

                <div className="h-2" />

                {/* Constraints section */}
                <p className="mb-1 opacity-80 font-bold">Constraints:</p>

                {item.constraint?.map((constraint: any, j: number) => (
                  <div className="constraint-template-group" key={j}>
                    <div className="constraint-create flex gap-3">
                      {/* Left operand (read-only) */}
                      <div className="flex flex-col">
                        <p className="text-xs text-gray-500 mb-1">Left Operand:</p>
                        <p className="key-policy-template">{constraint.leftOperand}</p>
                      </div>

                      {/* Operator (read-only) */}
                      <div className="flex flex-col">
                        <p className="text-xs text-gray-500 mb-1">Operator:</p>
                        <p className="key-policy-template">{constraint.operator}</p>
                      </div>

                      {/* Right operand (editable via form field) */}
                      <div className="flex flex-col">
                        <p className="text-xs text-gray-500 mb-1">Right Operand:</p>
                        <div>{renderFormField(constraint.rightOperand)}</div>
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          ))}
        </AccordionContent>
      </AccordionItem>
    </Accordion>
  );
};
