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
import { useRpcSetupOffer, useRpcSetupOfferInit } from "shared/src/data/orval/negotiation-rp-c/negotiation-rp-c";
import { useGetNegotiationProcesses, useGetOffersByProcessId } from "shared/src/data/orval/negotiations/negotiations";
import { NegotiationProcessDto } from "shared/src/data/orval/model/negotiationProcessDto";
import { OdrlOffer } from "shared/src/data/orval/model/odrlOffer";
import { OdrlInfo } from "shared/src/data/orval/model/odrlInfo";
import { PolicyWrapperEdit } from "../PolicyWrapperEdit";
import { BaseProcessDialog } from "./base";
import { mapCNProcessToInfoItemsForProvider, urnInfoItem } from "./base/infoItemMappers";
import Heading from "../ui/heading";
import { useNavigate, useRouter } from "@tanstack/react-router";
import { PolicyWrapperShow } from "../PolicyWrapperShow";
import { InfoItemProps } from "../ui/info-list";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "../ui/select";
import { useGetAllParticipants } from "../../data/orval/participants/participants";
import { OdrlPolicyDto, ParticipantDto } from "../../data/orval/model";
import { useMyWellKnownDSPPath, useParticipantDSPPath } from "../../hooks/useWellKnownUrl";

// =============================================================================
// TYPES
// =============================================================================

/**
 * Props for the ContractNegotiationOfferDialog component.
 */
export interface ContractNegotiationNewOfferDialogProps {
  /** The ODRL policy to negotiate */
  policy: OdrlPolicyDto;

  /** ID of the parent catalog */
  catalogId: string;

  /** ID of the dataset */
  datasetId: string;

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
export const ContractNegotiationNewOfferDialog = ({
  policy,
  catalogId,
  datasetId,
  onClose,
}: ContractNegotiationNewOfferDialogProps) => {
  // ---------------------------------------------------------------------------
  // State & Hooks
  // ---------------------------------------------------------------------------

  /** Current edited policy state (declarative pattern) */
  const navigate = useNavigate();
  const [currentPolicy, setCurrentPolicy] = useState<OdrlInfo | null>(null); // OdrlInfo matches OdrlOffer structure roughly
  const { refetch } = useGetNegotiationProcesses();
  const router = useRouter();
  const { mutateAsync: setupOffer } = useRpcSetupOfferInit();
  const { data: participants, refetch: refetchParticipants } = useGetAllParticipants();
  const [selectedParticipant, setSelectedParticipant] = useState<ParticipantDto | null>(null);
  const myDspPath = useMyWellKnownDSPPath();
  const { resolve: resolveProviderDspPath } = useParticipantDSPPath(undefined);

  // ---------------------------------------------------------------------------
  // Info Items
  // ---------------------------------------------------------------------------

  const infoItems: InfoItemProps[] = [
    urnInfoItem("Catalog ID", catalogId),
    urnInfoItem("Dataset ID", datasetId),
  ].filter((item): item is InfoItemProps => item !== undefined);

  // ---------------------------------------------------------------------------
  // Helpers
  // ---------------------------------------------------------------------------

  const sanitizePolicy = (policy: OdrlOffer): OdrlOffer => {
    const p = { ...policy };
    if (p.permission && p.permission.length === 0) p.permission = undefined;
    if (p.prohibition && p.prohibition.length === 0) p.prohibition = undefined;
    if (p.obligation && p.obligation.length === 0) p.obligation = undefined;
    return p;
  };

  // ---------------------------------------------------------------------------
  // Submit Handler
  // ---------------------------------------------------------------------------

  const handleSubmit = async () => {
    const resolvedPath = selectedParticipant?.participant_id
      ? await resolveProviderDspPath(selectedParticipant.participant_id)
      : null;

    console.log("policy", policy);

    const res = await setupOffer({
      data: {
        associatedAgentPeer: selectedParticipant?.participant_id || "",
        providerAddress: resolvedPath || "", // Use resolved path
        callbackAddress: myDspPath || "",
        offer: {
          "@id": policy.id
        }
      },
    });
    if (res.status === 201) {
      // closeDialogRef.current?.click();
      navigate({
        to: "/contract-negotiation",
      });
    }

  };

  // ---------------------------------------------------------------------------
  // Load Participants
  // ---------------------------------------------------------------------------

  const loadParticipants = async () => {
    await refetchParticipants();
  };


  const handleParticipantChange = (value: string) => {
    if (participants && Array.isArray(participants.data)) {
      const selected = participants.data.find((p) => p.participant_id === value);
      if (selected) {
        setSelectedParticipant(selected);
      }
    }
  };

  // ---------------------------------------------------------------------------
  // After Info Content (Policy show and Participant selector)
  // ---------------------------------------------------------------------------

  const policyContent = (
    <div>
      <div className="pt-4">
        <Heading level="h6" className="mb-2">
          Select an authenticated peer to send the offer to.
        </Heading>
        <Select onValueChange={handleParticipantChange} value={selectedParticipant?.participant_id} onOpenChange={loadParticipants}>
          <SelectTrigger>
            <SelectValue placeholder="Select a peer" />
          </SelectTrigger>
          <SelectContent>
            {Array.isArray(participants?.data) && participants.data
              .filter((participant) => participant.participant_id !== "Agent")
              .filter((participant) => !participant.is_me)
              .map((participant) => (
                <SelectItem key={participant.participant_id} value={participant.participant_id || ""}>
                  {participant.participant_slug || participant.participant_id} - {participant.base_url}
                </SelectItem>
              ))}
          </SelectContent>
        </Select>
      </div>

      <div className="pt-4">
        <Heading level="h6" className="mb-2">
          Current offer
        </Heading>
        <PolicyWrapperShow
          policy={policy}
          datasetId={datasetId}
          catalogId={catalogId}
          datasetName=""
        />
      </div>
    </div>
  );

  // ---------------------------------------------------------------------------
  // Render
  // ---------------------------------------------------------------------------

  return (
    <BaseProcessDialog
      title="Contract Negotiation Offer"
      description="Send a Contract Negotiation Offer."
      infoItems={infoItems}
      afterInfoContent={policyContent}
      submitLabel="Send Offer"
      submitVariant="outline"
      onSubmit={handleSubmit}
      scrollable
      disabledSubmit={!selectedParticipant}
    />
  );
};

