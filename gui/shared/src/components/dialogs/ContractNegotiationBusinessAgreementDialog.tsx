import React from "react";
import { BaseProcessDialog, mapCNProcessToInfoItemsForProvider } from "./base";
import { NegotiationProcessDto } from "../../data/orval/model";
import { useBffRpcSetupAgreement } from "../../data/orval/negotiation-rp-c/negotiation-rp-c";
import { useGetNegotiationProcesses } from "../../data/orval/negotiations/negotiations";
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
  const router = useRouter();

  const handleSubmit = async () => {
    await agreeAsync({
      data: {
        consumerPid: process.identifiers!.consumerPid,
        providerPid: process.identifiers!.providerPid,
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
          This will automatically complete the negotiation (Agreement - Verification - Finalization).
        </>
      }
      infoItems={mapCNProcessToInfoItemsForProvider(process)}
      submitLabel="Agree"
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
