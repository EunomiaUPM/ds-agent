import { Button, ButtonSizes } from "../ui/button";
import React, { useContext } from "react";
import { cva } from "class-variance-authority";
import { ContractNegotiationBusinessAgreementDialog } from "../dialogs/ContractNegotiationBusinessAgreementDialog";
import { ContractNegotiationBusinessAcceptanceDialog } from "../dialogs/ContractNegotiationBusinessAcceptanceDialog";
import { ContractNegotiationTerminationDialog } from "../dialogs/ContractNegotiationTerminationDialog";
import NoFurtherActions from "../ui/noFurtherActions";
import { ProcessActionDialog } from "./ProcessActionDialog";
import { NegotiationProcessDto } from "shared/src/data/orval/model/negotiationProcessDto";
import { Link } from "@tanstack/react-router";
import { ArrowRight } from "lucide-react";
/**
 * Actions available for a contract negotiation process.
 */
export const ContractNegotiationBusinessActions = ({
  process,
  tiny = false,
}: {
  process: NegotiationProcessDto;
  tiny: boolean;
}) => {
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
    if (process.role === "Provider") {
      switch (process.state) {
        case "REQUESTED":
        case "ACCEPTED":
          return [
            {
              label: "Agree",
              variant: "default",
              Component: ContractNegotiationBusinessAgreementDialog,
            },
          ];
        case "AGREED":
        case "VERIFIED":
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
              Component: ContractNegotiationTerminationDialog,
            },
          ];
        case "OFFERED":
          return [
            {
              label: "Accept",
              variant: "default",
              Component: ContractNegotiationBusinessAcceptanceDialog,
            },
            {
              label: "Terminate",
              variant: "destructive",
              Component: ContractNegotiationTerminationDialog,
            },
          ];
        case "AGREED":
          return [];
        default:
          return [];
      }
    }
    return [];
  };

  // Get the actions for the current process state and user role
  const actions = getActions();

  // Determine if no further actions are available
  const showNoFurtherActions = () =>
    process.state === "TERMINATED" ||
    process.state === "AGREED" ||
    process.state === "VERIFIED" ||
    (process.role === "Consumer" && process.state === "ACCEPTED");

  // Determine if agreement can be viewed
  const showGoToAgreement = () => {
    return process.state === "FINALIZED" && !!process.agreement;
  };

  return (
    <div className={containerClassName({ tiny })}>
      <div
        className={
          process.state === "OFFERED" ||
          process.state === "ACCEPTED" ||
          process.state === "VERIFIED"
            ? "flex justify-end flex-row-reverse gap-2"
            : process.state === "REQUESTED"
              ? "space-x-2 min-w-[260px]"
              : "flex justify-start gap-2"
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
        {showGoToAgreement() && (
          <Link to="/agreements/$agreementId" params={{ agreementId: process.agreement!.id }}>
            <Button variant="link">
              See agreement <ArrowRight />
            </Button>
          </Link>
        )}
      </div>
    </div>
  );
};
