import { createFileRoute, Link } from "@tanstack/react-router";
import { useRpcSetupCatalogRequest } from "shared/src/data/orval/catalog-rp-c/catalog-rp-c";
import { PageLayout } from "shared/src/components/layout/PageLayout";
import { PageHeader } from "shared/src/components/layout/PageHeader";
import { InfoGrid } from "shared/src/components/layout/InfoGrid";
import { PageSection } from "shared/src/components/layout/PageSection";
import { InfoList } from "shared/src/components/ui/info-list";
import { Badge } from "shared/src/components/ui/badge";
import { FormatDate } from "shared/src/components/ui/format-date";
import { formatUrn } from "shared/src/lib/utils";
import { DataTable } from "shared/src/components/DataTable";
import { useEffect } from "react";
import { Skeleton } from "shared/src/components/ui/skeleton";
import { Button } from "shared/components/ui/button";
import { ArrowRight } from "lucide-react";

function RouteComponent() {
  const { participantId } = Route.useParams();
  const { mutate, data, isPending, error } = useRpcSetupCatalogRequest();

  useEffect(() => {
    mutate({
      data: {
        associatedAgentPeer: participantId,
        filter: []
      },
    });
  }, [participantId, mutate]);

  if (isPending) {
    return (
      <PageLayout>
        <PageHeader
          title="Transfer Process"
          badge={<Skeleton className="h-8 w-48" />}
        />
        <div>Loading...</div>
      </PageLayout>
    );
  }

  if (error) {
    return (
      <div className="flex items-center justify-center h-full text-red-500">
        Error loading catalog: {error.message}
      </div>
    );
  }

  const catalog = data?.status === 200 ? data.data : undefined;

  if (!catalog) return null;

  return (
    <PageLayout>
      <PageHeader
        title="Participant Catalog"
        badge={
          <Badge variant="info" size="lg">
            {formatUrn(catalog.response!["@id"]!)}
          </Badge>
        }
      />
      <InfoGrid>
        <PageSection>
          <InfoList
            items={[
              { label: "Catalog Title", value: catalog.response?.title },
              { label: "Creator", value: catalog.response?.creator },
              {
                label: "Issued",
                // @ts-ignore
                value: { type: "custom", content: <FormatDate date={catalog.response?.issued!} /> },
              },
            ]}
          />
        </PageSection>
      </InfoGrid>

      <PageSection title="Datasets">
        <DataTable
          className="text-sm"
          data={catalog.response?.dataset ?? []}
          keyExtractor={(d) => d["@id"]!}
          columns={[
            {
              header: "Dataset ID",
              cell: (d) => <Badge variant="info">{formatUrn(d["@id"]!)}</Badge>,
            },
            {
              header: "Title",
              // @ts-ignore
              accessorKey: "title",
            },
            {
              header: "Description",
              // @ts-ignore
              accessorKey: "description",
            },
            {
              header: "Issued",
              // @ts-ignore
              cell: (d) => <FormatDate date={d.issued!} />,
            },
            {
              header: "",
              cell: (d) => (
                <Link
                  to="/catalog/participant/$participantId/dataset/$datasetId"
                  params={{
                    participantId: participantId,
                    datasetId: d["@id"]!,
                  }}
                >
                  <Button variant="link" size="sm" className="h-auto p-0 text-xs">
                    See dataset
                    <ArrowRight className="ml-1 h-3 w-3" />
                  </Button>
                </Link>
              ),
            },
          ]}
        />
      </PageSection>

      <InfoGrid>
        <PageSection>
          <InfoList
            items={[
              {
                label: "Service ID",
                // @ts-ignore
                value: catalog.response?.service?.["@id"],
              },
              {
                label: "Title",
                // @ts-ignore
                value: catalog.response?.service?.title,
              },
              {
                label: "Endpoint URL",
                // @ts-ignore
                value: catalog.response?.service?.endpointURL,
              },
            ]}
          />
        </PageSection>
      </InfoGrid>
    </PageLayout>
  );
}

export const Route = createFileRoute("/catalog/participant/$participantId/")({
  component: RouteComponent,
});
