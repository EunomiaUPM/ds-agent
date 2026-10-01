import React from "react";
import { cva } from "class-variance-authority";
import { TransferProcessTerminationDialog } from "shared/src/components/dialogs/TransferProcessTerminationDialog";
import { TransferProcessCompletionDialog } from "shared/src/components/dialogs/TransferProcessCompletionDialog";
import { TransferProcessSuspensionDialog } from "shared/src/components/dialogs/TransferProcessSuspensionDialog";
import NoFurtherActions from "../ui/noFurtherActions";
import { ProcessActionDialog } from "./ProcessActionDialog";
import { TransferProcessDto } from "../../data/orval/model";

export const TransferProcessBusinessActions = ({
  process,
  tiny = false,
}: {
  process: TransferProcessDto;
  tiny: boolean;
}) => {
  const containerClassName = cva("", {
    variants: {
      tiny: {
        true: "inline-flex items-center gap-2",
        false:
          "fixed bottom-0 right-0 left-0 md:left-[var(--sidebar-width)] peer-data-[state=collapsed]:md:left-[var(--sidebar-width-icon)] z-30 px-6 py-3.5 bg-background/85 backdrop-blur-md border-t border-ink/10 shadow-lg transition-[left] duration-200 ease-linear [&>*>button]:min-w-20",
      },
    },
  });

  const getActions = () => {
    if (process.role === "Provider") {
      switch (process.state) {
        case "REQUESTED":
          // Start was already sent automatically — only allow terminating
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
        default:
          return [];
      }
    } else if (process.role === "Consumer") {
      switch (process.state) {
        case "REQUESTED":
          // Waiting for Provider auto-start — only allow terminating
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
        default:
          return [];
      }
    }
    return [];
  };

  const actions = getActions();

  const isTerminalState = () => process.state === "COMPLETED" || process.state === "TERMINATED";

  const showSwitchToStandard = () => actions.length === 0 && !isTerminalState();

  if (!tiny && actions.length === 0 && !isTerminalState() && !showSwitchToStandard()) {
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
          {isTerminalState() && <NoFurtherActions />}
          {showSwitchToStandard() && (
            <span className="text-xs text-ink/40 italic">
              This step is only visible in Standard mode
            </span>
          )}
        </div>
      </div>
    </>
  );
};
