/**
 * PolicyTemplateWrapperEdit.tsx
 *
 * Form wrapper for creating policies from a template.
 * Allows users to fill in template placeholders (operand options) and
 * generates a complete ODRL policy on submission.
 *
 * The template defines the policy structure with placeholder variables
 * that are replaced with user-provided values on submit.
 *
 * @example
 * <PolicyTemplateWrapperEdit
 *   policyTemplate={selectedTemplate}
 *   onSubmit={async (odrlContent) => {
 *     await createPolicy(odrlContent);
 *   }}
 * />
 */

import React, { useEffect } from "react";
import { InfoList } from "shared/src/components/ui/info-list";
import Heading from "shared/src/components/ui/heading";
import { FormProvider, useForm } from "react-hook-form";
import { Button } from "shared/src/components/ui/button";
import { PolicyTemplateSection } from "shared/src/components/policy/PolicyTemplateSection";

// =============================================================================
// TYPES
// =============================================================================

/**
 * Dynamic form values where keys are operand names from the template.
 */
type TemplateFormValues = {
  [key: string]: string;
};

/**
 * Props for the PolicyTemplateWrapperEdit component.
 */
export interface PolicyTemplateWrapperEditProps {
  /** The policy template defining structure and operand options */
  policyTemplate: PolicyTemplate;

  /**
   * Callback when the form is submitted.
   * Receives the complete ODRL content with placeholders replaced.
   */
  onSubmit: (odrlContent: OdrlInfo) => Promise<void>;
}

// =============================================================================
// COMPONENT
// =============================================================================

/**
 * Template-based policy creation form.
 *
 * Features:
 * - Displays template metadata (title, description)
 * - Renders editable fields for template operand options
 * - Replaces placeholders with user values on submit
 * - Supports text, date, and select input types
 *
 * @param props - PolicyTemplateWrapperEdit properties
 * @returns A form for creating policies from templates
 */
export const PolicyTemplateWrapperEdit = ({
  policyTemplate,
  onSubmit,
}: PolicyTemplateWrapperEditProps) => {
  // ---------------------------------------------------------------------------
  // Form Setup
  // ---------------------------------------------------------------------------

  const form = useForm<TemplateFormValues>({
    defaultValues: {},
  });

  // ---------------------------------------------------------------------------
  // Effects
  // ---------------------------------------------------------------------------

  /**
   * Initialize form with default values from template operand options.
   * Re-runs when template changes.
   */
  useEffect(() => {
    const initialValues: TemplateFormValues = {};

    Object.entries(policyTemplate.operand_options).forEach(([key, value]) => {
      if (key && value.defaultValue !== undefined) {
        initialValues[key] = value.defaultValue;
      }
    });

    form.reset(initialValues);
  }, [policyTemplate, form]);

  // ---------------------------------------------------------------------------
  // Handlers
  // ---------------------------------------------------------------------------

  /**
   * Handles form submission by replacing template placeholders with form values.
   * Uses string replacement to substitute placeholder variables in the template content.
   */
  const handleSubmit = async (formData: TemplateFormValues) => {
    // Deep clone the template content to avoid mutation
    let odrlContent = JSON.parse(JSON.stringify(policyTemplate.content));

    // Replace each placeholder with its form value
    for (const key in formData) {
      if (formData.hasOwnProperty(key)) {
        const value = formData[key];
        let contentString = JSON.stringify(odrlContent);

        // Escape special regex characters in the key (like $)
        const escapedKey = key.replace(/[$]/g, "\\$&");
        contentString = contentString.replace(new RegExp(escapedKey, "g"), value);

        odrlContent = JSON.parse(contentString);
      }
    }

    await onSubmit(odrlContent);
    form.reset();
  };

  // ---------------------------------------------------------------------------
  // Render
  // ---------------------------------------------------------------------------

  return (
    <div>
      <FormProvider {...form}>
        <form onSubmit={form.handleSubmit(handleSubmit)} className="space-y-4">
          {/* Template Card */}
          <div className="border border-white/30 bg-white/10 px-4 py-2 pb-4 rounded-md justify-start max-h-[80vh] overflow-y-auto">
            {/* Template Header */}
            <div className="flex mb-4">
              <Heading level="h5" className="flex gap-3">
                <span className="font-light">Policy template:</span>
                {policyTemplate.title}
              </Heading>
            </div>

            {/* Template Description */}
            <InfoList items={[{ label: "Description", value: policyTemplate.description }]} />

            {/* ODRL Content Sections */}
            <div className="min-h-5" />
            <Heading level="h6">ODRL CONTENT</Heading>
            <p className="mb-2">Edit the constraint values</p>

            <div className="flex flex-col gap-2">
              <div className="flex flex-col gap-3">
                {/* Permission Section */}
                <PolicyTemplateSection
                  type="permission"
                  items={policyTemplate.content.permission}
                  operandOptions={policyTemplate.operand_options}
                  control={form.control}
                />

                {/* Obligation Section */}
                <PolicyTemplateSection
                  type="obligation"
                  items={policyTemplate.content.obligation}
                  operandOptions={policyTemplate.operand_options}
                  control={form.control}
                />

                {/* Prohibition Section */}
                <PolicyTemplateSection
                  type="prohibition"
                  items={policyTemplate.content.prohibition}
                  operandOptions={policyTemplate.operand_options}
                  control={form.control}
                />
              </div>
            </div>
          </div>

          {/* Submit Button */}
          <Button className="w-fit mt-4" type="submit">
            Create policy
          </Button>
        </form>
      </FormProvider>
    </div>
  );
};
