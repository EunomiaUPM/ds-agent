/**
 * PolicyWrapperShow.tsx
 *
 * Read-only display component for ODRL policy details.
 * Shows policy metadata, target information, and ODRL rules
 * (permissions, prohibitions, obligations).
 *
 * Features:
 * - Policy ID badge with formatted URN
 * - Policy target and entity information
 * - ODRL content visualization for all rule types
 * - Context-aware actions (delete for providers, request access for consumers)
 *
 * @example
 * <PolicyWrapperShow
 *   policy={policyData}
 *   datasetId="dataset-123"
 *   catalogId="catalog-456"
 *   participant={currentParticipant}
 *   datasetName="My Dataset"
 * />
 */

import React from "react";
import { formatUrn } from "shared/src/lib/utils";
import { InfoList } from "shared/src/components/ui/info-list";
import Heading from "shared/src/components/ui/heading";
import { Badge } from "shared/src/components/ui/badge";
import PolicyComponent from "shared/src/components/PolicyComponent";
import { Trash, Shield } from "lucide-react";
import { Button } from "./ui/button";
import { useRouterState } from "@tanstack/react-router";
import { BusinessRemovePolicyDialog } from "./dialogs/BusinessRemovePolicyDialog";
import { Dialog, DialogTrigger } from "shared/src/components/ui/dialog";
import { ContractNegotiationNewRequestDialog } from "./dialogs/ContractNegotiationNewRequestDialog";
import { OdrlOffer, OdrlPolicyDto } from "../data/orval/model";
import { ContractNegotiationNewOfferDialog } from "./dialogs/ContractNegotiationNewOfferDialog";
import {
  Accordion,
  AccordionContent,
  AccordionItem,
  AccordionTrigger,
} from "shared/src/components/ui/accordion";

// =============================================================================
// TYPES
// =============================================================================

/**
 * Props for the PolicyWrapperShow component.
 */
export interface PolicyWrapperShowProps {
  /** The ODRL policy offer to display */
  policy: OdrlOffer | OdrlPolicyDto;

  /** ID of the parent dataset (for actions) */
  datasetId?: string;

  /** ID of the parent catalog (for actions) */
  catalogId?: string;

  /** Name of the dataset (for display in dialogs) */
  datasetName?: string;

  /** Whether to show the Request Access button (default: false) */
  showRequestAccess?: boolean;

  /** Whether to show the Offer Access button (default: false) */
  showOfferAccess?: boolean;

  /** The participant ID of the provider (for negotiation request) */
  participant?: string;

  /** Whether to collapse the ODRL Content section inside an accordion (default: true) */
  showOfferHidden?: boolean;
}

// =============================================================================
// COMPONENT
// =============================================================================

/**
 * Displays policy details in a read-only card format.
 *
 * The component adapts its behavior based on:
 * - Current route (shows different actions on datahub-catalog pages)
 *
 * @param props - PolicyWrapperShow properties
 * @returns A styled policy display card
 */
export const PolicyWrapperShow = ({
  policy,
  datasetId,
  catalogId,
  datasetName,
  showRequestAccess = false,
  showOfferAccess = false,
  participant,
  showOfferHidden = true,
}: PolicyWrapperShowProps) => {
  const routerState = useRouterState();

  // ---------------------------------------------------------------------------
  // Computed Values
  // ---------------------------------------------------------------------------

  /** Whether we're viewing a dataset in the datahub catalog */
  const isDatahubDatasetView =
    routerState.location.pathname.includes("datahub-catalog") &&
    routerState.location.pathname.includes("dataset");

  // @ts-ignore
  const policyId = "id" in policy ? policy.id : policy["@id"];
  const odrlOffer = "odrlOffer" in policy ? policy.odrlOffer : (policy as OdrlOffer);
  const entityType = "entityType" in policy ? policy.entityType : (policy as any).entityType; // Fallback for flexibility
  const entity =
    ("entity" in policy ? policy.entity : (policy as any).entity || (policy as any).target) ??
    datasetId;
  const description = ("description" in policy ? (policy as any).description : undefined) as
    | string
    | undefined;

  // ---------------------------------------------------------------------------
  // Render
  // ---------------------------------------------------------------------------

  const descriptionText = description ? description : "No description available for this policy.";
  const generatedTitle = descriptionText
    .split(" ")
    .slice(0, 3)
    .map(word => word.charAt(0).toUpperCase() + word.slice(1))
    .join(" ");

  return (
    <div className="w-full h-full">
      <div className="h-full flex flex-col items-start justify-between border border-white/10 bg-background-300/20 p-4 rounded-lg">
        <div className="title-description-container w-full">  {/* Header: Title and actions */}
          <div className="flex justify-between items-start w-full mb-2">
            <Heading level="h4" className="flex items-center gap-2 font-bold text-white tracking-tight mb-1">

              <Shield
                className="h-[22px] w-[22px] text-primary-500"

              />
              {generatedTitle}
            </Heading>
            {/* Provider action: Delete policy */}
            {isDatahubDatasetView && (
              <Dialog>
                <DialogTrigger asChild>
                  <Button
                    variant="ghost"
                    size="icon"
                    className="h-6 w-6 text-muted-foreground hover:text-destructive transition-colors"
                  >
                    <Trash className="h-4 w-4" />
                  </Button>
                </DialogTrigger>
                <BusinessRemovePolicyDialog
                  policy={policy as any} // Cast as any for now, or ensure compatibility
                  catalogId={catalogId}
                  datasetId={datasetId}
                />
              </Dialog>
            )}
          </div>

          <p className="text-sm text-white/70 mb-6 leading-5"> {descriptionText} </p>
        </div>
        {/* ODRL Content section */}
        <div className="policies-requestButton-container w-full">
          {showOfferHidden ? (
            <Accordion type="single" collapsible className="w-full">
              <AccordionItem value="odrl-content" className="border border-white/20 rounded-md overflow-hidden bg-transparent">
                <AccordionTrigger className="px-3 py-3 hover:bg-background text-[12px] font-medium text-white uppercase tracking-wider transition-colors [&[data-state=open]]:border-b [&[data-state=open]]:border-white/20 hover:no-underline rounded-t-md">
                  Permissions, Obligations, Prohibitions
                </AccordionTrigger>
                <AccordionContent className="p-0 m-2">
                  <div className="flex flex-col gap-2 w-full">
                    <PolicyComponent
                      policyItem={odrlOffer?.permission || (policy as any).permission}
                      variant="permission"
                    />
                    <PolicyComponent
                      policyItem={odrlOffer?.obligation || (policy as any).obligation}
                      variant="obligation"
                    />
                    <PolicyComponent
                      policyItem={odrlOffer?.prohibition || (policy as any).prohibition}
                      variant="prohibition"
                    />
                  </div>
                </AccordionContent>
              </AccordionItem>
            </Accordion>
          ) : (
            <div className="w-full border border-white/20 rounded-md overflow-hidden bg-transparent">
              <div className="px-4 py-3 bg-background-800 border-b border-white/20">
                <Heading level="h6" className="text-[13px] font-bold text-white uppercase tracking-wider mb-0">
                  Permissions, Obligations and Prohibitions
                </Heading>
              </div>
              <div className="flex flex-col w-full bg-transparent">
                <PolicyComponent
                  policyItem={odrlOffer?.permission || (policy as any).permission}
                  variant="permission"
                />
                <PolicyComponent
                  policyItem={odrlOffer?.obligation || (policy as any).obligation}
                  variant="obligation"
                />
                <PolicyComponent
                  policyItem={odrlOffer?.prohibition || (policy as any).prohibition}
                  variant="prohibition"
                />
              </div>
            </div>
          )}

          {/* Consumer action: Request access */}
          {showRequestAccess && odrlOffer && (
            <div className="mt-4 w-full flex justify-end">
              <Dialog>
                <DialogTrigger asChild>
                  <Button size="sm" variant="default" className="w-full sm:w-auto">
                    Request Dataset Access
                  </Button>
                </DialogTrigger>
                <ContractNegotiationNewRequestDialog
                  policy={odrlOffer}
                  catalogId={catalogId || ""}
                  datasetId={datasetId || ""}
                  participantId={participant || ""}
                />
              </Dialog>
            </div>
          )}

          {/* Consumer action: Request access */}
          {showOfferAccess && odrlOffer && (
            <div className="mt-4 w-full flex justify-end">
              <Dialog>
                <DialogTrigger asChild>
                  <Button size="sm" variant="default" className="w-full sm:w-auto">
                    Offer Access to dataset
                  </Button>
                </DialogTrigger>
                <ContractNegotiationNewOfferDialog
                  policy={policy as OdrlPolicyDto}
                  catalogId={catalogId || ""}
                  datasetId={datasetId || ""}
                />
              </Dialog>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};
