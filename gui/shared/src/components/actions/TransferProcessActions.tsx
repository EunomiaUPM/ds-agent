import React, { FC, useContext } from "react";
import { cva } from "class-variance-authority";
import { GlobalInfoContext, GlobalInfoContextType } from "shared/src/context/GlobalInfoContext";
import { TransferProcessStartDialog } from "shared/src/components/dialogs/TransferProcessStartDialog";
import { TransferProcessTerminationDialog } from "shared/src/components/dialogs/TransferProcessTerminationDialog";
import NoFurtherActions from "../ui/noFurtherActions";
import { TransferProcessSuspensionDialog } from "shared/src/components/dialogs/TransferProcessSuspensionDialog";
import { TransferProcessCompletionDialog } from "shared/src/components/dialogs/TransferProcessCompletionDialog";
import { ProcessActionDialog } from "./ProcessActionDialog";
import { TransferProcessDto } from "../../data/orval/model";

/**
 * Actions available for a transfer process.
 */
export const TransferProcessActions: FC<{
  process: TransferProcessDto;
  tiny: boolean;
}> = ({ process, tiny = false }) => {
  // Define container class name with variants
  const containerClassName = cva("", {
    variants: {
      tiny: {
        true: "inline-flex items-center gap-2",
        false:
          "fixed bottom-0 right-0 left-0 md:left-[var(--sidebar-width)] peer-data-[state=collapsed]:md:left-[var(--sidebar-width-icon)] z-30 px-6 py-3.5 bg-background/85 backdrop-blur-md border-t border-ink/10 shadow-lg transition-[left] duration-200 ease-linear [&>*>button]:min-w-20",
      },
    },
  });

  // Determine available actions based on process state and user role
  const getActions = () => {
    if (process.role === "Provider") {
      switch (process.state) {
        case "REQUESTED":
          return [
            {
              label: "Terminate",
              variant: "destructive",
              Component: TransferProcessTerminationDialog,
            },
            { label: "Start", variant: "default", Component: TransferProcessStartDialog },
          ];
        case "STARTED":
          return [
            {
              label: "Terminate",
              variant: "destructive",
              Component: TransferProcessTerminationDialog,
            },
            { label: "Suspend", variant: "outline", Component: TransferProcessSuspensionDialog },
            { label: "Complete", variant: "outline", Component: TransferProcessCompletionDialog },
          ];
        case "SUSPENDED":
          const actions = [
            {
              label: "Terminate",
              variant: "destructive",
              Component: TransferProcessTerminationDialog,
            },
          ];
          if (process.stateAttribute && process.stateAttribute !== "ByConsumer") {
            actions.push({
              label: "Start",
              variant: "default",
              Component: TransferProcessStartDialog,
            });
          }
          actions.push({
            label: "Complete",
            variant: "outline",
            Component: TransferProcessCompletionDialog,
          });
          return actions;
        case "COMPLETED":
        case "TERMINATED":
          return [];
        default:
          return [];
      }
    } else if (process.role === "Consumer") {
      switch (process.state) {
        case "REQUESTED":
          return [
            {
              label: "Terminate",
              variant: "destructive",
              Component: TransferProcessTerminationDialog,
            },
          ];
        case "STARTED":
          return [
            {
              label: "Terminate",
              variant: "destructive",
              Component: TransferProcessTerminationDialog,
            },
            { label: "Suspend", variant: "outline", Component: TransferProcessSuspensionDialog },
            { label: "Complete", variant: "outline", Component: TransferProcessCompletionDialog },
          ];
        case "SUSPENDED":
          const actions = [
            {
              label: "Terminate",
              variant: "destructive",
              Component: TransferProcessTerminationDialog,
            },
          ];
          if (process.stateAttribute && process.stateAttribute !== "ByProvider") {
            actions.push({
              label: "Start",
              variant: "default",
              Component: TransferProcessStartDialog,
            });
          }
          actions.push({
            label: "Complete",
            variant: "outline",
            Component: TransferProcessCompletionDialog,
          });
          return actions;
        case "COMPLETED":
        case "TERMINATED":
          return [];
        default:
          return [];
      }
    }
    return [];
  };

  // Get the list of actions
  const actions = getActions();

  // Determine if no further actions are available
  const showNoFurtherActions = () => {
    if (process.state === "COMPLETED" || process.state === "TERMINATED") return true;
    return false;
  };

  if (!tiny && actions.length === 0 && !showNoFurtherActions()) {
    return null;
  }

  return (
    <>
      {!tiny && <div className="h-24 w-full shrink-0 pointer-events-none" aria-hidden="true" />}
      <div className={containerClassName({ tiny })}>
        <div
          className={
            tiny
              ? "inline-flex items-center gap-2"
              : "flex items-center justify-start gap-2.5 flex-wrap w-full"
          }
        >
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
    </>
  );
};
