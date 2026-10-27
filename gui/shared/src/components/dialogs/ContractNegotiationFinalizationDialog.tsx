/**
 * ContractNegotiationFinalizationDialog.tsx
 *
 * Dialog for finalizing a contract negotiation.
 * Provider-side action to complete the negotiation after verification.
 */

import React, { useContext } from "react";
import { GlobalInfoContext, GlobalInfoContextType } from "../../context/GlobalInfoContext";
import { BaseProcessDialog, mapCNProcessToInfoItemsForProvider } from "./base";
import { useRpcSetupFinalization } from "../../data/orval/negotiation-rp-c/negotiation-rp-c";
import { EUNOMIA_CONTEXT } from "../../data/eunomia";
import { NegotiationProcessDto } from "../../data/orval/model";
import {
  useGetAgreementsByNegotiationProcess,
  useGetNegotiationProcesses,
} from "../../data/orval/negotiations/negotiations";
import { useRouter } from "@tanstack/react-router";
import { PolicyWrapperShow } from "../PolicyWrapperShow";

export const ContractNegotiationFinalizationDialog = ({
  process,
  onClose,
}: {
  process: NegotiationProcessDto;
  onClose?: () => void;
}) => {
  const { api_gateway } = useContext<GlobalInfoContextType | null>(GlobalInfoContext)!;
  const { mutateAsync: finalizeAsync } = useRpcSetupFinalization();
  const { refetch } = useGetNegotiationProcesses();
  const { data: agreementsData } = useGetAgreementsByNegotiationProcess(process.id);
  const router = useRouter();

  const agreement = agreementsData?.status === 200 ? agreementsData.data.items?.[0] : undefined;
  const datasetId = process.correlation.identifiers?.datasetId;
  const catalogId = process.correlation.identifiers?.catalogId;

  /**
   * Handles the finalization submission.
   * Sends finalization event to close the negotiation.
   */
  const handleSubmit = async () => {
    await finalizeAsync({
      data: {
        "@context": EUNOMIA_CONTEXT,
        "@type": "RpcNegotiationFinalizationMessage",
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
      title="Finalization Dialog"
      description={
        <>
          You are about to finalize the contract negotiation.
          <br />
          Please review the details carefully before proceeding.
        </>
      }
      infoItems={mapCNProcessToInfoItemsForProvider(process)}
      submitLabel="Finalize"
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
