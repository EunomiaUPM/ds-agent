/**
 * AgreementTerminationDialog.tsx
 *
 * Dialog for terminating an agreement.
 */

import React, { useContext } from "react";
import { GlobalInfoContext, GlobalInfoContextType } from "../../context/GlobalInfoContext";
import { BaseProcessDialog, mapCNProcessToInfoItemsForConsumer } from "./base";
import { NegotiationProcessDto } from "../../data/orval/model";
import { useRpcSetupAcceptance } from "../../data/orval/negotiation-rp-c/negotiation-rp-c";
import { useGetNegotiationProcesses } from "../../data/orval/negotiations/negotiations";
import { useRouter } from "@tanstack/react-router";
import { PolicyWrapperShow } from "../PolicyWrapperShow";

export const AgreementTerminationDialog = ({
  process,
  onClose,
}: {
  process: NegotiationProcessDto;
  onClose?: () => void;
}) => {
  const { api_gateway } = useContext<GlobalInfoContextType | null>(GlobalInfoContext)!;
  const { mutateAsync: acceptAsync } = useRpcSetupAcceptance();
  const { refetch } = useGetNegotiationProcesses();
  const router = useRouter();

  /**
   * Handles the acceptance submission.
   * Sends acceptance event to the provider.
   */
  const handleSubmit = async () => {
    console.log("Terminating agreement");
  };

  return (
    <BaseProcessDialog
      title="Termination Dialog"
      description={
        <>
          You are about to terminate the agreement.
          <br />
          Please review the details carefully before proceeding.
        </>
      }
      infoItems={[]}
      submitLabel="Terminate"
      submitVariant="destructive"
      onSubmit={handleSubmit}
      scrollable={true}
      afterInfoContent={
        "Still to be implemented"
      }
    />
  );
};
