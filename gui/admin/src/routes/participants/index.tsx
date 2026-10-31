/**
 * participants/index.tsx
 *
 * Participants listing page with different layouts based on participant type:
 * - My agent: InfoList from /mates/myself (it is not a stored participant)
 * - Other agents: DataTable
 * - Authority: InfoList (authority info)
 *
 * @example
 * Used as the index route for /participants/
 */

import { createFileRoute, Link } from "@tanstack/react-router";
import { FormatDate } from "@eunomia/ui";

import { DataTable } from "@eunomia/ui";
import { Button } from "@eunomia/ui";
import { Badge, BadgeRole } from "@eunomia/ui";

// Icons
import { ArrowRight } from "lucide-react";
import { PageLayout } from "@eunomia/ui";
import { PageHeader } from "@eunomia/ui";
import { PageSection } from "@eunomia/ui";
import { FieldGrid, Identifier } from "@eunomia/ui";
import { useGetAllParticipants } from "shared/data/orval/participants/participants";
import { GeneralErrorComponent } from "@eunomia/ui";
import { ParticipantDto } from "shared/data/orval/model/participantDto";
import { useTableQueryParams } from "@eunomia/ui";

import { Skeleton } from "@eunomia/ui";

import { keepPreviousData } from "@tanstack/react-query";
import { useMyself } from "shared/src/data/useMyself";

type Participant = ParticipantDto;

// =============================================================================
// ROUTE
// =============================================================================

/**
 * Route definition for /participants/
 */
export const Route = createFileRoute("/participants/")({
  component: RouteComponent,
});

function RouteComponent() {
  const {
    params: queryParams,
    apiParams,
    onQueryChange,
  } = useTableQueryParams({
    defaultLimit: 10,
    defaultSort: "created_at_desc",
  });

  const {
    data: participants,
    isLoading,
    isFetching,
    isError,
    error,
  } = useGetAllParticipants(apiParams, {
    query: {
      placeholderData: keepPreviousData,
    },
  });
  const myAgent = useMyself();

  if (isLoading && !participants) {
    return (
      <PageLayout>
        <PageHeader title="Participants" />
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mb-8">
          {[1, 2, 3].map((i) => (
            <Skeleton key={i} className="h-32 w-full rounded-xl" />
          ))}
        </div>
        <Skeleton className="h-64 w-full rounded-xl" />
      </PageLayout>
    );
  }

  if (isError || !participants || participants.status !== 200) {
    const finalError = error instanceof Error ? error : new Error("Participants not found");
    return <GeneralErrorComponent error={finalError} reset={() => {}} />;
  }

  return (
    <PageLayout>
      <PageHeader
        title="Participants"
        description="This agent and the peers and authorities it has dealt with."
      />

      {myAgent && (
        <PageSection title="This agent">
          <FieldGrid
            columns={4}
            fields={[
              { label: "Name", value: myAgent.participant_nick || "Unnamed agent" },
              {
                label: "Role",
                value: (
                  <Badge variant="role" dsrole={myAgent.participant_type as BadgeRole}>
                    {myAgent.participant_type}
                  </Badge>
                ),
              },
              { label: "DID", value: <Identifier value={myAgent.participant_id} tail={12} /> },
              {
                label: "Base URL",
                value: <span className="break-all font-mono text-xs">{myAgent.base_url}</span>,
              },
            ]}
          />
        </PageSection>
      )}

      <PageSection title="Network Participants">
        <DataTable
          className="text-sm"
          data={participants?.data}
          serverSide={true}
          loading={isFetching}
          queryParams={queryParams}
          onQueryChange={onQueryChange}
          keyExtractor={(p) => p.participant_id!}
          searchPlaceholder="Filter participants by name or DID..."
          filters={[
            {
              id: "type",
              label: "Type",
              accessorKey: "participant_type",
              options: [
                { label: "Agent", value: "Agent" },
                { label: "Authority", value: "Authority" },
              ],
            },
          ]}
          columns={[
            {
              header: "Participant",
              accessorKey: "participant_nick",
              cell: (p) => (
                <div className="flex items-center gap-3">
                  <div className="w-8 h-8 rounded-lg flex items-center justify-center font-bold text-xs bg-background-200 text-muted-foreground">
                    {(p.participant_nick || "U").charAt(0).toUpperCase()}
                  </div>
                  <div className="flex items-baseline gap-2">
                    <span className="font-medium capitalize">
                      {p.participant_nick || "Unknown"}
                    </span>
                  </div>
                </div>
              ),
            },
            {
              header: "Participant Type",
              accessorKey: "participant_type",
              cell: (p) => (
                <Badge variant="role" dsrole={p.participant_type as BadgeRole}>
                  {p.participant_type}
                </Badge>
              ),
            },
            {
              header: "Participant DID",
              accessorKey: "participant_id",
              cell: (p) => (
                <Identifier value={p.participant_id} tail={12} />
              ),
            },
            {
              header: "Last interaction",
              accessorKey: "last_interaction",
              sortKey: "last_interaction",
              cell: (p: Participant) => (
                <p>{p.last_interaction ? <FormatDate date={p.last_interaction} /> : "Never"}</p>
              ),
            },
            {
              header: "Actions",
              cell: (p) => (
                <Link
                  to="/participants/$participantId"
                  params={{ participantId: p.participant_id! }}
                >
                  <Button variant="link">
                    See agent
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
}
