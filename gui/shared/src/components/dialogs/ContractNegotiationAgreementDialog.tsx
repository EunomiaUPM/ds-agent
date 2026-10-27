/**
 * ContractNegotiationAgreementDialog.tsx
 *
 * Dialog for establishing a contract negotiation agreement.
 * Provider-side action to accept and agree to negotiation terms.
 */

import React, { useContext } from "react";
import { GlobalInfoContext, GlobalInfoContextType } from "../../context/GlobalInfoContext";
import { BaseProcessDialog, mapCNProcessToInfoItemsForProvider } from "./base";
import { NegotiationProcessDto } from "../../data/orval/model";
import { useRpcSetupAgreement } from "../../data/orval/negotiation-rp-c/negotiation-rp-c";
import { EUNOMIA_CONTEXT } from "../../data/eunomia";
import {
  useGetNegotiationProcesses,
  useGetLastOfferByProcessId,
} from "../../data/orval/negotiations/negotiations";
import { useRouter } from "@tanstack/react-router";
import { PolicyWrapperShow } from "../PolicyWrapperShow";

export const ContractNegotiationAgreementDialog = ({
  process,
  onClose,
}: {
  process: NegotiationProcessDto;
  onClose?: () => void;
}) => {
  const { api_gateway } = useContext<GlobalInfoContextType | null>(GlobalInfoContext)!;
  const { mutateAsync: agreeAsync } = useRpcSetupAgreement();
  const { refetch } = useGetNegotiationProcesses();
  const { data: lastOfferData } = useGetLastOfferByProcessId(process.id);
  const lastOffer = lastOfferData?.status === 200 ? lastOfferData.data : undefined;
  const router = useRouter();

  const datasetId = process.correlation.identifiers?.datasetId;
  const catalogId = process.correlation.identifiers?.catalogId;

  /**
   * Handles the agreement submission.
   * Sends agreement message to the consumer.
   */
  const handleSubmit = async () => {
    await agreeAsync({
      data: {
        "@context": EUNOMIA_CONTEXT,
        "@type": "RpcNegotiationAgreementMessage",
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
      title="Agreement Dialog"
      description={
        <>
          You are about to agree to the terms of the contract negotiation.
          <br />
          Please review the details carefully before proceeding.
        </>
      }
      infoItems={mapCNProcessToInfoItemsForProvider(process)}
      submitLabel="Agree"
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
