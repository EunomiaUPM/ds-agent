import { createFileRoute, Link } from "@tanstack/react-router";
import { formatUrn } from "shared/src/lib/utils";
import dayjs from "dayjs";
import { ArrowRight } from "lucide-react";
import { DataTable } from "shared/src/components/DataTable";
import { FormatDate } from "shared/src/components/ui/format-date";
import Heading from "shared/src/components/ui/heading.tsx";
import { InfoList } from "shared/src/components/ui/info-list";
import { Button } from "shared/src/components/ui/button.tsx";
import { Input } from "shared/src/components/ui/input.tsx";
import { Badge, BadgeRole } from "shared/src/components/ui/badge";
import { PageLayout } from "shared/src/components/layout/PageLayout";
import { PageHeader } from "shared/src/components/layout/PageHeader";
import { PageSection } from "shared/src/components/layout/PageSection";
import { InfoGrid } from "shared/src/components/layout/InfoGrid";
import { useGetCatalogs, useGetMainCatalogs } from "shared/data/orval/catalogs/catalogs";
import { useGetAllParticipants } from "shared/data/orval/participants/participants";

const RouteComponent = () => {
  const { data: mainCatalog } = useGetMainCatalogs();
  const { data: catalogs } = useGetCatalogs();
  const { data: participants } = useGetAllParticipants();

  if (!mainCatalog?.data || mainCatalog.status !== 200) return null;
  return (
    <PageLayout>
      <PageHeader
        title="Main Catalog with id"
        badge={
          <Badge variant="info" size="lg">
            {formatUrn(mainCatalog.data.id)}
          </Badge>
        }
      />
      <InfoGrid>
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
      </InfoGrid>

      <PageSection title="Catalogs">
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
      </PageSection>

      <PageSection title="Catalogs from other participants">
        <DataTable
          className="text-sm"
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
      </PageSection>
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
