/**
 * ContractNegotiationVerificationDialog.tsx
 *
 * Dialog for verifying a contract negotiation agreement.
 * Consumer-side action to verify the agreement received from provider.
 */

import React, { useContext } from "react";
import { GlobalInfoContext, GlobalInfoContextType } from "../../context/GlobalInfoContext";
import { BaseProcessDialog, mapCNProcessToInfoItemsForConsumer } from "./base";
import { useRpcSetupVerification } from "../../data/orval/negotiation-rp-c/negotiation-rp-c";
import { NegotiationProcessDto } from "../../data/orval/model";
import { useGetNegotiationProcesses } from "../../data/orval/negotiations/negotiations";
import { useRouter } from "@tanstack/react-router";
import { PolicyWrapperShow } from "../PolicyWrapperShow";

export const ContractNegotiationVerificationDialog = ({
  process,
  onClose,
}: {
  process: NegotiationProcessDto;
  onClose?: () => void;
}) => {
  const { api_gateway } = useContext<GlobalInfoContextType | null>(GlobalInfoContext)!;
  const { mutateAsync: verifyAsync } = useRpcSetupVerification();
  const { refetch } = useGetNegotiationProcesses();
  const router = useRouter();

  /**
   * Handles the verification submission.
   * Sends verification message to the provider.
   */
  const handleSubmit = async () => {
    if (!process.identifiers?.consumerPid || !process.identifiers?.providerPid) {
      console.error("Missing process identifiers");
      return;
    }
    await verifyAsync({
      data: {
        providerPid: process.identifiers.providerPid,
        consumerPid: process.identifiers.consumerPid,
      },
    });
    await refetch();
    router.invalidate();
    if (onClose) {
      onClose();
    }
  };

  return (
    <BaseProcessDialog
      title="Verification Dialog"
      description={
        <>
          You are about to verify the agreement.
          <br />
          Please review the details carefully before proceeding.
        </>
      }
      infoItems={mapCNProcessToInfoItemsForConsumer(process)}
      submitLabel="Verify"
      submitVariant="default"
      onSubmit={handleSubmit}
      scrollable={true}
      afterInfoContent={
        <div className="pt-4">
          <PolicyWrapperShow
            policy={
              process.agreement?.agreementContent || {
                permission: [],
                prohibition: [],
                obligation: [],
              }
            }
            datasetId={process.identifiers?.datasetId}
            catalogId={process.identifiers?.catalogId}
          />
        </div>
      }
    />
  );
};
