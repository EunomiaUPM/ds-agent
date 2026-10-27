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
import { EUNOMIA_CONTEXT } from "../../data/eunomia";
import {
  useGetNegotiationProcesses,
  useGetLastOfferByProcessId,
} from "../../data/orval/negotiations/negotiations";
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
  const { data: lastOfferData } = useGetLastOfferByProcessId(process.id);
  const lastOffer = lastOfferData?.status === 200 ? lastOfferData.data : undefined;
  const router = useRouter();

  const datasetId = process.correlation.identifiers?.datasetId;
  const catalogId = process.correlation.identifiers?.catalogId;

  /**
   * Handles the acceptance submission.
   * Sends acceptance event to the provider.
   */
  const handleSubmit = async () => {
    await acceptAsync({
      data: {
        "@context": EUNOMIA_CONTEXT,
        "@type": "RpcNegotiationAcceptanceMessage",
        "@id": process.id,
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
        lastOffer?.offerContent ? (
          <div className="pt-4">
            <PolicyWrapperShow
              policy={lastOffer.offerContent}
              datasetId={datasetId}
              catalogId={catalogId}
            />
          </div>
        ) : undefined
      }
    />
  );
};
