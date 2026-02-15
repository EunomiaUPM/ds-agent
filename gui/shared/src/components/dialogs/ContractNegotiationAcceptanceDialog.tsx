/**
 * ContractNegotiationAcceptanceDialog.tsx
 *
 * Dialog for accepting a contract negotiation offer.
 * Consumer-side action to accept offered terms.
 */

import React, { useContext } from "react";
import { GlobalInfoContext, GlobalInfoContextType } from "../../context/GlobalInfoContext";
import { BaseProcessDialog, mapCNProcessToInfoItemsForConsumer } from "./base";
import { NegotiationProcessDto } from "../../data/orval/model";
import { useRpcSetupAcceptance } from "../../data/orval/negotiation-rp-c/negotiation-rp-c";
import { useGetNegotiationProcesses } from "../../data/orval/negotiations/negotiations";
import { useRouter } from "@tanstack/react-router";
import { PolicyWrapperShow } from "../PolicyWrapperShow";

export const ContractNegotiationAcceptanceDialog = ({
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
    await acceptAsync({
      data: {
        consumerPid: process.identifiers!.consumerPid,
        providerPid: process.identifiers!.providerPid,
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
      title="Acceptance Dialog"
      description={
        <>
          You are about to accept the contract offer.
          <br />
          Please review the details carefully before proceeding.
        </>
      }
      infoItems={mapCNProcessToInfoItemsForConsumer(process)}
      submitLabel="Accept"
      submitVariant="default"
      onSubmit={handleSubmit}
      scrollable={true}
      afterInfoContent={
        <div className="pt-4">
          <PolicyWrapperShow
            policy={process.offers!.at(-1)!.offerContent}
            datasetId={process.identifiers!.datasetId}
            catalogId={process.identifiers!.catalogId}
          />
        </div>
      }
    />
  );
};
