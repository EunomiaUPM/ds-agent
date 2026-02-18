import { createFileRoute } from "@tanstack/react-router";
import { useRpcSetupCatalogRequest, useRpcSetupDatasetRequest } from "shared/src/data/orval/catalog-rp-c/catalog-rp-c";
import { PageLayout } from "shared/src/components/layout/PageLayout";
import { PageHeader } from "shared/src/components/layout/PageHeader";
import { InfoGrid } from "shared/src/components/layout/InfoGrid";
import { PageSection } from "shared/src/components/layout/PageSection";
import { InfoList } from "shared/src/components/ui/info-list";
import { Badge } from "shared/src/components/ui/badge";
import { FormatDate } from "shared/src/components/ui/format-date";
import { formatUrn } from "shared/src/lib/utils";
import { DataTable } from "shared/src/components/DataTable";
import { useEffect, useMemo } from "react";
import { Skeleton } from "shared/src/components/ui/skeleton";
import { PolicyWrapperShow } from "shared/src/components/PolicyWrapperShow.tsx";
import { OdrlOffer } from "shared/src/data/orval/model/odrlOffer";
import { Dataset, RpcCatalogResponseMessageDto } from "shared/data/orval/model";

function RouteComponent() {
  const { participantId, datasetId } = Route.useParams();
  const { mutate, data, isPending, error } = useRpcSetupCatalogRequest();
  const currentDataset = useMemo(() => {
    if (!data) return null;
    const response = (data?.data as RpcCatalogResponseMessageDto).response!;
    const dataset = response.dataset!.find((d) => d["@id"] === datasetId);
    return dataset as Dataset
  }, [data]);

  useEffect(() => {
    mutate({
      data: {
        associatedAgentPeer: participantId,
        filter: [],
      },
    });
  }, [participantId, datasetId, mutate]);

  if (isPending) {
    return (
      <PageLayout>
        <PageHeader
          title="Participant Dataset"
          badge={<Skeleton className="h-8 w-48" />}
        />
        <div>Loading...</div>
      </PageLayout>
    );
  }

  if (error) {
    return (
      <div className="flex items-center justify-center h-full text-red-500">
        Error loading dataset: {error.message}
      </div>
    );
  }

  const dataset = data?.status === 200 ? data.data : undefined;

  if (!dataset) return null;

  return (
    <PageLayout>
      <PageHeader
        title="Dataset"
        badge={
          <Badge variant="info" size="lg">
            {formatUrn(currentDataset?.["@id"])}
          </Badge>
        }
      />
      <InfoGrid>
        <PageSection>
          <InfoList
            items={[
              {
                label: "Dataset title",
                // @ts-ignore
                value: currentDataset?.title
              },
              {
                label: "Issued",
                // @ts-ignore
                value: { type: "custom", content: <FormatDate date={currentDataset?.issued!} /> },
              },
            ]}
          />
        </PageSection>
      </InfoGrid>

      <PageSection title="Distributions">
        <DataTable
          className="text-sm"
          data={currentDataset?.distribution ?? []}
          keyExtractor={(d) => d["@id"]!}
          columns={[
            {
              header: "Distribution ID",
              cell: (d) => <Badge variant="info">{formatUrn(d["@id"]!)}</Badge>,
            },
            {
              header: "Title",
              // @ts-ignore
              accessorKey: "title",
            },
            // Add more columns if needed, e.g., accessService
          ]}
        />
      </PageSection>

      <PageSection title="ODRL Policies" className="mt-10">
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {currentDataset?.hasPolicy &&
            currentDataset.hasPolicy.map((policy) => (
              <PolicyWrapperShow
                key={policy["@id"]}
                // @ts-ignore
                policy={policy}
                datasetId={currentDataset["@id"]!}
                catalogId={undefined}
                datasetName={currentDataset.title!}
                showRequestAccess={true}
                participant={participantId}
              />
            ))}
        </div>
      </PageSection>
    </PageLayout>
  );
}

export const Route = createFileRoute(
  "/catalog/participant/$participantId/dataset/$datasetId",
)({
  component: RouteComponent,
});
