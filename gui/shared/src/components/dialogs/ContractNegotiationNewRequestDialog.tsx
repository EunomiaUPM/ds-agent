/**
 * ContractNegotiationNewRequestDialog.tsx
 *
 * Dialog for creating a new contract negotiation request.
 * Displays policy information and initiates a new CN process.
 *
 * @example
 * <ContractNegotiationNewRequestDialog
 *   policy={policyData}
 *   catalogId="catalog-123"
 *   datasetId="dataset-456"
 *   participantId="provider-participant-id"
 * />
 */

import React, { useRef } from "react";
import { PolicyWrapperShow } from "shared/src/components/PolicyWrapperShow";
import { BaseProcessDialog } from "./base";
import { urnInfoItem } from "./base/infoItemMappers";
import { InfoItemProps } from "../ui/info-list";
import { useRpcSetupRequestInit } from "../../data/orval/negotiation-rp-c/negotiation-rp-c";
import { useNavigate } from "@tanstack/react-router";
import { OdrlOffer } from "../../data/orval/model";
import { useMyWellKnownDSPPath, useParticipantDSPPath } from "../../hooks/useWellKnownUrl";

// =============================================================================
// TYPES
// =============================================================================

/**
 * Props for the ContractNegotiationNewRequestDialog component.
 */
export interface ContractNegotiationNewRequestDialogProps {
  /** The ODRL policy to negotiate */
  policy: OdrlOffer;

  /** ID of the parent catalog */
  catalogId: string;

  /** ID of the dataset */
  datasetId: string;

  /** Provider participant ID to negotiate with */
  participantId: string;
}

// =============================================================================
// COMPONENT
// =============================================================================

/**
 * Dialog for initiating a new contract negotiation request.
 *
 * Features:
 * - Displays catalog and dataset information
 * - Shows policy details via PolicyWrapperShow
 * - Initiates CN request with provider
 */
export const ContractNegotiationNewRequestDialog = ({
  policy,
  catalogId,
  datasetId,
  participantId,
}: ContractNegotiationNewRequestDialogProps) => {
  const navigate = useNavigate();
  const { mutateAsync: requestAsync } = useRpcSetupRequestInit();
  const myDspPath = useMyWellKnownDSPPath();
  const { path: providerDspPath } = useParticipantDSPPath(participantId);

  // ---------------------------------------------------------------------------
  // Info Items
  // ---------------------------------------------------------------------------

  const infoItems: InfoItemProps[] = [
    urnInfoItem("Catalog ID", catalogId),
    urnInfoItem("Dataset ID", datasetId),
    urnInfoItem("Provider", participantId),
  ].filter((item): item is InfoItemProps => item !== undefined);

  // ---------------------------------------------------------------------------
  // Submit Handler
  // ---------------------------------------------------------------------------

  const handleSubmit = async () => {
    if (!myDspPath || !providerDspPath) {
      return;
    }

    const res = await requestAsync({
      data: {
        associatedAgentPeer: participantId,
        providerAddress: providerDspPath,
        callbackAddress: myDspPath,
        offer: {
          ...policy,
          target: datasetId,
        },
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
  // After Info Content (Policy Display)
  // ---------------------------------------------------------------------------

  const policyContent = (
    <div className="pt-4">
      <PolicyWrapperShow
        policy={policy}
        datasetId={datasetId}
        catalogId={catalogId}
        participant={participantId}
        datasetName=""
      />
    </div>
  );

  // ---------------------------------------------------------------------------
  // Render
  // ---------------------------------------------------------------------------

  return (
    <BaseProcessDialog
      title="Contract Negotiation Request"
      description="You are about to request a contract negotiation for the selected dataset and policy."
      infoItems={infoItems}
      afterInfoContent={policyContent}
      submitLabel="Request Contract Negotiation"
      submitVariant="default"
      onSubmit={handleSubmit}
      scrollable
    />
  );
};
