/**
 * BusinessRequestAccessDialog.tsx
 *
 * Dialog for requesting access to a dataset through a business request.
 * Displays dataset and policy information before submission.
 *
 * @example
 * <BusinessRequestAccessDialog
 *   policy={policyData}
 *   catalogId="catalog-123"
 *   datasetId="dataset-456"
 *   datasetName="My Dataset"
 * />
 */

import React, { useContext, useRef } from "react";
import { GlobalInfoContext, GlobalInfoContextType } from "shared/src/context/GlobalInfoContext";
//import { usePostNewBusinessRequest } from "shared/src/data/business-mutations";
import { AuthContext, AuthContextType } from "shared/src/context/AuthContext";
import { PolicyWrapperShow } from "shared/src/components/PolicyWrapperShow";
import { BaseProcessDialog } from "./base";
import { urnInfoItem, textInfoItem } from "./base/infoItemMappers";
import { InfoItemProps } from "../ui/info-list";
import { Badge } from "../ui/badge";
import { formatUrn } from "shared/src/lib/utils";
import { DialogClose } from "../ui/dialog";

// =============================================================================
// TYPES
// =============================================================================

/**
 * Props for the BusinessRequestAccessDialog component.
 */
export interface BusinessRequestAccessDialogProps {
  /** The ODRL policy to request access under */
  policy: OdrlOffer;

  /** ID of the parent catalog */
  catalogId?: UUID;

  /** ID of the dataset to request access to */
  datasetId?: UUID;

  /** Display name of the dataset */
  datasetName?: string;
}

// =============================================================================
// COMPONENT
// =============================================================================

/**
 * Dialog for requesting dataset access through a business request.
 *
 * Features:
 * - Displays dataset and catalog information
 * - Shows policy details via PolicyWrapperShow
 * - Handles business request submission
 */
export const BusinessRequestAccessDialog = ({
  policy,
  catalogId,
  datasetId,
  datasetName,
}: BusinessRequestAccessDialogProps) => {
  const closeDialogRef = useRef<HTMLButtonElement>(null);
  //const { mutateAsync: requestAsync } = usePostNewBusinessRequest();
  const { api_gateway } = useContext<GlobalInfoContextType | null>(GlobalInfoContext)!;
  const { participant } = useContext<AuthContextType | null>(AuthContext)!;

  // ---------------------------------------------------------------------------
  // Info Items
  // ---------------------------------------------------------------------------

  const infoItems: InfoItemProps[] = [
    textInfoItem("Dataset", datasetName),
    urnInfoItem("Catalog ID", catalogId),
    urnInfoItem("Policy ID", policy["@id"]),
  ].filter((item): item is InfoItemProps => item !== undefined);

  // ---------------------------------------------------------------------------
  // Submit Handler
  // ---------------------------------------------------------------------------

  const handleSubmit = async () => {
    // await requestAsync({
    //   api_gateway,
    //   content: {
    //     consumerParticipantId: participant?.participant_id!,
    //     offer: {
    //       "@id": policy["@id"],
    //     },
    //   },
    // });
    // closeDialogRef.current?.click();
  };

  // ---------------------------------------------------------------------------
  // After Info Content (Policy Display)
  // ---------------------------------------------------------------------------

  const policyContent = (
    <div className="pt-4">
      <PolicyWrapperShow
        policy={policy}
        datasetId={undefined}
        catalogId={undefined}
        participant={undefined}
        datasetName=""
      />
      {/* Hidden close button for programmatic dialog close */}
      <DialogClose ref={closeDialogRef} className="hidden" />
    </div>
  );

  // ---------------------------------------------------------------------------
  // Render
  // ---------------------------------------------------------------------------

  return (
    <BaseProcessDialog
      title="Request Dataset Access"
      description={
        <span className="max-w-full flex flex-col gap-1">
          You are about to request access to dataset{" "}
          <Badge variant="infoLighter">{datasetName}</Badge> in catalog{" "}
          <Badge variant="infoLighter">{formatUrn(catalogId)}</Badge> under policy{" "}
          <Badge variant="infoLighter">{formatUrn(policy["@id"])}</Badge>
        </span>
      }
      infoItems={infoItems}
      afterInfoContent={policyContent}
      submitLabel="Request Access"
      submitVariant="default"
      onSubmit={handleSubmit}
      scrollable
    />
  );
};
