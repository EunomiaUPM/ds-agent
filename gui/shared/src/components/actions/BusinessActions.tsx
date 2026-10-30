import React, { useContext } from "react";
import { AuthContext, AuthContextType } from "shared/src/context/AuthContext";
import { BusinessRequestTerminationDialog } from "../dialogs/BusinessRequestTerminationDialog";
import { BusinessRequestAcceptanceDialog } from "../dialogs/BusinessRequestAcceptanceDialog";
import { NoFurtherActions } from "@eunomia/ui";
import { ProcessActionDialog } from "./ProcessActionDialog";

/**
 * Actions available for a business process.
 */
export const BusinessActions = ({ process }: { process: any }) => {
  const { participant } = useContext<AuthContextType | null>(AuthContext)!;
  const role = participant?.participant_type;

  const getActions = () => {
    if (role === "Provider") {
      if (process.state === "REQUESTED") {
        return [
          { label: "Accept", variant: "default", Component: BusinessRequestAcceptanceDialog },
          { label: "Reject", variant: "destructive", Component: BusinessRequestTerminationDialog },
        ];
      }
    } else if (role === "Consumer") {
      if (process.state === "REQUESTED") {
        return [
          { label: "Cancel", variant: "destructive", Component: BusinessRequestTerminationDialog },
        ];
      }
    }
    return [];
  };

  const actions = getActions();

  const showNoFurtherActions = () => {
    return actions.length === 0;
  };

  return (
    <div className="inline-flex items-center">
      <div className="space-x-2 min-w-[260px]">
        {actions.map((action, idx) => (
          <ProcessActionDialog
            key={idx}
            label={action.label}
            variant={action.variant as any}
            DialogComponent={action.Component}
            process={process}
          />
        ))}
        {showNoFurtherActions() && <NoFurtherActions />}
      </div>
    </div>
  );
};
