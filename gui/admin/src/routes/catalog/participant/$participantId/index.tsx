import { createFileRoute, Link } from "@tanstack/react-router";
import { useRpcSetupCatalogRequest } from "shared/src/data/orval/catalog-rp-c/catalog-rp-c";
import { PageLayout } from "shared/src/components/layout/PageLayout";
import { PageHeader } from "shared/src/components/layout/PageHeader";
import { InfoList } from "shared/src/components/ui/info-list";
import { FormatDate } from "shared/src/components/ui/format-date";
import { formatUrn } from "shared/src/lib/utils";
import { useEffect } from "react";
import { Skeleton } from "shared/src/components/ui/skeleton";
import Heading from "shared/src/components/ui/heading";
import DatasetItem from "shared/src/components/ui/dataset-item";
import Avatar from "shared/components/ui/avatar-img";
import { useFederatedCatalog } from "shared/data/useFederatedCatalog";
import { Badge } from "shared/src/components/ui/badge";
import { useGetAllParticipants } from "shared/src/data/orval/participants/participants";


function RouteComponent() {
  const { participantId } = Route.useParams();
  const federated = useFederatedCatalog();
  const { mutate, data, isPending, error } = useRpcSetupCatalogRequest();

  const { data: participantsResponse } = useGetAllParticipants();
  const localParticipants = participantsResponse?.status === 200 ? participantsResponse.data : [];

  const myAgent = localParticipants?.find((p) => p.is_me && p.participant_type === "Agent");
   
  console.log(myAgent, "myAgent")

  if (federated.state === "loading") {
    return (
      <PageLayout>
        <div>Loading...</div>
      </PageLayout>
    );
  }

  if (federated.state === "no-authority") {
    return (
      <p className="italic"> No authority found </p>
    );
  }

  if (federated.state === "error") {
    return (
      <PageLayout>
        <div className="flex items-center justify-center h-full text-red-500">
          Error loading federated catalog.
        </div>
      </PageLayout>
    );
  }

  const { agents } = federated;


  const participant = agents?.find((p) => {
    return p.participant_id === participantId;
  }
  )


  useEffect(() => {
    mutate({
      data: {
        associatedAgentPeer: participantId,
        filter: [],
        noCache: true,
      },
    });
  }, [participantId, mutate]);

  if (isPending) {
    return (
      <PageLayout>
        <PageHeader title="Participant Catalog" badge={<Skeleton className="h-8 w-48" />} />
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
      <div className="grid grid-cols-3 gap-12">
        <div className="rounded-md border border-background-200/60 bg-background-200/5 p-4 ">
          <div>
            <Heading level="h2" className="capitalize">
              {" "}
              {catalog.response?.title
                ? catalog.response?.title
                : `${participant?.participant_slug} Catalog `}
            </Heading>
            {participant?.participant_id === myAgent?.participant_id &&
              <Badge variant="detail" size="lg" className="uppercase text-blue-300 font-semibold mb-3">
                My own catalog
              </Badge>
            }
            <p className="text-sm mb-2">Description of the catalog. </p>
            <InfoList
              items={[
                {
                  label: "Catalog creation date",
                  value: {
                    type: "custom",
                    content: <FormatDate date={catalog.response?.issued} />,
                  },
                },
                {
                  label: "Organization",
                  value: {
                    type: "custom",
                    content: (
                      <div className="catalog-participant-container flex gap-2 justify-start">
                        <Avatar sizeClass="h-7" />
                        <Heading level="h4" className="capitalize">
                          {participant?.participant_slug}
                        </Heading>
                      </div>
                    ),
                  },
                },
              ]}
            />
          </div>

          <div className="h-3"></div>
          <div className="border-t border-white/10"></div>
          <div className="h-4"></div>

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
          ) : (
            <p className="text-muted-foreground italic text-sm">No dataservices to show</p>
          )}
        </div>
        <div className="col-span-2">
          <Heading level="h4" className="text-left">
            Datasets
          </Heading>
          <div className="grid grid-cols-2 gap-3">
            {Array.isArray(catalog?.response?.dataset) && catalog.response?.dataset.length > 0 ? (
              catalog.response.dataset.map((dataset: any) => (
                <DatasetItem
                  key={dataset["@id"]!}
                  title={dataset.title!}
                  description={dataset.dctDescription!}
                  date={dataset.issued!}
                  prevRoute={participantId}
                  datasetId={dataset["@id"]!}
                  ownDataset={false}
                  dataset={dataset}
                />
              ))
            ) : (
              <p className="text-muted-foreground italic text-sm">No datasets to show</p>
            )}
          </div>
        </div>
      </div>

    </PageLayout>
  );
}

export const Route = createFileRoute("/catalog/participant/$participantId/")({
  component: RouteComponent,
});
