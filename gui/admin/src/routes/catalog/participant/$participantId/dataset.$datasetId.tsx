import { createFileRoute, Link } from "@tanstack/react-router";
import {
  useRpcSetupCatalogRequest,
  useRpcSetupDatasetRequest,
} from "shared/src/data/orval/catalog-rp-c/catalog-rp-c";
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
import Heading from "shared/components/ui/heading";
import { useGetAllParticipants } from "shared/data/orval/participants/participants";
import DistributionItem from "shared/components/ui/distribution-item";
import Avatar from "shared/components/ui/avatar-img";

function RouteComponent() {
  const { participantId, datasetId } = Route.useParams();
  const { data: participants } = useGetAllParticipants();
  const { mutate, data, isPending, error } = useRpcSetupCatalogRequest();
  const currentDataset = useMemo(() => {
    if (!data) return null;
    const response = (data?.data as RpcCatalogResponseMessageDto).response!;
    const dataset = response.dataset!.find((d) => d["@id"] === datasetId);
    return dataset as Dataset;
  }, [data]);

  const otherParticipant = Array.isArray(participants?.data)
    ? participants.data.find((p) => !p.is_me && p.participant_type === "Agent")
    : undefined;
  const otherParticipantSlug =
    otherParticipant?.participant_slug?.toString() || "Unknown Participant";

  useEffect(() => {
    mutate({
      
      data: {
        associatedAgentPeer: participantId,
        filter: [],
        noCache: true,
      },
    });
  }, [participantId, datasetId, mutate]);

  if (isPending) {
    return (
      <PageLayout>
        <PageHeader title="Participant Dataset" badge={<Skeleton className="h-8 w-48" />} />
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
      <div className="grid grid-cols-3 gap-12 max-h-[85vh]">
        <div className="rounded-md border border-background-200/60 bg-background-200/5 p-4 ">
          <Heading level="h2">{currentDataset?.title} Dataset</Heading>
          <p className="text-sm mb-2"> {`Dataset with data about ${currentDataset?.title}`} </p>
          <InfoList
            items={[
              {
                label: "Issued",
                // @ts-ignore
                value: { type: "custom", content: <FormatDate date={currentDataset?.issued!} /> },
              },

              {
                label: "Organization",
                value: {
                  type: "custom",
                  content: (
                    <div className="catalog-participant-container flex gap-2 justify-start">
                  <Avatar />
                      <Heading level="h4" className="capitalize">
                        {" "}
                        {otherParticipantSlug}{" "}
                      </Heading>
                    </div>
                  ),
                },
              },
              {
                label: "Part of Catalog",
                value: {
                  type: "custom",
                  content: (
                    <div className="bg-background-200/15  border rounded-md border-white/5 flex flex-col p-3 gap-1">
                      <Link
                        to="/catalog/participant/$participantId"
                        params={{ participantId: participantId }}
                      >
                        <Heading
                          level="h5"
                          className="capitalize !mb-0 underline-offset-2 hover:underline"
                        >
                          {" "}
                          {otherParticipantSlug}'s Catalog
                        </Heading>
                      </Link>
                      <p className="text-xs text-muted-foreground">
                        {" "}
                        This is the catalog of{" "}
                        <span className="capitalize">{otherParticipantSlug}</span>
                      </p>
                    </div>
                  ),
                },
              },
            ]}
          />
        </div>
        <div className="container-policies-distributions max-h-[85vh] overflow-y-scroll pr-4 col-span-2">
          <Heading level="h4" className="text-left flex gap-3">
            Policies
          </Heading>
          <div className="grid grid-cols-2 gap-3">
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
          <div className="h-6"></div>
          <Heading level="h4" className="text-left">
            Distributions
          </Heading>
          <div className="grid grid-cols-2 gap-3">
            {currentDataset?.distribution &&
              currentDataset?.distribution.map((d: any) => (
                <DistributionItem
                  key={d["@id"]}
                  title={d.title ? d.title : "Distribution title"}
                  description={d.dctDescription}
                  date={d.dctIssued}
                  ownDataset={false}
                  prevRoute={""}
                  distribuionId={d["@id"]}
                  dataserviceId={d.dcatAccessService}
                />
              ))}
          </div>
        </div>
      </div>

      {/* <PageHeader
                title="Dataset"
                badge={
                    <Badge variant="info" size="lg">
                        {formatUrn(currentDataset?.["@id"])}
                    </Badge>
                }
            /> */}
      {/* <InfoGrid>
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
            </InfoGrid> */}
      {/* 
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
            </PageSection> */}

      {/* <PageSection title="ODRL Policies" className="mt-10">
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
            </PageSection> */}
    </PageLayout>
  );
}

export const Route = createFileRoute("/catalog/participant/$participantId/dataset/$datasetId")({
  component: RouteComponent,
});
