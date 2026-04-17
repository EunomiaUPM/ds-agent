import { createFileRoute, Link } from "@tanstack/react-router";
import { formatUrn } from "shared/src/lib/utils";
import dayjs from "dayjs";
import { ArrowRight } from "lucide-react";
import { DataTable } from "shared/src/components/DataTable";
import { FormatDate } from "shared/src/components/ui/format-date";
import CatalogItem from "shared/src/components/ui/catalog-item";
import Heading from "shared/src/components/ui/heading.tsx";
import { InfoList } from "shared/src/components/ui/info-list";
import { Button } from "shared/src/components/ui/button.tsx";
import { Input } from "shared/src/components/ui/input.tsx";
import { Badge, BadgeRole } from "shared/src/components/ui/badge";
import { PageLayout } from "shared/src/components/layout/PageLayout";
import { PageHeader } from "shared/src/components/layout/PageHeader";
import { PageSection } from "shared/src/components/layout/PageSection";
import { Skeleton } from "shared/src/components/ui/skeleton";
import { InfoGrid } from "shared/src/components/layout/InfoGrid";
import { useGetCatalogs, useGetMainCatalogs } from "shared/data/orval/catalogs/catalogs";
import { useGetAllParticipants } from "shared/data/orval/participants/participants";
import { useRpcSetupCatalogRequest } from "shared/src/data/orval/catalog-rp-c/catalog-rp-c";
import { useEffect } from "react";

const RouteComponent = () => {
  const { data: mainCatalog } = useGetMainCatalogs();
  const { data: catalogs } = useGetCatalogs();
  const { data: participants } = useGetAllParticipants();

  // For fetching catalogs from other participants
  // This is only useful for testing with one participant I think
  // 1. Get the participant of the other participant (not me) && with participant type Agent
  const otherParticipantForId = Array.isArray(participants?.data) ? participants.data.filter(p => !p.is_me && p.participant_type === "Agent") : [];
  
  // 2. Get the participant Id of this other participant
  const { participantId } = otherParticipantForId.map((p) => ({ participantId: p.participant_id }))[0] || {};

  // No sé si este paso es necesario 
  const participant = Array.isArray(participants?.data)  
  ? participants.data.find((p) => p.participant_id === participantId)
  : undefined;

  // 3. Get the participant slug and turn to string
  const otherParticipantSlug = participant?.participant_slug?.toString() || "Unknown Participant";

  // 4. Get participant id for URL
  const otherParticipantId = participant?.participant_id || "Unknown Participant ID";
  
  console.log(otherParticipantForId, "otherParticipantForId. 11");
  console.log(participant, "participant.  22");
  console.log(otherParticipantSlug, "otherParticipantSlug");

  console.log(catalogs, "catalogs")



  const { mutate, data, isPending, error } = useRpcSetupCatalogRequest();
    useEffect(() => {
      mutate({
        data: {
          associatedAgentPeer: participantId,
          filter: [],
            noCache: true
        },
      });
    }, [participantId, mutate]);
  const catalog = data?.status === 200 ? data.data : undefined;

    {console.log(catalog, "others catalogs")}
  if (!catalog) return null;


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
  
  if (!mainCatalog?.data || mainCatalog.status !== 200) return null;
  return (
    <PageLayout>
        <div className="bg-violet-900 flex justify-center items-center h-48">
                <Heading level="h2">
                    Browse catalogs from your participants' connections
                </Heading>
            </div>
    
      {/* <InfoGrid>
        <PageSection title="Main Catalog info: ">
          <InfoList
            items={[
              { label: "Catalog title", value: mainCatalog.data?.dctTitle },
              {
                label: "Catalog participant id",
                value: { type: "urn", value: mainCatalog.data.dspaceParticipantId },
              },
              { label: "Catalog homepage", value: mainCatalog.data.foafHomePage },
              {
                label: "Catalog creation date",
                value: { type: "custom", content: <FormatDate date={mainCatalog.data.dctIssued} /> },
              },
            ]}
          />
        </PageSection>
      </InfoGrid> */}

      {/* <PageSection title="Catalogs">
        <DataTable
          className="text-sm"
          data={Array.isArray(catalogs?.data) ? catalogs.data : []}
          keyExtractor={(c) => c.id!}
          columns={[
            {
              header: "Title",
              accessorKey: "dctTitle",
              cell: (c) => <p className="text-18">{c.dctTitle}</p>,
            },
            {
              header: "Created at",
              cell: (c) => <FormatDate date={c.dctIssued} />,
            },
            {
              header: "Catalog ID",
              cell: (c) => <Badge variant="info">{formatUrn(c.id)}</Badge>,
            },
            {
              header: "Provider ID",
              cell: (c) => <Badge variant="info">{formatUrn(c.dspaceParticipantId)}</Badge>,
            },
            {
              header: "Link",
              cell: (c) => (
                <Link to="/catalog/$catalogId" params={{ catalogId: c.id }}>
                  <Button variant={"link"}>
                    See catalog
                    <ArrowRight />
                  </Button>
                </Link>
              ),
            },
          ]}
        />
      </PageSection> */}

      {/* <PageSection title="Catalogs from other participants"> */}
        <div className="h-4" />
        <CatalogItem 
          date={catalog?.response?.issued}
          datasetNumber={catalog?.response?.dataset?.length}
          organizationName={otherParticipantSlug}
          id={otherParticipantId}

       />

          <div className="h-4" />
    
        <DataTable
          className="text-sm opacity-20"
          data={Array.isArray(participants?.data) ? participants.data.filter(p => !p.is_me && p.participant_type === "Agent") : []}
          keyExtractor={(c) => c.participant_id!}
          columns={[
            {
              header: "Participant ID",
              cell: (p) => <Badge variant={"info"}>{formatUrn(p.participant_id)}</Badge>,
            },
            {
              header: "Participant Type",
              cell: (p) => (
                <Badge variant={"role"} dsrole={p.participant_type as BadgeRole}>
                  {p.participant_type}
                </Badge>
              ),
            },
            {
              header: "Base URL",
              cell: (p) => <Badge variant={"info"}>{p.base_url}</Badge>,
            },
            {
              header: "Link",
              cell: (p) => (
                <Link to="/catalog/participant/$participantId" params={{ participantId: p.participant_id }}>
                  <Button variant="link">
                    Fetch catalog
                    <ArrowRight />
                  </Button>
                </Link>
              ),
            },
          ]}
        />
      {/* </PageSection> */}
    </PageLayout>
  );
};

/**
 * Route for listing catalogs.
 */
export const Route = createFileRoute("/catalog/")({
  component: RouteComponent,
  pendingComponent: () => <div>Loading...</div>,
});
