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
import Heading from "shared/src/components/ui/heading";
import DatasetItem from "shared/src/components/ui/dataset-item";

function RouteComponent() {
  const { participantId } = Route.useParams();
  const { mutate, data, isPending, error } = useRpcSetupCatalogRequest();

{console.log(participantId, "participant id in participant catalog route")}

  useEffect(() => {
    mutate({
      data: {
        associatedAgentPeer: participantId,
        filter: [],
        noCache: true
      },
    });
  }, [participantId, mutate]);
  

  if (isPending) {
    return (
      <PageLayout>
        <PageHeader
          title="Participant Catalog"
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
{console.log(catalog?.response?.dataset, "catalog datasets")}
{console.log(catalog?.response?.dataset?.map((d: any) => d["@id"]), "catalog datasets ids in participant")}

  return (
    <PageLayout>
      {/* <PageHeader
        title="Participant Catalog"
        badge={
          <Badge variant="info" size="lg">
            {formatUrn(catalog.response!["@id"]!)}
          </Badge>
        }
      /> */}
      <div className="grid grid-cols-3 gap-12">
        <div className="rounded-md border border-background-200/60 bg-background-200/5 p-4 ">
          <div>
            <Heading level="h2"> {catalog.response?.title ? catalog.response?.title : "Participant Catalog"}</Heading>
            <p className="text-sm mb-2">Description of the catalog. </p>
            <InfoList
              items={[
             
                {
                  label: "Catalog creation date",
                  value: {
                    type: "custom",
                    content: <FormatDate date={catalog.response?.issued} />
                  },
                },

              ]}

            />
          </div>
                {/* <div className="catalog-participant-container flex gap-2 justify-start">
                        <img className="rounded-full bg-violet-600 h-6 aspect-square"></img>
                        <Heading level="h4" className='capitalize'> {myAgentSlug} </Heading>
                    </div> */}
          <div className="h-3"></div>
          <div className="border-t border-white/10"></div>
          <div className="h-4">

          </div>

          {catalog.response?.service ? (
            <div>
              <Heading level="h4" className="text-left">
                Dataservice
              </Heading>

              <InfoList
                items={[
                  {
                    label: "Service ID",
                    // @ts-ignore
                    value: formatUrn(catalog.response?.service?.["@id"]),
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
            </div>
          ) : <p className="text-muted-foreground italic text-sm">No dataservices to show</p>
          }

        </div>
        <div className="col-span-2">
          <Heading level="h4" className="text-left">
            Datasets
          </Heading>
          <div className="grid grid-cols-2 gap-3">
            {(Array.isArray(catalog?.response?.dataset) && catalog.response?.dataset.length > 0) ? catalog.response.dataset.map((dataset: any) => (
              <DatasetItem
                key={dataset["@id"]!}
                title={dataset.title!}
                description={dataset.dctDescription!}
                date={dataset.issued!}
                prevRoute={participantId}
                datasetId={dataset["@id"]!}
                ownDataset={false}

              />
            )) : <p className="text-muted-foreground italic text-sm">No datasets to show</p>}
          </div>
        </div>
      </div>
      {/* <InfoGrid>
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
      </InfoGrid>  */}
    </PageLayout>
  );

}

export const Route = createFileRoute("/catalog/participant/$participantId/")({
  component: RouteComponent,
});
