/**
 * BaseProcessDialog.tsx
 *
 * A reusable base dialog component that eliminates code duplication across
 * process-related dialogs (Contract Negotiation, Transfer Process, Business).
 *
 * This component provides:
 * - Consistent dialog structure (Header, Description, Body, Footer)
 * - Built-in form handling with react-hook-form
 * - Automatic InfoList rendering from process data
 * - Configurable submit button variants and labels
 * - Optional custom content slots for extended functionality
 *
 * @example
 * // Simple usage for a termination dialog
 * <BaseProcessDialog
 *   title="Termination Dialog"
 *   description="You are about to terminate this process."
 *   process={process}
 *   infoItems={mapCNProcessToInfoItems(process)}
 *   submitLabel="Terminate"
 *   submitVariant="destructive"
 *   onSubmit={handleSubmit}
 * />
 */

import React, { ReactNode, useEffect, MutableRefObject } from "react";
import {
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "../../ui/dialog";
import { Button } from "../../ui/button";
import { Form } from "../../ui/form";
import { InfoList, InfoItemProps } from "../../ui/info-list";
import { useForm, UseFormReturn, FieldValues, DefaultValues } from "react-hook-form";

// =============================================================================
// TYPES
// =============================================================================

export type ButtonVariant = "default" | "destructive" | "outline" | "secondary" | "ghost" | "link";

export interface BaseProcessDialogProps<TFormValues extends FieldValues = FieldValues> {
  /** Dialog title displayed in the header */
  title: string;

  /** Description text below the title */
  description: string | ReactNode;

  /** Info items to display in the InfoList component */
  infoItems: InfoItemProps[];

  /** Label for the submit button */
  submitLabel: string;

  /** Visual variant for the submit button */
  submitVariant?: ButtonVariant;

  /** Callback when form is submitted */
  onSubmit: (data: TFormValues, form: UseFormReturn<TFormValues>) => void | Promise<void>;

  /** Optional default values for the form */
  defaultValues?: DefaultValues<TFormValues>;

  /** Optional content to render before the InfoList */
  beforeInfoContent?: ReactNode;

  /** Optional content to render after the InfoList */
  afterInfoContent?: ReactNode;

  /** Optional form fields to render (for dialogs with input fields) */
  formFields?: ReactNode;

  /** Custom className for the DialogContent */
  contentClassName?: string;

  /** Whether to use scrollable body layout */
  scrollable?: boolean;

  /** Label for the cancel button (defaults to "Cancel") */
  cancelLabel?: string;

  /** Whether to hide the info list entirely */
  hideInfoList?: boolean;

  /** Whether to disable the submit button */
  disabledSubmit?: boolean;

  /** Optional external form instance to use (for shared state with custom fields) */
  form?: UseFormReturn<TFormValues>;
}

// =============================================================================
// COMPONENT
// =============================================================================

/**
 * Base dialog component for process-related actions.
 *
 * Provides a consistent structure for dialogs that:
 * 1. Display process information via InfoList
 * 2. Handle form submission
 * 3. Have Cancel/Submit action buttons
 */
export function BaseProcessDialog<TFormValues extends FieldValues = FieldValues>({
  title,
  description,
  infoItems,
  submitLabel,
  submitVariant = "default",
  onSubmit,
  defaultValues,
  beforeInfoContent,
  afterInfoContent,
  formFields,
  contentClassName = "w-[70dvw] sm:max-w-fit",
  scrollable = false,
  cancelLabel = "Cancel",
  hideInfoList = false,
  disabledSubmit = false,
  form: externalForm,
}: BaseProcessDialogProps<TFormValues>) {
  // Initialize form with optional default values or use provided external form
  const internalForm = useForm<TFormValues>({
    defaultValues: defaultValues as DefaultValues<TFormValues>,
  });

  const form = externalForm || internalForm;

  // Handle form submission
  const handleSubmit = async (data: TFormValues) => {
    await onSubmit(data, form);
  };

  // Render description as string or ReactNode
  const renderDescription = () => {
    if (typeof description === "string") {
      return <span className="max-w-full flex flex-wrap">{description}</span>;
    }
    return description;
  };

  // Filter out undefined info items
  const filteredInfoItems = infoItems.filter(
    (item): item is InfoItemProps => item !== undefined && item.value !== undefined,
  );

  // Scrollable layout for dialogs with lots of content
  if (scrollable) {
    return (
      <DialogContent className={`p-0 ${contentClassName}`}>
        <Form {...form}>
          <form
            onSubmit={form.handleSubmit(handleSubmit)}
            className="space-y-6 flex flex-col h-fit max-h-[90dvh]"
          >
            <DialogHeader className="px-6 pt-6">
              <DialogTitle>{title}</DialogTitle>
              <DialogDescription className="max-w-full flex flex-wrap">
                {renderDescription()}
              </DialogDescription>
            </DialogHeader>

            <div className="overflow-y-scroll px-6">
              {beforeInfoContent}
              {!hideInfoList && filteredInfoItems.length > 0 && (
                <InfoList items={filteredInfoItems} />
              )}
              {formFields}
              {afterInfoContent}
            </div>

            <DialogFooter className="[&>*]:w-full p-6 pt-0">
              <DialogClose asChild>
                <Button variant="ghost" type="reset">
                  {cancelLabel}
                </Button>
              </DialogClose>
              <Button type="submit" variant={submitVariant} isLoading={form.formState.isSubmitting} disabled={disabledSubmit}>
                {submitLabel}
              </Button>
            </DialogFooter>
          </form>
        </Form>
      </DialogContent>
    );
  }

  // Standard layout for simpler dialogs
  return (
    <DialogContent className={contentClassName}>
      <DialogHeader>
        <DialogTitle>{title}</DialogTitle>
        <DialogDescription className="max-w-full flex flex-wrap">
          {renderDescription()}
        </DialogDescription>
      </DialogHeader>

      {beforeInfoContent}
      {!hideInfoList && filteredInfoItems.length > 0 && <InfoList items={filteredInfoItems} />}

      <Form {...form}>
        <form onSubmit={form.handleSubmit(handleSubmit)} className="space-y-6">
          {formFields}
          {afterInfoContent}
          <DialogFooter className="[&>*]:w-full">
            <DialogClose asChild>
              <Button variant="ghost" type="reset">
                {cancelLabel}
              </Button>
            </DialogClose>
            <Button
              type="submit"
              variant={submitVariant}
              isLoading={form.formState.isSubmitting}
              disabled={disabledSubmit}
            >
              {submitLabel}
            </Button>
          </DialogFooter>
        </form>
      </Form>
    </DialogContent>
  );
}

export default BaseProcessDialog;
