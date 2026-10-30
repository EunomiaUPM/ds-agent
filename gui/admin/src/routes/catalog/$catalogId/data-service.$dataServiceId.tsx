import React, { useState } from "react";
import { createFileRoute, useNavigate } from "@tanstack/react-router";
import { Pencil, Trash2 } from "lucide-react";
import { CatalogViews, type CatalogDoc } from "shared/src/data/catalog";
import {
  useGetDataServiceById,
  getGetDataServiceByIdQueryOptions,
  useGetMainDataService,
} from "shared/src/data/orval/data-services/data-services";
import { FormatDate } from "shared/src/components/ui/format-date";
import { Button } from "shared/src/components/ui/button";
import { Badge } from "shared/src/components/ui/badge";
import { PageLayout } from "shared/src/components/layout/PageLayout";
import { PageHeader } from "shared/src/components/layout/PageHeader";
import { FieldGrid } from "shared/src/components/layout/FieldGrid";
import { Identifier } from "shared/src/components/identity/Identifier";
import { JsonTable } from "shared/src/components/json/JsonTable";
import { PageSection } from "shared/src/components/layout/PageSection";
import {
  CatalogResourceEditor,
  type EditorTarget,
} from "shared/src/components/catalog/CatalogResourceEditor";
import {
  DeleteResourceDialog,
  type DeleteTarget,
} from "shared/src/components/catalog/DeleteResourceDialog";

function RouteComponent() {
  const { dataServiceId } = Route.useParams();
  const navigate = useNavigate();
  const [editing, setEditing] = useState<EditorTarget | null>(null);
  const [deleting, setDeleting] = useState<DeleteTarget | null>(null);
  const { data: dataServiceData } = useGetDataServiceById(dataServiceId);
  const { data: mainData } = useGetMainDataService();
  const isMain =
    mainData?.status === 200 &&
    CatalogViews.dataService(mainData.data as CatalogDoc).id === dataServiceId;
  const dataService =
    dataServiceData?.status === 200
      ? CatalogViews.dataService(dataServiceData.data as CatalogDoc)
      : undefined;

  if (!dataService) return null;
  return (
    <PageLayout>
      <PageHeader
        eyebrow="Data service"
        title={dataService.title || <Identifier value={dataService.id} />}
        badge={
          isMain && (
            <Badge variant="status" state="active">
              Main data service
            </Badge>
          )
        }
        description={dataService.description || undefined}
        actions={
          <>
            <Button
              variant="outline"
              size="sm"
              onClick={() => setEditing({ kind: "dataService", id: dataServiceId })}
            >
              <Pencil /> Edit
            </Button>
            {!isMain && (
              <Button
                variant="outline_destructive"
                size="sm"
                onClick={() =>
                  setDeleting({ kind: "dataService", id: dataServiceId, title: dataService.title })
                }
              >
                <Trash2 /> Delete
              </Button>
            )}
          </>
        }
      />
      <FieldGrid
        columns={3}
        className="mb-6"
        fields={[
          {
            label: "Endpoint URL",
            value: dataService.endpointURL && (
              <span className="break-all font-mono text-xs">{dataService.endpointURL}</span>
            ),
            span: 2,
          },
          {
            label: "Issued",
            value: <FormatDate date={dataService.issued} />,
          },
          {
            label: "Endpoint description",
            value: dataService.endpointDescription && (
              <span className="break-all font-mono text-xs">{dataService.endpointDescription}</span>
            ),
            span: 2,
          },
          { label: "Identifier", value: <Identifier value={dataService.id} /> },
        ]}
      />
      <PageSection title="Metadata">
        <JsonTable value={dataServiceData?.data} expandDepth={1} />
      </PageSection>

      <CatalogResourceEditor target={editing} onClose={() => setEditing(null)} />
      <DeleteResourceDialog
        target={deleting}
        onClose={() => setDeleting(null)}
        onDeleted={() => navigate({ to: "/my-catalog" })}
      />
    </PageLayout>
  );
}

/**
 * Route for displaying data service details.
 */
export const Route = createFileRoute("/catalog/$catalogId/data-service/$dataServiceId")({
  component: RouteComponent,
  pendingComponent: () => <div>Loading...</div>,
  loader: ({ context: { queryClient }, params: { dataServiceId } }) => {
    return queryClient.ensureQueryData(getGetDataServiceByIdQueryOptions(dataServiceId));
  },
});
