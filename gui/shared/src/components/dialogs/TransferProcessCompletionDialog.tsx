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
import {
  useGetTransferProcesses,
  useGetTransferProcessById,
} from "../../data/orval/transfers/transfers";
import { EUNOMIA_CONTEXT } from "../../data/eunomia";

export const TransferProcessCompletionDialog = ({
  process,
  onClose,
}: {
  process: TransferProcessDto;
  onClose?: () => void;
}) => {
  const { mutateAsync: completeAsync } = useSetupTransferCompletion();
  const { refetch } = useGetTransferProcesses();
  const { refetch: refetchDetail } = useGetTransferProcessById(process.id!);
  const router = useRouter();

  /**
   * Handles the completion submission.
   * Payload structure differs based on the user's role.
   */
  const handleSubmit = async () => {
    await completeAsync({
      data: {
        "@context": EUNOMIA_CONTEXT,
        "@type": "RpcTransferCompletionMessage",
        "@id": process.id,
      },
    });

    await refetch();
    await refetchDetail();
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
