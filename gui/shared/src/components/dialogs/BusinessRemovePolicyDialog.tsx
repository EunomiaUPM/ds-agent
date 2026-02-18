/**
 * BusinessRemovePolicyDialog.tsx
 *
 * Confirmation dialog for removing a policy from a dataset.
 * Uses BaseProcessDialog for consistent structure.
 *
 * @example
 * <BusinessRemovePolicyDialog
 *   policy={policy}
 *   catalogId="catalog-123"
 *   datasetId="dataset-456"
 * />
 */

import React, { useContext } from "react";
import { GlobalInfoContext, GlobalInfoContextType } from "shared/src/context/GlobalInfoContext";
//import { useDeleteBusinessNewPolicyInDataset } from "shared/src/data/business-mutations";
import { BaseProcessDialog } from "./base";
import { urnInfoItem } from "./base/infoItemMappers";
import { InfoItemProps } from "../ui/info-list";
import { OdrlOffer } from "../../data/orval/model";

// =============================================================================
// TYPES
// =============================================================================

/**
 * Props for the BusinessRemovePolicyDialog component.
 */
export interface BusinessRemovePolicyDialogProps {
  /** The ODRL policy to remove */
  policy: OdrlOffer;

  /** ID of the parent catalog */
  catalogId?: string;

  /** ID of the parent dataset */
  datasetId?: string;
}

// =============================================================================
// COMPONENT
// =============================================================================

/**
 * Confirmation dialog for deleting a policy from a dataset.
 *
 * Features:
 * - Displays policy information for confirmation
 * - Destructive action styling
 * - Handles deletion via business mutations API
 */
export const BusinessRemovePolicyDialog = ({
  policy,
  catalogId,
  datasetId,
}: BusinessRemovePolicyDialogProps) => {
  //const { mutateAsync: deleteAsync } = useDeleteBusinessNewPolicyInDataset();
  const { api_gateway } = useContext<GlobalInfoContextType | null>(GlobalInfoContext)!;

  // ---------------------------------------------------------------------------
  // Info Items
  // ---------------------------------------------------------------------------

  const infoItems: InfoItemProps[] = [
    urnInfoItem("Policy ID", policy["@id"]),
    urnInfoItem("Catalog ID", catalogId),
    urnInfoItem("Dataset ID", datasetId),
  ].filter((item): item is InfoItemProps => item !== undefined);

  // ---------------------------------------------------------------------------
  // Submit Handler
  // ---------------------------------------------------------------------------

  const handleSubmit = async () => {
    // await deleteAsync({
    //   api_gateway,
    //   catalogId: catalogId!,
    //   datasetId: datasetId!,
    //   policyId: policy["@id"],
    // });
  };

  // ---------------------------------------------------------------------------
  // Render
  // ---------------------------------------------------------------------------

  return (
    <BaseProcessDialog
      title="Delete Policy"
      description="Are you sure you want to delete this policy? This action cannot be undone."
      infoItems={infoItems}
      submitLabel="Remove Policy"
      submitVariant="destructive"
      onSubmit={handleSubmit}
    />
  );
};
