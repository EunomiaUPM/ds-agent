/**
 * TransferProcessTerminationDialog.tsx
 *
 * Dialog for terminating a transfer process.
 * Available to both provider and consumer roles.
 */

import React, { useContext } from "react";
import { GlobalInfoContext, GlobalInfoContextType } from "../../context/GlobalInfoContext";
import { BaseProcessDialog, mapTransferProcessToInfoItems } from "./base";
import { TransferProcessDto } from "../../data/orval/model";
import { useSetupTransferTermination } from "../../data/orval/transfer-rp-c/transfer-rp-c";
import { useRouter } from "@tanstack/react-router";
import { useGetTransferProcesses } from "../../data/orval/transfers/transfers";

export const TransferProcessTerminationDialog = ({ process, onClose }: {
  process: TransferProcessDto; onClose?: () => void;
}) => {
  const { mutateAsync: terminateAsync } = useSetupTransferTermination();
  const { refetch } = useGetTransferProcesses();
  const router = useRouter();

  /**
   * Handles the termination submission.
   * Payload structure differs based on the user's role.
   */
  const handleSubmit = async () => {
    if (!process.identifiers?.consumerPid || !process.identifiers?.providerPid) {
      console.error("Missing process identifiers");
      return;
    }
    await terminateAsync({
      data: {
        consumerPid: process.identifiers.consumerPid,
        providerPid: process.identifiers.providerPid,
        code: "TERMINATED",
        reason: ["Terminated from GUI"],
      }
    })

    await refetch();
    router.invalidate();
    if (onClose) {
      onClose();
    }
  };

  return (
    <BaseProcessDialog
      title="Transfer Termination Dialog"
      description="You are about to terminate the transfer process."
      infoItems={mapTransferProcessToInfoItems(process)}
      submitLabel="Terminate"
      submitVariant="destructive"
      onSubmit={handleSubmit}
    />
  );
};
