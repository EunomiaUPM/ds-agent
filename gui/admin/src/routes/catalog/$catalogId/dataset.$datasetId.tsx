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
import { CatalogViews, type CatalogDoc, type DistributionView } from "shared/src/data/catalog";
import { JsonLd } from "shared/src/data/jsonld";
import { createFileRoute, Link, useNavigate } from "@tanstack/react-router";
import { ArrowUpRight, Pencil, Plus, Trash2 } from "lucide-react";
import {
  useGetDatasetById,
  getGetDatasetByIdQueryOptions,
} from "shared/src/data/orval/datasets/datasets";
import {
  useGetDistributionsByDatasetId,
  getGetDistributionsByDatasetIdQueryOptions,
} from "shared/src/data/orval/distributions/distributions";
import {
  useCreateOdrlPolicy,
  useGetPoliciesByEntityId,
} from "shared/src/data/orval/odrl-policies/odrl-policies";
import { useGetCatalogById } from "shared/src/data/orval/catalogs/catalogs";
import { useDoctorDataset } from "shared/src/data/orval/doctor/doctor";
import { OdrlPolicyDto } from "shared/src/data/orval/model";
import { PageLayout } from "shared/src/components/layout/PageLayout";
import { PageHeader } from "shared/src/components/layout/PageHeader";
import { FieldGrid } from "shared/src/components/layout/FieldGrid";
import { DataTable } from "shared/src/components/DataTable";
import { Button } from "shared/src/components/ui/button";
import { FormatDate } from "shared/src/components/ui/format-date";
import { CountBadge } from "shared/src/components/ui/count-badge";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "shared/src/components/ui/tabs";
import { Dialog, DialogTrigger } from "shared/src/components/ui/dialog";
import { Identifier } from "shared/src/components/identity/Identifier";
import { ParticipantLabel } from "shared/src/components/identity/ParticipantLabel";
import { JsonTable } from "shared/src/components/json/JsonTable";
import { PolicyCard } from "shared/src/components/policy/PolicyCard";
import { PolicyComposer } from "shared/src/components/policy/PolicyComposer";
import { DoctorPanel } from "shared/src/components/catalog/DoctorPanel";
import { ContractNegotiationNewOfferDialog } from "shared/src/components/dialogs/ContractNegotiationNewOfferDialog";
import {
  CatalogResourceEditor,
  type EditorTarget,
} from "shared/src/components/catalog/CatalogResourceEditor";
import {
  DeleteResourceDialog,
  type DeleteTarget,
} from "shared/src/components/catalog/DeleteResourceDialog";
import { DatasetDcatSummary } from "shared/src/components/catalog/DcatSummary";
import { useParticipantDirectory } from "shared/src/data/useParticipantDirectory";

export const Route = createFileRoute("/catalog/$catalogId/dataset/$datasetId")({
  component: DatasetPage,
  loader: async ({ context: { queryClient }, params: { datasetId } }) => {
    await queryClient.ensureQueryData(getGetDatasetByIdQueryOptions(datasetId));
    return queryClient.ensureQueryData(getGetDistributionsByDatasetIdQueryOptions(datasetId));
  },
});

function DatasetPage() {
  const { catalogId, datasetId } = Route.useParams();
  const navigate = useNavigate();
  const { self } = useParticipantDirectory();
  const [composerOpen, setComposerOpen] = useState(false);
  const [editing, setEditing] = useState<EditorTarget | null>(null);
  const [deleting, setDeleting] = useState<DeleteTarget | null>(null);

  const { data: catalogData } = useGetCatalogById(catalogId);
  const { data: datasetData } = useGetDatasetById(datasetId);
  const { data: distributionsData } = useGetDistributionsByDatasetId(datasetId);
  const { data: policiesData, refetch: refetchPolicies } = useGetPoliciesByEntityId(datasetId);
  const { mutateAsync: createPolicy } = useCreateOdrlPolicy();
  const { data: doctorData } = useDoctorDataset(datasetId, { deep: true });

  const dataset =
    datasetData?.status === 200 ? CatalogViews.dataset(datasetData.data as CatalogDoc) : undefined;
  const catalog =
    catalogData?.status === 200 ? CatalogViews.catalog(catalogData.data as CatalogDoc) : undefined;
  const distributions =
    distributionsData?.status === 200
      ? CatalogViews.list(distributionsData.data, CatalogViews.distribution)
      : [];
  const policies = policiesData?.status === 200 ? policiesData.data : [];
  const doctor = doctorData?.status === 200 ? doctorData.data : undefined;

  if (!dataset) return null;

  return (
    <PageLayout>
      <PageHeader
        eyebrow="Dataset"
        title={dataset.title || <Identifier value={dataset.id} />}
        description={dataset.description || "No description."}
        actions={
          <>
            <Button
              variant="outline"
              onClick={() => setEditing({ kind: "dataset", id: datasetId })}
            >
              <Pencil /> Edit
            </Button>
            <Button
              variant="outline_destructive"
              onClick={() => setDeleting({ kind: "dataset", id: datasetId, title: dataset.title })}
            >
              <Trash2 /> Delete
            </Button>
            <Button onClick={() => setComposerOpen(true)}>
              <Plus /> Add policy
            </Button>
          </>
        }
      />

      <FieldGrid
        columns={4}
        className="mb-6"
        fields={[
          {
            label: "Catalog",
            value: (
              <Link
                to="/catalog/$catalogId"
                params={{ catalogId }}
                className="inline-flex items-center gap-1 text-brand-sky hover:underline"
              >
                {catalog?.title || <Identifier value={catalogId} copyable={false} />}
                <ArrowUpRight className="h-3.5 w-3.5" />
              </Link>
            ),
          },
          { label: "Publisher", value: <ParticipantLabel id={self?.participant_id} compact /> },
          {
            label: "Issued",
            value: <FormatDate date={dataset.issued} format="YYYY-MM-DD HH:mm" />,
          },
          { label: "Identifier", value: <Identifier value={dataset.id} /> },
        ]}
      />

      <Tabs defaultValue="details">
        <TabsList>
          <TabsTrigger value="details">Details</TabsTrigger>
          <TabsTrigger value="policies">
            Access policies <CountBadge n={policies.length} showZero />
          </TabsTrigger>
          <TabsTrigger value="distributions">
            Distributions <CountBadge n={distributions.length} showZero />
          </TabsTrigger>
          <TabsTrigger value="conformance">Conformance</TabsTrigger>
          <TabsTrigger value="metadata">Metadata</TabsTrigger>
        </TabsList>

        <TabsContent value="details" className="pt-5">
          <DatasetDcatSummary doc={datasetData?.data as Record<string, unknown>} />
          <div className="mt-3 flex justify-end">
            <Button
              size="sm"
              variant="ghost"
              onClick={() => setEditing({ kind: "dataset", id: datasetId })}
            >
              <Pencil /> Edit metadata
            </Button>
          </div>
        </TabsContent>

        <TabsContent value="policies" className="pt-5">
          {policies.length === 0 ? (
            <div className="rounded-lg border border-dashed border-ink/15 py-10 text-center">
              <p className="text-sm text-ink/60">
                No offer is attached yet, so consumers cannot negotiate this dataset.
              </p>
              <Button size="sm" className="mt-4" onClick={() => setComposerOpen(true)}>
                <Plus /> Add policy
              </Button>
            </div>
          ) : (
            <div className="grid grid-cols-1 gap-4 xl:grid-cols-2">
              {policies.map((policy: OdrlPolicyDto) => (
                <PolicyCard
                  key={policy.id}
                  policy={policy}
                  actions={
                    <>
                      <Button
                        size="sm"
                        variant="ghost"
                        onClick={() =>
                          setDeleting({
                            kind: "policy",
                            id: policy.id,
                            title: policy.description ?? undefined,
                          })
                        }
                      >
                        <Trash2 /> Delete
                      </Button>
                      <Dialog>
                        <DialogTrigger asChild>
                          <Button size="sm" variant="outline">
                            Offer to a participant
                          </Button>
                        </DialogTrigger>
                        <ContractNegotiationNewOfferDialog
                          policy={policy}
                          catalogId={catalogId}
                          datasetId={dataset.id}
                        />
                      </Dialog>
                    </>
                  }
                />
              ))}
            </div>
          )}
        </TabsContent>

        <TabsContent value="distributions" className="pt-5">
          <div className="mb-3 flex justify-end">
            <Button
              size="sm"
              variant="outline"
              onClick={() => setEditing({ kind: "distribution", parentId: datasetId })}
            >
              <Plus /> Add distribution
            </Button>
          </div>
          <DataTable<DistributionView>
            data={distributions}
            keyExtractor={(d) => d.id}
            hideToolbar
            onRowClick={(d) =>
              navigate({
                to: "/catalog/$catalogId/distribution-connector/$distributionId",
                params: { catalogId, distributionId: d.id },
              })
            }
            emptyMessage="This dataset has no distributions."
            columns={[
              {
                header: "Distribution",
                cell: (d) => (
                  <span className="flex flex-col">
                    <span className="font-medium">{d.title || "Untitled distribution"}</span>
                    {d.description && <span className="text-xs text-ink/55">{d.description}</span>}
                  </span>
                ),
              },
              {
                header: "Format",
                cell: (d) => d.format ?? <span className="text-ink/35">—</span>,
              },
              {
                header: "Data service",
                cell: (d) => (
                  <Link
                    to="/catalog/$catalogId/data-service/$dataServiceId"
                    params={{ catalogId, dataServiceId: d.accessService }}
                    onClick={(e: React.MouseEvent) => e.stopPropagation()}
                    className="inline-flex items-center gap-1 text-brand-sky hover:underline"
                  >
                    <Identifier value={d.accessService} copyable={false} />
                    <ArrowUpRight className="h-3.5 w-3.5" />
                  </Link>
                ),
              },
              {
                header: "Issued",
                accessorKey: "issued",
                cell: (d) => <FormatDate date={d.issued} format="YYYY-MM-DD" />,
              },
              {
                header: "",
                className: "text-right",
                cell: (d) => (
                  <span
                    className="inline-flex items-center gap-1"
                    onClick={(e: React.MouseEvent) => e.stopPropagation()}
                  >
                    <Button
                      size="icon_sm"
                      variant="ghost"
                      title="Edit distribution"
                      onClick={() => setEditing({ kind: "distribution", id: d.id })}
                    >
                      <Pencil />
                    </Button>
                    <Button
                      size="icon_sm"
                      variant="ghost"
                      title="Delete distribution"
                      className="text-danger-700 dark:text-danger-300"
                      onClick={() =>
                        setDeleting({ kind: "distribution", id: d.id, title: d.title })
                      }
                    >
                      <Trash2 />
                    </Button>
                  </span>
                ),
              },
            ]}
          />
        </TabsContent>

        <TabsContent value="conformance" className="pt-5">
          <DoctorPanel report={doctor} />
        </TabsContent>

        <TabsContent value="metadata" className="pt-5">
          <JsonTable value={dataset} expandDepth={1} />
        </TabsContent>
      </Tabs>

      <PolicyComposer
        open={composerOpen}
        onOpenChange={setComposerOpen}
        targetLabel={dataset.title ?? dataset.id}
        onSubmit={async (odrlOffer, description) => {
          await createPolicy({
            data: {
              entityId: datasetId,
              entityType: "Dataset",
              odrlOffer: JsonLd.odrlOffer(odrlOffer),
              description,
            },
          });
          await refetchPolicies();
        }}
      />

      <CatalogResourceEditor
        target={editing}
        onClose={() => setEditing(null)}
        catalogId={catalogId}
      />
      <DeleteResourceDialog
        target={deleting}
        onClose={() => setDeleting(null)}
        onDeleted={(t) => t.kind === "dataset" && navigate({ to: "/my-catalog" })}
      />
    </PageLayout>
  );
}
