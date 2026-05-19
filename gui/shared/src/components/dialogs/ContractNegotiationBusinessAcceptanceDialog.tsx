import React from "react";
import { BaseProcessDialog, mapCNProcessToInfoItemsForConsumer } from "./base";
import { NegotiationProcessDto } from "../../data/orval/model";
import { useBffRpcSetupAcceptance } from "../../data/orval/negotiation-rp-c/negotiation-rp-c";
import { useGetNegotiationProcesses } from "../../data/orval/negotiations/negotiations";
import { useRouter } from "@tanstack/react-router";
import { PolicyWrapperShow } from "../PolicyWrapperShow";

export const ContractNegotiationBusinessAcceptanceDialog = ({
  process,
  onClose,
}: {
  process: NegotiationProcessDto;
  onClose?: () => void;
}) => {
  const { mutateAsync: acceptAsync } = useBffRpcSetupAcceptance();
  const { refetch } = useGetNegotiationProcesses();
  const router = useRouter();

  const handleSubmit = async () => {
    await acceptAsync({
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
      title="Acceptance Dialog"
      description={
        <>
          You are about to accept the contract offer.
          <br />
          The provider will then complete the negotiation automatically.
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
