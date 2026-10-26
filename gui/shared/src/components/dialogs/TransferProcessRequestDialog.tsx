import React, { useEffect, useState } from "react";
import { CatalogRpc, asArray, type DspNode } from "../../data/catalog";
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
import { Input } from "shared/src/components/ui/input";
import { Checkbox } from "shared/src/components/ui/checkbox";
import { useForm } from "react-hook-form";
import { BaseProcessDialog } from "./base";
import { urnInfoItem } from "./base/infoItemMappers";
import { InfoItemProps } from "../ui/info-list";
import {
  AgreementDto,
  DataAddressDto,
  TransferRPCRequestBody,
} from "../../data/orval/model";
import {
  useBffRpcSetupTransferRequest,
  useSetupTransferRequest,
} from "../../data/orval/transfer-rp-c/transfer-rp-c";
import { EUNOMIA_CONTEXT } from "../../data/eunomia";
import { useRpcSetupDatasetRequest } from "../../data/orval/catalog-rp-c/catalog-rp-c";
import { useMyWellKnownDSPPath, useParticipantDSPPath } from "../../hooks/useWellKnownUrl";
import { useNavigate } from "@tanstack/react-router";

type TransferRequestInputs = { distributionId: string; pushEndpointUrl: string };

export interface TransferProcessRequestDialogProps {
  process: AgreementDto;
  onClose?: () => void;
}

export const TransferProcessRequestDialog = ({ process }: TransferProcessRequestDialogProps) => {
  const navigate = useNavigate();
  const { mutateAsync: setupTransferRequest } = useSetupTransferRequest();
  const { mutateAsync: bffSetupTransferRequest } = useBffRpcSetupTransferRequest();
  const { mutateAsync: setupDatasetRequestAsync } = useRpcSetupDatasetRequest();
  const [distributions, setDistributions] = useState<DspNode[]>([]);
  const [autoStart, setAutoStart] = useState(true);
  const myDspPath = useMyWellKnownDSPPath();
  const { path: providerDspPath } = useParticipantDSPPath(process.providerParticipantId);

  const form = useForm<TransferRequestInputs>({
    defaultValues: { distributionId: "", pushEndpointUrl: "" },
  });

  const fetchDistributions = async () => {
    try {
      const res = await setupDatasetRequestAsync({
        data: CatalogRpc.dataset(process.providerParticipantId, process.target),
      });
      const dataset = res.status === 200 ? CatalogRpc.response(res.data) : undefined;
      setDistributions(asArray(dataset?.distribution));
    } catch (e) {
      console.error("Failed to fetch distributions", e);
    }
  };

  useEffect(() => {
    fetchDistributions();
  }, []);

  const handleSubmit = async ({ distributionId, pushEndpointUrl }: TransferRequestInputs) => {
    const distribution = distributions.find((d) => d["@id"] === distributionId);
    if (!distribution) return;

    const dataAddress: DataAddressDto | undefined = pushEndpointUrl
      ? {
          endpointType: "https://w3id.org/idsa/v4.1/HTTP",
          endpoint: pushEndpointUrl,
          endpointProperties: [
            { name: "authorization", value: "TOKEN-ABCDEFG" },
            { name: "authType", value: "bearer" },
          ],
        }
      : undefined;

    const payload: TransferRPCRequestBody = {
      "@context": EUNOMIA_CONTEXT,
      "@type": "RpcTransferRequestMessage",
      associatedAgentPeer: process.providerParticipantId,
      providerAddress: providerDspPath || "",
      callbackAddress: myDspPath || "",
      agreementId: process.id,
      // @ts-ignore
      format: distribution.format || "",
      dataAddress,
    };

    // Auto-start: the provider sends the TransferStart itself, REQUESTED -> STARTED.
    const res = autoStart
      ? await bffSetupTransferRequest({ data: payload })
      : await setupTransferRequest({ data: payload });

    if (res.status === 201) navigate({ to: "/transfer-process" });
  };

  return (
    <BaseProcessDialog<TransferRequestInputs>
      title="Transfer Request"
      description={
        <span className="max-w-full flex flex-wrap gap-1">
          Start transfer process for Agreement <Badge variant="info">{formatUrn(process.id)}</Badge>
        </span>
      }
      infoItems={[urnInfoItem("Dataset", process.target)].filter(Boolean) as InfoItemProps[]}
      formFields={
        <div className="grid grid-cols-2 gap-4">
          <FormField
            control={form.control}
            name="distributionId"
            render={({ field }) => (
              <FormItem className="flex flex-col gap-2">
                <label htmlFor="distributionId" className="text-sm -mt-2 mb-1 text-inherit">
                  Select distribution
                </label>
                <FormControl>
                  <Select
                    value={field.value || ""}
                    onValueChange={field.onChange}
                    onOpenChange={(open) => {
                      if (open) fetchDistributions();
                    }}
                  >
                    <SelectTrigger className="w-full">
                      <SelectValue placeholder="Select distribution" />
                    </SelectTrigger>
                    <SelectContent>
                      {distributions.map((d) => (
                        <SelectItem value={d["@id"] || ""} key={d["@id"]}>
                          {/* @ts-ignore */}
                          {d.format} - {d["@id"]}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </FormControl>
                <FormDescription>
                  By selecting distribution method you are choosing how the data will be
                  transferred.
                </FormDescription>
                <FormMessage />
              </FormItem>
            )}
          />
          <FormField
            control={form.control}
            name="pushEndpointUrl"
            render={({ field }) => (
              <FormItem className="flex flex-col gap-2">
                <label htmlFor="pushEndpointUrl" className="text-sm -mt-2 mb-1 text-inherit">
                  Push endpoint URL
                </label>
                <FormControl>
                  <Input
                    id="pushEndpointUrl"
                    type="url"
                    placeholder="https://your-endpoint.example.com"
                    {...field}
                  />
                </FormControl>
                <FormDescription>Optional. Fill only for push distributions.</FormDescription>
                <FormMessage />
              </FormItem>
            )}
          />
        </div>
      }
      submitLabel={autoStart ? "Request & Auto-start" : "Request Transfer"}
      submitVariant="default"
      onSubmit={handleSubmit}
      form={form}
      afterInfoContent={
        <label className="flex items-center gap-2 text-xs text-brand-snow/80 cursor-pointer select-none pt-2">
          <Checkbox
            id="autoStart"
            checked={autoStart}
            onCheckedChange={(checked) => setAutoStart(!!checked)}
          />
          Auto-start (Provider starts the transfer automatically)
        </label>
      }
    />
  );
};
