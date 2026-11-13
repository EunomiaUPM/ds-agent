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
import { ArrowUpRight, Pencil, Plug, Sparkles, Trash2, Unplug } from "lucide-react";
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
  PageSection,
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
          <>
            <VisibilitySelect kind="distribution" id={distributionId} visibility={visibility} />
            <PublicationToggle
              kind="distribution"
              id={distributionId}
              draft={distribution.draft}
              title={distribution.title}
              size="sm"
            />
            <Button size="sm" onClick={() => setInstantiateOpen(true)}>
              <Sparkles />
              {instance ? "Replace from template" : "Instantiate from template"}
            </Button>
            {instance && (
              <Button variant="outline" size="sm" onClick={() => setUnbindOpen(true)}>
                <Unplug /> Unbind connector
              </Button>
            )}
            <Button asChild variant="outline" size="sm">
              <Link
                to="/catalog/$catalogId/dataset/$datasetId"
                params={{ catalogId, datasetId: distribution.datasetId }}
              >
                Open dataset
              </Link>
            </Button>
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
          </>
        }
      />

      <FieldGrid
        columns={3}
        className="mb-6"
        fields={[
          { label: "Dataset", value: <DatasetLabel id={distribution.datasetId} /> },
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
            label: "Source connector template",
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
        <div className="rounded-xl border border-dashed border-ink/15 px-6 py-12 text-center">
          <Plug className="mx-auto h-10 w-10 text-ink/30" />
          <p className="mt-3 text-sm font-medium">
            No connector instance is bound to this distribution
          </p>
          <p className="mx-auto mt-1 max-w-lg text-xs text-ink/55">
            Bind a parameterized connector template (HTTP Pull, HTTP Push Webhook, or Kafka Push
            Stream) so the dataplane knows how to serve or stream data for this distribution.
          </p>
          <div className="mt-5 flex items-center justify-center gap-2">
            <Button size="sm" onClick={() => setInstantiateOpen(true)}>
              <Sparkles /> Instantiate from template
            </Button>
            <Button asChild size="sm" variant="outline">
              <Link to="/my-catalog/connectors">Browse connector templates</Link>
            </Button>
          </div>
        </div>
      ) : (
        <div className="space-y-8">
          {/* 1. Resolved Connector Instance card */}
          <section className="rounded-xl border border-ink/10 bg-ink/[0.012] p-5">
            <div className="mb-4 flex flex-wrap items-start justify-between gap-3 border-b border-ink/[0.07] pb-3.5">
              <div>
                <div className="flex items-center gap-2">
                  <h2 className="text-sm font-semibold">Connector Instance (Resolved Binding)</h2>
                  <Badge variant="info">Active binding</Badge>
                </div>
                <p className="mt-0.5 text-xs text-ink/55">
                  Concrete configuration resolved for this distribution and consumed by the
                  dataplane during DSP transfers.
                </p>
              </div>
              <Button size="xs" variant="outline" onClick={() => setInstantiateOpen(true)}>
                <Sparkles /> Re-instantiate / update parameters
              </Button>
            </div>

            <FieldGrid
              columns={4}
              className="mb-5"
              fields={[
                { label: "Instance ID", value: <Identifier value={instance.id} /> },
                {
                  label: "Template & version",
                  value: instance.name ? (
                    <span className="font-mono text-xs">
                      {instance.name} {instance.version ? `v${instance.version}` : ""}
                    </span>
                  ) : (
                    "—"
                  ),
                },
                {
                  label: "Bound at",
                  value: <FormatDate date={instance.createdAt ?? undefined} />,
                },
                {
                  label: "Instance notes",
                  value: instance.description || <span className="text-ink/40">—</span>,
                },
              ]}
            />

            <ConnectorSpecView
              interaction={instance.interaction}
              authentication={instance.authenticationConfig}
            />
          </section>

          {/* 2. Source Connector Template card alongside the distribution */}
          <section className="rounded-xl border border-ink/10 bg-ink/[0.012] p-5">
            <div className="mb-4 flex flex-wrap items-start justify-between gap-3 border-b border-ink/[0.07] pb-3.5">
              <div>
                <div className="flex items-center gap-2">
                  <h2 className="text-sm font-semibold">Connector Template (Source Blueprint)</h2>
                  {instance.name && (
                    <Badge variant="outline" className="font-mono text-xs">
                      {instance.name}
                      {instance.version ? `@${instance.version}` : ""}
                    </Badge>
                  )}
                </div>
                <p className="mt-0.5 text-xs text-ink/55">
                  Reusable template definition from which this connector instance was instantiated.
                </p>
              </div>
              {instance.name && (
                <div className="flex items-center gap-2">
                  <Button asChild size="xs" variant="outline">
                    <Link
                      to="/my-catalog/connectors/$templateName"
                      params={{ templateName: instance.name }}
                      search={{ version: instance.version ?? undefined }}
                    >
                      Open template <ArrowUpRight className="h-3.5 w-3.5" />
                    </Link>
                  </Button>
                </div>
              )}
            </div>

            {loadingTemplate ? (
              <p className="text-xs text-ink/50">Loading connector template blueprint…</p>
            ) : !template ? (
              <p className="text-xs text-ink/55">
                {instance.name
                  ? `Template ${instance.name} (v${instance.version ?? "?"}) is no longer in the template library, though the resolved connector instance above remains intact.`
                  : "This connector instance has no linked template metadata."}
              </p>
            ) : (
              <>
                <FieldGrid
                  columns={4}
                  className="mb-4"
                  fields={[
                    {
                      label: "Template name",
                      value: <span className="font-mono text-xs">{template.name}</span>,
                    },
                    {
                      label: "Version",
                      value: <span className="font-mono text-xs">v{template.version}</span>,
                    },
                    {
                      label: "Author",
                      value: template.author || <span className="text-ink/40">—</span>,
                    },
                    {
                      label: "Description",
                      value: template.description || <span className="text-ink/40">—</span>,
                    },
                  ]}
                />

                <Tabs defaultValue="parameters">
                  <TabsList>
                    <TabsTrigger value="parameters">
                      Declared parameters ({template.parameters?.length ?? 0})
                    </TabsTrigger>
                    <TabsTrigger value="blueprint">Unexpanded blueprint</TabsTrigger>
                    <TabsTrigger value="json">Raw template JSON</TabsTrigger>
                  </TabsList>
                  <TabsContent value="parameters" className="pt-4">
                    <PageSection title="Template parameters">
                      <ConnectorParametersTable parameters={template.parameters ?? []} />
                    </PageSection>
                  </TabsContent>
                  <TabsContent value="blueprint" className="pt-4">
                    <ConnectorSpecView
                      interaction={template.interaction}
                      authentication={template.authentication}
                    />
                  </TabsContent>
                  <TabsContent value="json" className="pt-4">
                    <JsonViewer value={template} />
                  </TabsContent>
                </Tabs>
              </>
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
