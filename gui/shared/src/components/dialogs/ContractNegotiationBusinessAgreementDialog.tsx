import React from "react";
import { BaseProcessDialog, mapCNProcessToInfoItemsForProvider } from "./base";
import { NegotiationProcessDto } from "../../data/orval/model";
import { useBffRpcSetupAgreement } from "../../data/orval/negotiation-rp-c/negotiation-rp-c";
import { EUNOMIA_CONTEXT } from "../../data/eunomia";
import {
  useGetNegotiationProcesses,
  useGetLastOfferByProcessId,
} from "../../data/orval/negotiations/negotiations";
import { useRouter } from "@tanstack/react-router";
import { PolicyWrapperShow } from "../PolicyWrapperShow";

export const ContractNegotiationBusinessAgreementDialog = ({
  process,
  onClose,
}: {
  process: NegotiationProcessDto;
  onClose?: () => void;
}) => {
  const { mutateAsync: agreeAsync } = useBffRpcSetupAgreement();
  const { refetch } = useGetNegotiationProcesses();
  const { data: lastOfferData } = useGetLastOfferByProcessId(process.id);
  const lastOffer = lastOfferData?.status === 200 ? lastOfferData.data : undefined;
  const router = useRouter();

  const datasetId = process.correlation.identifiers?.datasetId;
  const catalogId = process.correlation.identifiers?.catalogId;

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
    if (onClose) onClose();
  };

  return (
    <BaseProcessDialog
      title="Agreement Dialog"
      description={
        <>
          You are about to agree to the contract negotiation terms.
          <br />
          This will automatically complete the negotiation (Agreement - Verification -
          Finalization).
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
