/*
 * Copyright (C) 2026 - Universidad Politécnica de Madrid - UPM
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program. If not, see <https://www.gnu.org/licenses/>.
 */

import React, { useState } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { createFileRoute, Link, useNavigate } from "@tanstack/react-router";
import { ArrowUpRight, KeyRound, Pencil, Plug, Sparkles, Trash2, Unplug } from "lucide-react";
import { toast } from "sonner";
import {
  Badge,
  Button,
  ConfirmDialog,
  FieldGrid,
  FormatDate,
  Identifier,
  JsonViewer,
  PageHeader,
  PageLayout,
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
} from "@eunomia/ui";
import { CatalogViews, text, type CatalogDoc } from "shared/src/data/catalog";
import { CatalogResources } from "shared/src/data/catalogResources";
import {
  CatalogResourceEditor,
  type EditorTarget,
} from "shared/src/components/catalog/CatalogResourceEditor";
import {
  DeleteResourceDialog,
  type DeleteTarget,
} from "shared/src/components/catalog/DeleteResourceDialog";
import { PublicationBadge, PublicationToggle } from "shared/src/components/catalog/Publication";
import { VisibilitySelect } from "shared/src/components/catalog/Visibility";
import {
  ConnectorParametersTable,
  ConnectorSpecView,
  extractInteractionMode,
  extractPrimaryProtocol,
} from "shared/src/components/connector/ConnectorSpecView";
import { ConnectorTemplateModel } from "shared/src/components/connector/connectorTemplateModel";
import { InstantiateConnectorDialog } from "shared/src/components/connector/InstantiateConnectorDialog";
import { DatasetLabel } from "shared/src/components/identity/DatasetLabel";
import { SchemaUrl } from "shared/src/components/schemas/SchemaPicker";
import {
  getGetConnectorInstanceByDistributionQueryKey,
  getGetConnectorInstanceByDistributionQueryOptions,
  useDeleteConnectorInstanceById,
  useGetConnectorInstanceByDistribution,
} from "shared/src/data/orval/connector/connector";
import { useGetConnectorTemplateByNameVersion } from "shared/src/data/orval/connector-templates/connector-templates";
import { useGetDatasetById } from "shared/src/data/orval/datasets/datasets";
import {
  getGetDistributionByIdQueryOptions,
  useGetDistributionById,
} from "shared/src/data/orval/distributions/distributions";

export const Route = createFileRoute("/catalog/$catalogId/distribution-connector/$distributionId")({
  component: DistributionConnectorPage,
  loader: async ({ context: { queryClient }, params: { distributionId } }) => {
    await queryClient.ensureQueryData(getGetDistributionByIdQueryOptions(distributionId));
    await queryClient
      .ensureQueryData(getGetConnectorInstanceByDistributionQueryOptions(distributionId))
      .catch(() => null);
  },
});

function DistributionConnectorPage() {
  const { catalogId, distributionId } = Route.useParams();
  const navigate = useNavigate();
  const queryClient = useQueryClient();

  const [editing, setEditing] = useState<EditorTarget | null>(null);
  const [deleting, setDeleting] = useState<DeleteTarget | null>(null);
  const [instantiateOpen, setInstantiateOpen] = useState(false);
  const [unbindOpen, setUnbindOpen] = useState(false);

  const { data: distributionData } = useGetDistributionById(distributionId);
  const { data: instanceData } = useGetConnectorInstanceByDistribution(distributionId, {
    query: { retry: false },
  });

  const distribution =
    distributionData?.status === 200
      ? CatalogViews.distribution(distributionData.data as CatalogDoc)
      : undefined;
  const instance = instanceData?.status === 200 ? instanceData.data : undefined;

  const { data: templateData, isLoading: loadingTemplate } = useGetConnectorTemplateByNameVersion(
    instance?.name ?? "",
    instance?.version ?? "",
    {
      query: {
        enabled: !!(instance?.name && instance?.version),
        retry: false,
      },
    },
  );
  const template = templateData?.status === 200 ? templateData.data : undefined;

  const { data: datasetData } = useGetDatasetById(distribution?.datasetId ?? "", {
    query: { enabled: !!distribution?.datasetId },
  });

  const unbindMutation = useDeleteConnectorInstanceById();

  const ownSchema =
    distributionData?.status === 200 ? text((distributionData.data as CatalogDoc)?.schema) : "";
  const datasetSchema =
    datasetData?.status === 200 ? text((datasetData.data as CatalogDoc)?.schema) : "";
  // A draft dataset hides its distributions, whatever their own state.
  const datasetDraft =
    datasetData?.status === 200 &&
    text((datasetData.data as CatalogDoc)?.publicationState) === "draft";

  if (!distribution) return null;
  const visibility = CatalogResources.visibilityOf(distributionData?.headers);

  const mode = instance ? extractInteractionMode(instance.interaction) : undefined;
  const protocol = instance ? extractPrimaryProtocol(instance.interaction) : undefined;
  const runtimeBuiltIns = instance
    ? ConnectorTemplateModel.builtInPlaceholders({
        authentication: instance.authenticationConfig,
        interaction: instance.interaction,
        parameters: [],
      })
    : [];

  const handleUnbind = async () => {
    if (!instance?.id) return;
    try {
      const res = await unbindMutation.mutateAsync({ id: instance.id });
      if (res.status !== 202) {
        const msg =
          (res.data as { message?: string; details?: string } | undefined)?.details ||
          (res.data as { message?: string } | undefined)?.message ||
          "Could not unbind connector instance.";
        toast.error(msg);
        return;
      }
      await queryClient.invalidateQueries({
        queryKey: getGetConnectorInstanceByDistributionQueryKey(distributionId),
      });
      await queryClient.invalidateQueries({ queryKey: ["/doctor"] });
      toast.success("Connector instance unbound from distribution.");
      setUnbindOpen(false);
    } catch (err) {
      toast.error(
        err instanceof Error
          ? err.message
          : "Could not unbind connector instance (check if active transfers exist).",
      );
    }
  };

  return (
    <PageLayout>
      <PageHeader
        eyebrow="Distribution"
        title={distribution.title || <Identifier value={distribution.id} />}
        badge={
          <>
            <PublicationBadge draft={distribution.draft || datasetDraft} />
            {distribution.format && <Badge variant="outline">{distribution.format}</Badge>}
            {mode && protocol && (
              <Badge variant="info">
                {mode} · {protocol}
              </Badge>
            )}
          </>
        }
        description={
          datasetDraft
            ? "Hidden from DSP and DCAT while its dataset is a draft."
            : distribution.description || undefined
        }
        actions={
          <div className="flex flex-wrap items-center justify-end gap-1.5">
            <VisibilitySelect kind="distribution" id={distributionId} visibility={visibility} />
            <PublicationToggle
              kind="distribution"
              id={distributionId}
              draft={distribution.draft}
              title={distribution.title}
              size="sm"
            />
            <Button
              variant="outline"
              size="sm"
              onClick={() => setEditing({ kind: "distribution", id: distributionId })}
            >
              <Pencil /> Edit
            </Button>
            <Button
              variant="outline_destructive"
              size="sm"
              onClick={() =>
                setDeleting({ kind: "distribution", id: distributionId, title: distribution.title })
              }
            >
              <Trash2 /> Delete
            </Button>
          </div>
        }
      />

      <FieldGrid
        columns={3}
        className="mb-4"
        fields={[
          {
            label: "Dataset",
            value: (
              <div className="flex items-start justify-between gap-2">
                <DatasetLabel id={distribution.datasetId} />
                <Link
                  to="/catalog/$catalogId/dataset/$datasetId"
                  params={{ catalogId, datasetId: distribution.datasetId }}
                  className="inline-flex shrink-0 items-center gap-0.5 text-xs text-brand-sky hover:underline"
                >
                  Open
                  <ArrowUpRight className="h-3.5 w-3.5" />
                </Link>
              </div>
            ),
          },
          {
            label: "Data service",
            value: (
              <Link
                to="/catalog/$catalogId/data-service/$dataServiceId"
                params={{ catalogId, dataServiceId: distribution.accessService }}
                className="inline-flex items-center gap-1 text-brand-sky hover:underline"
              >
                <Identifier value={distribution.accessService} copyable={false} />
                <ArrowUpRight className="h-3.5 w-3.5" />
              </Link>
            ),
          },
          {
            label: "Issued",
            value: <FormatDate date={distribution.issued} />,
          },
          {
            label: "Media type",
            value: distribution.mediaType && (
              <span className="font-mono text-xs">
                {distribution.mediaType.replace("http://www.iana.org/assignments/media-types/", "")}
              </span>
            ),
          },
          {
            label: ownSchema ? "Data schema" : "Data schema (inherited)",
            span: 2,
            value:
              ownSchema || datasetSchema ? <SchemaUrl url={ownSchema || datasetSchema} /> : null,
          },
          { label: "Distribution ID", value: <Identifier value={distribution.id} /> },
          {
            label: "Connector instance",
            value: instance ? (
              <Identifier value={instance.id} />
            ) : (
              <span className="text-xs text-ink/45">Unbound</span>
            ),
          },
          {
            label: "Source template",
            value: instance?.name ? (
              <Link
                to="/my-catalog/connectors/$templateName"
                params={{ templateName: instance.name }}
                search={{ version: instance.version ?? undefined }}
                className="inline-flex items-center gap-1.5 font-mono text-xs text-brand-sky hover:underline"
              >
                <span>{instance.name}</span>
                {instance.version && <span className="text-ink/55">v{instance.version}</span>}
                <ArrowUpRight className="h-3.5 w-3.5" />
              </Link>
            ) : (
              <span className="text-xs text-ink/45">—</span>
            ),
          },
        ]}
      />

      {!instance ? (
        <div className="rounded-lg border border-dashed border-ink/15 px-6 py-10 text-center">
          <Plug className="mx-auto h-9 w-9 text-ink/30" />
          <p className="mt-2.5 text-sm font-medium">
            No connector instance is bound to this distribution
          </p>
          <p className="mx-auto mt-1 max-w-md text-xs text-ink/55">
            Instantiate a HTTP or Kafka connector template so the dataplane can serve or stream this
            distribution.
          </p>
          <div className="mt-4 flex items-center justify-center gap-2">
            <Button size="sm" onClick={() => setInstantiateOpen(true)}>
              <Sparkles /> Instantiate from template
            </Button>
            <Button asChild size="sm" variant="outline">
              <Link to="/my-catalog/connectors">Browse templates</Link>
            </Button>
          </div>
        </div>
      ) : (
        <div className="space-y-4">
          <section className="rounded-lg border border-ink/10 p-4">
            <div className="mb-3 flex flex-wrap items-start justify-between gap-2 border-b border-ink/[0.07] pb-2.5">
              <div>
                <div className="flex flex-wrap items-center gap-2">
                  <h2 className="text-sm font-semibold">Resolved connector</h2>
                  <Badge variant="info">Active</Badge>
                  {instance.createdAt && (
                    <span className="text-xs text-ink/50">
                      Bound <FormatDate date={instance.createdAt} />
                    </span>
                  )}
                </div>
                {instance.description && (
                  <p className="mt-0.5 text-xs text-ink/60">{instance.description}</p>
                )}
              </div>
              <div className="flex flex-wrap items-center gap-1.5">
                <Button size="xs" variant="outline" onClick={() => setInstantiateOpen(true)}>
                  <Sparkles /> Update / replace
                </Button>
                <Button size="xs" variant="outline" onClick={() => setUnbindOpen(true)}>
                  <Unplug /> Unbind
                </Button>
              </div>
            </div>

            {runtimeBuiltIns.length > 0 && (
              <div className="mb-3 flex flex-wrap items-center gap-1.5 rounded-md border border-ink/10 bg-ink/[0.02] px-2.5 py-1.5 text-xs">
                <KeyRound className="h-3.5 w-3.5 shrink-0 text-ink/45" />
                <span className="text-ink/60">Runtime bindings:</span>
                {runtimeBuiltIns.map((name) => (
                  <code
                    key={name}
                    className="rounded bg-ink/[0.06] px-1.5 py-0.5 font-mono text-[11px] text-ink/80"
                  >
                    {`{{__${name}__}}`}
                  </code>
                ))}
              </div>
            )}

            <Tabs defaultValue="spec">
              <TabsList>
                <TabsTrigger value="spec">Resolved configuration</TabsTrigger>
                <TabsTrigger value="json">Raw JSON</TabsTrigger>
              </TabsList>
              <TabsContent value="spec" className="pt-3">
                <ConnectorSpecView
                  interaction={instance.interaction}
                  authentication={instance.authenticationConfig}
                />
              </TabsContent>
              <TabsContent value="json" className="pt-3">
                <JsonViewer value={instance} />
              </TabsContent>
            </Tabs>
          </section>

          <section className="rounded-lg border border-ink/10 p-4">
            <div className="mb-3 flex flex-wrap items-start justify-between gap-2 border-b border-ink/[0.07] pb-2.5">
              <div>
                <div className="flex flex-wrap items-center gap-2">
                  <h2 className="text-sm font-semibold">Source template</h2>
                  {instance.name && (
                    <Badge variant="outline" className="font-mono text-xs">
                      {instance.name}
                      {instance.version ? `@${instance.version}` : ""}
                    </Badge>
                  )}
                  {template?.author && (
                    <span className="text-xs text-ink/50">by {template.author}</span>
                  )}
                </div>
                {template?.description && (
                  <p className="mt-0.5 text-xs text-ink/55">{template.description}</p>
                )}
              </div>
              {instance.name && (
                <Button asChild size="xs" variant="outline">
                  <Link
                    to="/my-catalog/connectors/$templateName"
                    params={{ templateName: instance.name }}
                    search={{ version: instance.version ?? undefined }}
                  >
                    Open template <ArrowUpRight className="h-3.5 w-3.5" />
                  </Link>
                </Button>
              )}
            </div>

            {loadingTemplate ? (
              <p className="text-xs text-ink/50">Loading template…</p>
            ) : !template ? (
              <p className="text-xs text-ink/55">
                {instance.name
                  ? `Template ${instance.name} (v${instance.version ?? "?"}) is no longer in the library; the resolved instance above stays active.`
                  : "This connector instance has no linked template metadata."}
              </p>
            ) : (
              <Tabs defaultValue="parameters">
                <TabsList>
                  <TabsTrigger value="parameters">
                    Parameters ({template.parameters?.length ?? 0})
                  </TabsTrigger>
                  <TabsTrigger value="blueprint">Unexpanded blueprint</TabsTrigger>
                  <TabsTrigger value="json">Template JSON</TabsTrigger>
                </TabsList>
                <TabsContent value="parameters" className="pt-3">
                  <ConnectorParametersTable parameters={template.parameters ?? []} />
                </TabsContent>
                <TabsContent value="blueprint" className="pt-3">
                  <ConnectorSpecView
                    interaction={template.interaction}
                    authentication={template.authentication}
                  />
                </TabsContent>
                <TabsContent value="json" className="pt-3">
                  <JsonViewer value={template} />
                </TabsContent>
              </Tabs>
            )}
          </section>
        </div>
      )}

      <InstantiateConnectorDialog
        open={instantiateOpen}
        onOpenChange={setInstantiateOpen}
        distributionId={distributionId}
        distributionTitle={distribution.title}
        initialTemplateName={instance?.name ?? undefined}
        initialTemplateVersion={instance?.version ?? undefined}
        initialDescription={instance?.description ?? undefined}
      />

      <ConfirmDialog
        open={unbindOpen}
        onOpenChange={setUnbindOpen}
        title="Unbind connector instance?"
        description="The resolved connector instance will be deleted from this distribution. The underlying connector template is preserved."
        confirmLabel="Unbind"
        onConfirm={handleUnbind}
      >
        Active DSP transfers using this connector instance will prevent deletion (HTTP 409).
      </ConfirmDialog>

      <CatalogResourceEditor
        target={editing}
        onClose={() => setEditing(null)}
        catalogId={catalogId}
      />
      <DeleteResourceDialog
        target={deleting}
        onClose={() => setDeleting(null)}
        onDeleted={() =>
          navigate({
            to: "/catalog/$catalogId/dataset/$datasetId",
            params: { catalogId, datasetId: distribution.datasetId },
          })
        }
      />
    </PageLayout>
  );
}
