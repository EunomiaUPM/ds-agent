/**
 * TransferProcessSuspensionDialog.tsx
 *
 * Dialog for suspending a transfer process.
 * Available to both provider and consumer roles.
 */

import React, { useContext } from "react";
import { GlobalInfoContext, GlobalInfoContextType } from "../../context/GlobalInfoContext";
import { BaseProcessDialog, mapTransferProcessToInfoItems } from "./base";
import { TransferProcessDto } from "../../data/orval/model";
import { useSetupTransferSuspension } from "../../data/orval/transfer-rp-c/transfer-rp-c";
import { useGetTransferProcesses } from "../../data/orval/transfers/transfers";
import { useRouter } from "@tanstack/react-router";

export const TransferProcessSuspensionDialog = ({ process, onClose }: { process: TransferProcessDto; onClose?: () => void }) => {
  const { api_gateway, dsrole } = useContext<GlobalInfoContextType | null>(GlobalInfoContext)!;
  const { mutateAsync: suspendAsync } = useSetupTransferSuspension();
  const { refetch } = useGetTransferProcesses();
  const router = useRouter();
  /**
   * Handles the suspension submission.
   */
  const handleSubmit = async () => {
    await suspendAsync({
      data: {
        consumerPid: process.identifiers.consumerPid,
        providerPid: process.identifiers.providerPid,
        code: "SUSPENDED",
        reason: ["Suspended from GUI"],
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
      title="Transfer Suspension Dialog"
      description="You are about to suspend the transfer process."
      infoItems={mapTransferProcessToInfoItems(process)}
      submitLabel="Suspend"
      submitVariant="outline"
      onSubmit={handleSubmit}
    />
  );
};
