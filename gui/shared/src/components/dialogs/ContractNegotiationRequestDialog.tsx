/**
 * ContractNegotiationRequestDialog.tsx
 *
 * Dialog for making a contract negotiation request/counter-request.
 * Allows consumer to modify policy terms via PolicyWrapperEdit.
 *
 * @example
 * <ContractNegotiationRequestDialog process={cnProcess} />
 */

import React, { useContext, useMemo, useState } from "react";
import { GlobalInfoContext, GlobalInfoContextType } from "shared/src/context/GlobalInfoContext";
import { PolicyWrapperEdit } from "../PolicyWrapperEdit";
import { BaseProcessDialog } from "./base";
import { mapCNProcessToInfoItemsForConsumer } from "./base/infoItemMappers";
import Heading from "../ui/heading";
import { OdrlInfo, NegotiationProcessDto } from "../../data/orval/model";
import { useRpcSetupRequest, useRpcSetupRequestInit } from "../../data/orval/negotiation-rp-c/negotiation-rp-c";
import { useGetNegotiationProcesses } from "../../data/orval/negotiations/negotiations";
import { useRouter } from "@tanstack/react-router";
import { PolicyWrapperShow } from "../PolicyWrapperShow";

// =============================================================================
// TYPES
// =============================================================================

/**
 * Props for the ContractNegotiationRequestDialog component.
 */
export interface ContractNegotiationRequestDialogProps {
  /** The contract negotiation process */
  process: NegotiationProcessDto;
  /** Callback when the dialog is closed */
  onClose?: () => void;
}

// =============================================================================
// COMPONENT
// =============================================================================

/**
 * Dialog for making a contract negotiation request/counter-request.
 *
 * Features:
 * - Displays current CN process information
 * - Allows policy modification via PolicyWrapperEdit
 * - Submits request/counter-request to provider
 */
export const ContractNegotiationRequestDialog = ({
  process,
  onClose,
}: ContractNegotiationRequestDialogProps) => {
  // ---------------------------------------------------------------------------
  // State & Hooks
  // ---------------------------------------------------------------------------

  /** Current edited policy state (declarative pattern) */
  const { refetch } = useGetNegotiationProcesses();
  const router = useRouter();
  const [currentPolicy, setCurrentPolicy] = useState<OdrlInfo | null>(null);
  const { mutateAsync: dataRequestAsync } = useRpcSetupRequest();
  const lastOffer = useMemo(() => {
    return process.offers?.at(-1);
  }, [process]);

  // ---------------------------------------------------------------------------
  // Info Items
  // ---------------------------------------------------------------------------

  const infoItems = mapCNProcessToInfoItemsForConsumer(process);

  // ---------------------------------------------------------------------------
  // Submit Handler
  // ---------------------------------------------------------------------------

  const handleSubmit = async () => {
    if (!process.identifiers?.consumerPid || !process.identifiers?.providerPid || !lastOffer || !currentPolicy) {
      console.error("Missing required data for request");
      return;
    }

    await dataRequestAsync({
      data: {
        consumerPid: process.identifiers.consumerPid,
        providerPid: process.identifiers.providerPid,
        offer: {
          "@id": (lastOffer.offerContent["@id"] as string) ?? "urn:uuid:placeholder",
          "@type": (lastOffer.offerContent["@type"] as any),
          target: (lastOffer.offerContent.target as string) ?? "",
          ...currentPolicy
        },
      },
    });
    await refetch();
    router.invalidate();
    if (onClose) {
      onClose();
    }

  };

  // ---------------------------------------------------------------------------
  // After Info Content (Policy Editor)
  // ---------------------------------------------------------------------------

  const policyEditorContent = lastOffer && lastOffer.offerContent ? (
    <div className="pt-4 flex gap-2">
      <div className="w-1/2">
        <Heading level="h6" className="mb-2">
          Current Policy
        </Heading>
        <PolicyWrapperShow policy={lastOffer.offerContent} />
      </div>
      <div className="w-1/2">
        <Heading level="h6" className="mb-2">
          New Policy Request
        </Heading>
        <PolicyWrapperEdit policy={lastOffer.offerContent} onChange={setCurrentPolicy} />
      </div>
    </div>
  ) : null;

  // ---------------------------------------------------------------------------
  // Render
  // ---------------------------------------------------------------------------

  return (
    <BaseProcessDialog
      title="Contract Negotiation Request"
      description="Make changes to the Contract Negotiation Request and submit a counter-request."
      infoItems={infoItems}
      afterInfoContent={policyEditorContent}
      submitLabel="Request"
      disabledSubmit={!process.identifiers?.consumerPid || !process.identifiers?.providerPid || !lastOffer || !currentPolicy}
      submitVariant="outline"
      onSubmit={handleSubmit}
      scrollable
      contentClassName="w-[75vw] max-w-[960px]"
    />
  );
};
