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
import { EUNOMIA_CONTEXT } from "../../data/eunomia";
import { NegotiationProcessDto } from "../../data/orval/model";
import {
  useGetAgreementsByNegotiationProcess,
  useGetNegotiationProcesses,
} from "../../data/orval/negotiations/negotiations";
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
  const { data: agreementsData } = useGetAgreementsByNegotiationProcess(process.id);
  const router = useRouter();

  const agreement = agreementsData?.status === 200 ? agreementsData.data.items?.[0] : undefined;
  const datasetId = process.correlation.identifiers?.datasetId;
  const catalogId = process.correlation.identifiers?.catalogId;

  /**
   * Handles the verification submission.
   * Sends verification message to the provider.
   */
  const handleSubmit = async () => {
    await verifyAsync({
      data: {
        "@context": EUNOMIA_CONTEXT,
        "@type": "RpcNegotiationVerificationMessage",
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
        agreement?.agreementContent ? (
          <div className="pt-4">
            <PolicyWrapperShow
              policy={agreement.agreementContent}
              datasetId={datasetId}
              catalogId={catalogId}
            />
          </div>
        ) : undefined
      }
    />
  );
};
