import { Button } from "../ui/button";
import React from "react";
import { cva } from "class-variance-authority";
import { ContractNegotiationBusinessAgreementDialog } from "../dialogs/ContractNegotiationBusinessAgreementDialog";
import { ContractNegotiationBusinessAcceptanceDialog } from "../dialogs/ContractNegotiationBusinessAcceptanceDialog";
import { ContractNegotiationTerminationDialog } from "../dialogs/ContractNegotiationTerminationDialog";
import NoFurtherActions from "../ui/noFurtherActions";
import { ProcessActionDialog } from "./ProcessActionDialog";
import { NegotiationProcessDto } from "shared/src/data/orval/model/negotiationProcessDto";
import { Link } from "@tanstack/react-router";
import { ArrowRight } from "lucide-react";

export const ContractNegotiationBusinessActions = ({
  process,
  tiny = false,
}: {
  process: NegotiationProcessDto;
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
        case "ACCEPTED":
          return [
            {
              label: "Agree",
              variant: "default",
              Component: ContractNegotiationBusinessAgreementDialog,
            },
            {
              label: "Terminate",
              variant: "destructive",
              Component: ContractNegotiationTerminationDialog,
            },
          ];
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
        default:
          return [];
      }
    }
    return [];
  };

  const actions = getActions();

  const isTerminalState = () =>
    process.state === "TERMINATED" ||
    process.state === "AGREED" ||
    process.state === "VERIFIED" ||
    process.state === "FINALIZED" ||
    (process.role === "Consumer" && process.state === "ACCEPTED");

  const showGoToAgreement = () => process.state === "FINALIZED" && !!process.agreement;

  const showSwitchToStandard = () =>
    actions.length === 0 && !isTerminalState() && !showGoToAgreement();

  if (!tiny && actions.length === 0 && !isTerminalState() && !showGoToAgreement() && !showSwitchToStandard()) {
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
          {isTerminalState() && !showGoToAgreement() && <NoFurtherActions />}
          {showGoToAgreement() && (
            <Link to="/agreements/$agreementId" params={{ agreementId: process.agreement!.id }}>
              <Button variant="link" className="gap-1.5">
                See agreement <ArrowRight className="h-4 w-4" />
              </Button>
            </Link>
          )}
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
