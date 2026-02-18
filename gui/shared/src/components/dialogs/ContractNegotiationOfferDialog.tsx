/**
 * ContractNegotiationOfferDialog.tsx
 *
 * Dialog for making a counter-offer in a contract negotiation.
 * Allows provider to modify policy terms via PolicyWrapperEdit.
 *
 * @example
 * <ContractNegotiationOfferDialog process={cnProcess} />
 */

import React, { useContext, useMemo, useState } from "react";
import { GlobalInfoContext, GlobalInfoContextType } from "shared/src/context/GlobalInfoContext";
import { useRpcSetupOffer } from "shared/src/data/orval/negotiation-rp-c/negotiation-rp-c";
import { useGetNegotiationProcesses, useGetOffersByProcessId } from "shared/src/data/orval/negotiations/negotiations";
import { NegotiationProcessDto } from "shared/src/data/orval/model/negotiationProcessDto";
import { OdrlOffer } from "shared/src/data/orval/model/odrlOffer";
import { OdrlInfo } from "shared/src/data/orval/model/odrlInfo";
import { PolicyWrapperEdit } from "../PolicyWrapperEdit";
import { BaseProcessDialog } from "./base";
import { mapCNProcessToInfoItemsForProvider } from "./base/infoItemMappers";
import Heading from "../ui/heading";
import { useRouter } from "@tanstack/react-router";
import { PolicyWrapperShow } from "../PolicyWrapperShow";

// =============================================================================
// TYPES
// =============================================================================

/**
 * Props for the ContractNegotiationOfferDialog component.
 */
export interface ContractNegotiationOfferDialogProps {
  /** The contract negotiation process */
  process: NegotiationProcessDto;
  /** Callback when the dialog is closed */
  onClose?: () => void;
}

// =============================================================================
// COMPONENT
// =============================================================================

/**
 * Dialog for making a contract negotiation counter-offer.
 *
 * Features:
 * - Displays current CN process information
 * - Allows policy modification via PolicyWrapperEdit
 * - Submits counter-offer to consumer
 */
export const ContractNegotiationOfferDialog = ({
  process,
  onClose,
}: ContractNegotiationOfferDialogProps) => {
  // ---------------------------------------------------------------------------
  // State & Hooks
  // ---------------------------------------------------------------------------

  /** Current edited policy state (declarative pattern) */
  const [currentPolicy, setCurrentPolicy] = useState<OdrlInfo | null>(null); // OdrlInfo matches OdrlOffer structure roughly
  const { refetch } = useGetNegotiationProcesses();
  const router = useRouter();
  const { mutateAsync: setupOffer } = useRpcSetupOffer();
  const lastOffer = useMemo(() => {
    return process.offers!.at(-1);
  }, [process]);

  // ---------------------------------------------------------------------------
  // Info Items
  // ---------------------------------------------------------------------------

  const infoItems = mapCNProcessToInfoItemsForProvider(process);

  // ---------------------------------------------------------------------------
  // Submit Handler
  // ---------------------------------------------------------------------------

  const handleSubmit = async () => {
    if (!process.id || !lastOffer || !currentPolicy) {
      console.error("Missing required data for offer");
      return;
    }

    await setupOffer({
      data: {
        processId: process.id,
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
      title="Contract Negotiation Offer"
      description="Make changes to the Contract Negotiation Offer and submit a counter-offer."
      infoItems={infoItems}
      afterInfoContent={policyEditorContent}
      submitLabel="Counter Offer"
      disabledSubmit={!process.id || !lastOffer || !currentPolicy}
      submitVariant="outline"
      onSubmit={handleSubmit}
      scrollable
      contentClassName="w-[75vw] max-w-[960px]"
    />
  );
};
