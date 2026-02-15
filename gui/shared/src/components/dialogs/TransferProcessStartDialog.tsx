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
import { useGetTransferProcesses } from "../../data/orval/transfers/transfers";

export const TransferProcessStartDialog = ({ process, onClose }: {
  process: TransferProcessDto; onClose?: () => void;
}) => {
  const { mutateAsync: startAsync } = useSetupTransferStart();
  const { refetch } = useGetTransferProcesses();
  const router = useRouter();

  /**
   * Handles the start submission.
   * Payload structure differs based on the user's role.
   */
  const handleSubmit = async () => {
    const dataAddress = process.stateAttribute == "OnRequest" ? {
      endpointType: "https://w3id.org/idsa/v4.1/HTTP",
      endpoint: "http://example.com",
      endpointProperties: [
        {
          "@type": "EndpointProperty",
          name: "authorization",
          value: "TOKEN-ABCDEFG"
        },
        {
          "@type": "EndpointProperty",
          name: "authType",
          value: "bearer"
        }
      ]
    } : undefined


    if (!process.identifiers?.consumerPid || !process.identifiers?.providerPid) {
      console.error("Missing process identifiers");
      return;
    }

    await startAsync({
      data: {
        consumerPid: process.identifiers.consumerPid,
        providerPid: process.identifiers.providerPid,
        dataAddress: dataAddress ?? ({} as DataAddressDto)
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
      title="Transfer Start Dialog"
      description="You are about to start the transfer process."
      infoItems={mapTransferProcessToInfoItems(process)}
      submitLabel="Start"
      submitVariant="default"
      onSubmit={handleSubmit}
    />
  );
};
