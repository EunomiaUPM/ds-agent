import React, { FC, useContext } from "react";
import { cva } from "class-variance-authority";
import { GlobalInfoContext, GlobalInfoContextType } from "shared/src/context/GlobalInfoContext";
import { TransferProcessStartDialog } from "shared/src/components/dialogs/TransferProcessStartDialog";
import { TransferProcessTerminationDialog } from "shared/src/components/dialogs/TransferProcessTerminationDialog";
import NoFurtherActions from "../ui/noFurtherActions";
import { TransferProcessSuspensionDialog } from "shared/src/components/dialogs/TransferProcessSuspensionDialog";
import { TransferProcessCompletionDialog } from "shared/src/components/dialogs/TransferProcessCompletionDialog";
import { ProcessActionDialog } from "./ProcessActionDialog";
import { AgreementDto } from "../../data/orval/model";
import { useGetNegotiationProcessById } from "../../data/orval/negotiations/negotiations";
import { TransferProcessRequestDialog } from "../dialogs/TransferProcessRequestDialog";

/**
 * Actions available for a transfer process.
 */
export const AgreementActions: FC<{
  process: AgreementDto;
  tiny: boolean;
}> = ({ process, tiny = false }) => {
  const { data: negotiationProcessResponse } = useGetNegotiationProcessById(
    process.negotiationAgentProcessId,
  );
  const negotiationProcess = negotiationProcessResponse?.status === 200 ? negotiationProcessResponse.data : undefined;

  // Define container class name with variants
  const containerClassName = cva("", {
    variants: {
      tiny: {
        true: "inline-flex items-center ",
        false:
          "w-[calc(100%_+_2px_-_var(--sidebar-width))] p-6 fixed bottom-0 -right-px bg-background/80 backdrop-blur-sm border border-t-stroke [&>*>button]:min-w-20",
      },
    },
  });

  // Determine available actions based on process state and user role
  const getActions = () => {
    if (process.state === "ACTIVE" && negotiationProcess?.role === "Provider") {
      return [
        {
          label: "Terminate",
          variant: "destructive",
          Component: TransferProcessTerminationDialog,
        }
      ];
    }
    if (negotiationProcess?.role === "Consumer") {
      return [
        {
          label: "Terminate",
          variant: "destructive",
          Component: TransferProcessTerminationDialog,
        },
        {
          label: "Transfer Request",
          variant: "default",
          Component: TransferProcessRequestDialog,
        }
      ];
    }
    return [];
  };

  // Get the list of actions
  const actions = getActions();

  // Determine if no further actions are available
  const showNoFurtherActions = () => {
    if (process.state === "TERMINATED" || process.state === "REQUESTED") return true;
    return false;
  };

  return (
    <div className={containerClassName({ tiny })}>
      <div className="space-x-2">
        {actions.map((action, idx) => (
          <ProcessActionDialog
            key={idx}
            label={action.label}
            variant={action.variant as any}
            tiny={tiny}
            DialogComponent={action.Component}
            process={process}
          />
        ))}
        {showNoFurtherActions() && <NoFurtherActions />}
      </div>
    </div>
  );
};
