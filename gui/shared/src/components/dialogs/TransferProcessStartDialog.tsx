/**
 * TransferProcessStartDialog.tsx
 *
 * Dialog for starting a transfer process.
 * Available to both provider and consumer roles.
 */

import React, { useContext } from "react";
import { GlobalInfoContext, GlobalInfoContextType } from "../../context/GlobalInfoContext";
import { BaseProcessDialog, mapTransferProcessToInfoItems } from "./base";
import { DataAddressDto, TransferProcessDto } from "../../data/orval/model";
import { useSetupTransferStart } from "../../data/orval/transfer-rp-c/transfer-rp-c";
import { useRouter } from "@tanstack/react-router";
import { useGetTransferProcesses, useGetTransferProcessById } from "../../data/orval/transfers/transfers";

export const TransferProcessStartDialog = ({ process, onClose }: {
  process: TransferProcessDto; onClose?: () => void;
}) => {
  const { mutateAsync: startAsync } = useSetupTransferStart();
  const { refetch } = useGetTransferProcesses();
  const { refetch: refetchDetail } = useGetTransferProcessById(process.id!);
  const router = useRouter();

  /**
   * Handles the start submission.
   * Payload structure differs based on the user's role.
   */
  const handleSubmit = async () => {
    if (!process.identifiers?.consumerPid || !process.identifiers?.providerPid) {
      console.error("Missing process identifiers");
      return;
    }
    await startAsync({
      data: {
        consumerPid: process.identifiers.consumerPid,
        providerPid: process.identifiers.providerPid,
      }
    })

    await refetch();
    await refetchDetail();
    router.invalidate();
    if (onClose) {
      onClose();
    }
  };

  return (
    <BaseProcessDialog
      title="Transfer Start Dialog"
      description="You are about to start the transfer process."
      infoItems={mapTransferProcessToInfoItems(process)}
      submitLabel="Start"
      submitVariant="default"
      onSubmit={handleSubmit}
    />
  );
};
