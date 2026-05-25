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
        true: "inline-flex items-center ",
        false:
          "w-[calc(100%+2px-var(--sidebar-width))] p-6 fixed bottom-0 -right-px bg-background/80 backdrop-blur-sm border border-t-stroke z-50 [&>*>button]:min-w-20",
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

  const isTerminalState = () =>
    process.state === "COMPLETED" || process.state === "TERMINATED";

  const showSwitchToStandard = () =>
    actions.length === 0 && !isTerminalState();

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
        {isTerminalState() && <NoFurtherActions />}
        {showSwitchToStandard() && (
          <span className="text-xs text-white/40 italic">
            This step is only visible in Standard mode
          </span>
        )}
      </div>
    </div>
  );
};
