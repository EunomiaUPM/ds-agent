/**
 * TransferProcessRequestDialog.tsx
 *
 * Dialog for initiating a new transfer process request.
 * Allows users to select transfer method (PULL/PUSH) and protocol (http/kafka/ftp).
 *
 * @example
 * <TransferProcessRequestDialog agreement={agreementData} />
 */

import React, { useContext, useEffect, useState } from "react";
import { formatUrn } from "shared/src/lib/utils";
import { Badge } from "shared/src/components/ui/badge";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "shared/src/components/ui/select";
import {
  FormControl,
  FormDescription,
  FormField,
  FormItem,
  FormMessage,
} from "shared/src/components/ui/form";
import { useForm } from "react-hook-form";
import { GlobalInfoContext, GlobalInfoContextType } from "shared/src/context/GlobalInfoContext";
import { BaseProcessDialog } from "./base";
import { urnInfoItem } from "./base/infoItemMappers";
import { InfoItemProps } from "../ui/info-list";
import { AgreementDto, Distribution } from "../../data/orval/model";
import { useSetupTransferRequest } from "../../data/orval/transfer-rp-c/transfer-rp-c";
import { useGetPeerCatalog } from "../../data/orval/catalogs/catalogs";
import { useRpcSetupDatasetRequest } from "../../data/orval/catalog-rp-c/catalog-rp-c";
import { useMyWellKnownDSPPath, useParticipantDSPPath } from "../../hooks/useWellKnownUrl";

// =============================================================================
// TYPES
// =============================================================================

/**
 * Form input values for transfer request.
 */
type TransferRequestInputs = {
  distributionId: string;
};

/**
 * Props for the TransferProcessRequestDialog component.
 */
export interface TransferProcessRequestDialogProps {
  /** The agreement to start a transfer for */
  process: AgreementDto;
  onClose?: () => void;
}

// =============================================================================
// COMPONENT
// =============================================================================

/**
 * Dialog for requesting a new transfer process.
 *
 * Features:
 * - Method selection (PULL/PUSH)
 * - Protocol selection (HTTP/Kafka/FTP)
 * - Agreement information display
 */
export const TransferProcessRequestDialog = ({ process, onClose }: TransferProcessRequestDialogProps) => {
  const { mutateAsync } = useSetupTransferRequest();
  const { mutateAsync: setupDatasetRequestAsync } = useRpcSetupDatasetRequest()
  const [selectableDistributions, setSelectableDistributions] = useState<Distribution[]>([]);
  const myDspPath = useMyWellKnownDSPPath();
  const { path: providerDspPath } = useParticipantDSPPath(process.providerParticipantId);

  // Form with default values
  const form = useForm<TransferRequestInputs>({
    defaultValues: {
      distributionId: "",
    },
  });

  useEffect(() => {
    fetchDistributions();
  }, []);

  // ---------------------------------------------------------------------------
  // Info Items
  // ---------------------------------------------------------------------------

  const infoItems: InfoItemProps[] = [
    urnInfoItem("Dataset", process.target),
  ].filter((item): item is InfoItemProps => item !== undefined);

  // ---------------------------------------------------------------------------
  // Submit Handler
  // ---------------------------------------------------------------------------

  const handleSubmit = async (data: TransferRequestInputs) => {
    const distribution = selectableDistributions.find(d => d["@id"] === data.distributionId);
    if (!distribution) return;

    await mutateAsync({
      data: {
        associatedAgentPeer: process.providerParticipantId,
        providerAddress: providerDspPath,
        callbackAddress: myDspPath,
        agreementId: process.id,
        // @ts-ignore - formats is present in the data but not in the model
        format: distribution.formats || "http+pull",
      },
    });
  };

  const fetchDistributions = async () => {
    try {
      const datasetResponse = await setupDatasetRequestAsync({
        data: {
          associatedAgentPeer: process.providerParticipantId,
          dataset: process.target,
        },
      });
      console.log("TransferProcessRequestDialog: Fetching distributions", datasetResponse);
      const data = datasetResponse.status === 200 ? datasetResponse.data : undefined;
      // @ts-ignore - distribution is part of the Dataset model
      const distributions = data?.distribution || data?.response?.distribution || [];
      setSelectableDistributions(distributions);
    } catch (error) {
      console.error("TransferProcessRequestDialog: Failed to fetch distributions", error);
    }
  };

  // ---------------------------------------------------------------------------
  // Form Fields
  // ---------------------------------------------------------------------------

  const formFieldsContent = (
    <div className="grid grid-cols-2 gap-4">
      <FormField
        control={form.control}
        name="distributionId"
        render={({ field }) => (
          <FormItem className="flex flex-col gap-2">
            <label htmlFor="distributionId" className="text-sm -mt-2 mb-1 text-inherit">
              Select distribution
            </label>
            <FormControl >
              <Select
                value={field.value}
                onValueChange={field.onChange}
                onOpenChange={(open) => {
                  if (open) fetchDistributions();
                }}
              >
                <SelectTrigger className="w-full">
                  <SelectValue placeholder="Select distribution" />
                </SelectTrigger>
                <SelectContent>
                  {selectableDistributions.map((distribution) => (
                    <SelectItem value={distribution["@id"] || ""} key={distribution["@id"]}>
                      {/* @ts-ignore - formats is present in the data but not in the model */}
                      {distribution.formats} - {distribution["@id"]}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </FormControl>
            <FormDescription>
              By selecting distribution method you are choosing how the data will be transferred. The direction, protocol and other parameters will be set automatically by the agent.
            </FormDescription>
            <FormMessage />
          </FormItem>
        )
        }
      />
    </div >
  );

  // ---------------------------------------------------------------------------
  // Render
  // ---------------------------------------------------------------------------

  return (
    <BaseProcessDialog<TransferRequestInputs>
      title="Transfer Request"
      description={
        <span className="max-w-full flex flex-wrap gap-1">
          Start transfer process for Agreement{" "}
          <Badge variant="info">{formatUrn(process.id)}</Badge>
        </span>
      }
      infoItems={infoItems}
      formFields={formFieldsContent}
      submitLabel="Request Transfer"
      submitVariant="default"
      onSubmit={handleSubmit}
      form={form}
    />
  );
};
