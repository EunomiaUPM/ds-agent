/**
 * ProcessActionDialog.tsx
 *
 * Generic dialog wrapper for process action buttons.
 * Combines a trigger button with a dialog component that receives the process data.
 *
 * This component is used by action components (ContractNegotiationActions,
 * TransferProcessActions, etc.) to create consistent action buttons that
 * open their respective dialog components.
 *
 * @example
 * <ProcessActionDialog
 *   label="Terminate"
 *   variant="destructive"
 *   DialogComponent={ContractNegotiationTerminationDialog}
 *   process={currentProcess}
 * />
 */

import React, { FC, useState } from "react";
import { Button, ButtonSizes } from "shared/src/components/ui/button";
import { Dialog, DialogTrigger } from "shared/src/components/ui/dialog";

// =============================================================================
// TYPES
// =============================================================================

/**
 * Supported button variants for action triggers.
 */
export type ActionButtonVariant =
  | "default"
  | "destructive"
  | "outline"
  | "secondary"
  | "ghost"
  | "link"
  | "icon_destructive";

/**
 * Props for the ProcessActionDialog component.
 */
export interface ProcessActionDialogProps {
  /** Button label text */
  label: string;

  /** Visual variant for the trigger button */
  variant?: ActionButtonVariant;

  /** Whether to use small button size */
  tiny?: boolean;

  /** Dialog component to render (receives process as prop) */
  DialogComponent: FC<{ process: any; onClose?: () => void }>;

  /** Process data to pass to the dialog */
  process: any;
}

// =============================================================================
// COMPONENT
// =============================================================================

/**
 * Wraps an action button with its associated dialog.
 *
 * Provides a consistent pattern for process actions where:
 * 1. User clicks the action button
 * 2. A dialog opens with process details
 * 3. Dialog handles the actual mutation
 *
 * @param props - ProcessActionDialog properties
 * @returns A button that triggers the associated dialog
 */
export const ProcessActionDialog: FC<ProcessActionDialogProps> = ({
  label,
  variant,
  tiny,
  DialogComponent,
  process,
}) => {
  const [open, setOpen] = useState(false);

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      {/* Trigger button */}
      <DialogTrigger asChild>
        <Button variant={variant} size={(tiny ? "sm" : "") as ButtonSizes}>
          {label}
        </Button>
      </DialogTrigger>

      {/* Dialog content - receives process data */}
      <DialogComponent process={process} onClose={() => setOpen(false)} />
    </Dialog>
  );
};
