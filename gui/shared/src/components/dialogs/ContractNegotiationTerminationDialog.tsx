/**
 * ContractNegotiationTerminationDialog.tsx
 *
 * Dialog for terminating a contract negotiation process.
 * Uses the base dialog component for consistent structure.
 */

import { GlobalInfoContext, GlobalInfoContextType } from "../../context/GlobalInfoContext";
import { BaseProcessDialog, mapCNProcessToFullInfoItems } from "./base";
import { NegotiationProcessDto } from "../../data/orval/model";
import {
  useRpcSetupFinalization,
  useRpcSetupTermination,
} from "../../data/orval/negotiation-rp-c/negotiation-rp-c";
import { EUNOMIA_CONTEXT } from "../../data/eunomia";
import { useRouter } from "@tanstack/react-router";
import React, { useContext } from "react";
import { useGetNegotiationProcesses } from "../../data/orval/negotiations/negotiations";
export const ContractNegotiationTerminationDialog = ({
  process,
  onClose,
}: {
  process: NegotiationProcessDto;
  onClose?: () => void;
}) => {
  const { api_gateway } = useContext<GlobalInfoContextType | null>(GlobalInfoContext)!;
  const { mutateAsync: terminateAsync, reset } = useRpcSetupTermination();
  const { refetch } = useGetNegotiationProcesses();
  const router = useRouter();

  /**
   * Handles the termination submission.
   * The payload differs based on whether the user is a provider or consumer.
   */
  const handleSubmit = async () => {
    await terminateAsync({
      data: {
        "@context": EUNOMIA_CONTEXT,
        "@type": "RpcNegotiationTerminationMessage",
        "@id": process.id,
        code: "TERMINATED",
        reason: ["Terminated from GUI"],
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
      title="Termination Dialog"
      description={
        <>
          You are about to terminate the contract negotiation.
          <br />
          Please review the details carefully before proceeding.
        </>
      }
      infoItems={mapCNProcessToFullInfoItems(process)}
      submitLabel="Terminate"
      submitVariant="destructive"
      onSubmit={handleSubmit}
    />
  );
};
