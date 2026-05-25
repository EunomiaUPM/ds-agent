import React, { useEffect, useState } from "react";
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
import { BaseProcessDialog } from "./base";
import { urnInfoItem } from "./base/infoItemMappers";
import { InfoItemProps } from "../ui/info-list";
import { AgreementDto, DataAddressDto, Distribution } from "../../data/orval/model";
import { useRpcSetupDatasetRequest } from "../../data/orval/catalog-rp-c/catalog-rp-c";
import { useMyWellKnownDSPPath, useParticipantDSPPath } from "../../hooks/useWellKnownUrl";
import { useNavigate } from "@tanstack/react-router";
import { useMutation } from "@tanstack/react-query";
import axios from "axios";

type TransferRequestInputs = { distributionId: string; pushEndpointUrl: string };

export interface TransferProcessBusinessRequestDialogProps {
  process: AgreementDto;
  onClose?: () => void;
}

const useBffSetupTransferRequest = () =>
  useMutation({
    mutationFn: (data: Record<string, unknown>) =>
      axios.post("/api/transfers/bff-rpc/setup-request", data),
  });

export const TransferProcessBusinessRequestDialog = ({
  process,
}: TransferProcessBusinessRequestDialogProps) => {
  const navigate = useNavigate();
  const { mutateAsync } = useBffSetupTransferRequest();
  const { mutateAsync: setupDatasetRequestAsync } = useRpcSetupDatasetRequest();
  const [distributions, setDistributions] = useState<Distribution[]>([]);
  const myDspPath = useMyWellKnownDSPPath();
  const { path: providerDspPath } = useParticipantDSPPath(process.providerParticipantId);

  const form = useForm<TransferRequestInputs>({
    defaultValues: { distributionId: "", pushEndpointUrl: "" },
  });

  const fetchDistributions = async () => {
    try {
      const res = await setupDatasetRequestAsync({
        data: { associatedAgentPeer: process.providerParticipantId, dataset: process.target },
      });
      const data = res.status === 200 ? res.data : undefined;
      // @ts-ignore
      setDistributions(data?.distribution || data?.response?.distribution || []);
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
            { "@type": "EndpointProperty", name: "authorization", value: "TOKEN-ABCDEFG" },
            { "@type": "EndpointProperty", name: "authType", value: "bearer" },
          ],
        }
      : undefined;

    const res = await mutateAsync({
      associatedAgentPeer: process.providerParticipantId,
      providerAddress: providerDspPath || "",
      callbackAddress: myDspPath || "",
      agreementId: process.id,
      // @ts-ignore
      format: distribution.formats || "",
      ...(dataAddress ? { dataAddress } : {}),
    });

    if (res.status === 201) navigate({ to: "/transfer-process" });
  };

  return (
    <BaseProcessDialog<TransferRequestInputs>
      title="Transfer Request (Business)"
      description={
        <span className="max-w-full flex flex-wrap gap-1">
          Start transfer for Agreement{" "}
          <Badge variant="info">{formatUrn(process.id)}</Badge>.{" "}
          <span className="text-white/60">
            The Provider will automatically start the transfer upon receiving the request.
          </span>
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
                          {d.formats} - {d["@id"]}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </FormControl>
                <FormDescription>
                  Choosing the distribution method determines how data will be transferred.
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
                  <input
                    id="pushEndpointUrl"
                    type="url"
                    placeholder="https://your-endpoint.example.com"
                    className="flex h-7 w-full rounded-sm border-0 bg-white/5 px-2 py-1 text-xs placeholder:text-muted-foreground/70 focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring/50"
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
      submitLabel="Request & Auto-start"
      submitVariant="default"
      onSubmit={handleSubmit}
      form={form}
    />
  );
};
