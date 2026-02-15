/**
 * TransferProcessCompletionDialog.tsx
 *
 * Dialog for completing a transfer process.
 * Available to both provider and consumer roles.
 */

import React, { useContext } from "react";
import { GlobalInfoContext, GlobalInfoContextType } from "../../context/GlobalInfoContext";
import { BaseProcessDialog, mapTransferProcessToInfoItems } from "./base";
import { TransferProcessDto } from "../../data/orval/model";
import { useSetupTransferCompletion } from "../../data/orval/transfer-rp-c/transfer-rp-c";
import { useRouter } from "@tanstack/react-router";
import { useGetTransferProcesses } from "../../data/orval/transfers/transfers";


export const TransferProcessCompletionDialog = ({ process, onClose }: {
  process: TransferProcessDto; onClose?: () => void;
}) => {
  const { mutateAsync: completeAsync } = useSetupTransferCompletion();
  const { refetch } = useGetTransferProcesses();
  const router = useRouter();

  /**
   * Handles the completion submission.
   * Payload structure differs based on the user's role.
   */
  const handleSubmit = async () => {
    if (!process.identifiers?.consumerPid || !process.identifiers?.providerPid) {
      console.error("Missing process identifiers");
      return;
    }
    await completeAsync({
      data: {
        consumerPid: process.identifiers.consumerPid,
        providerPid: process.identifiers.providerPid,
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
      title="Transfer Completion Dialog"
      description="You are about to complete the transfer process."
      infoItems={mapTransferProcessToInfoItems(process)}
      submitLabel="Complete"
      submitVariant="outline"
      onSubmit={handleSubmit}
    />
  );
};
