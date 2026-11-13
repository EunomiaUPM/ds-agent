/**
 * participants/index.tsx
 *
 * This agent, read from /mates/myself, and the participants it has stored.
 */

import { createFileRoute, useNavigate } from "@tanstack/react-router";
import { FormatDate } from "@eunomia/ui";

import { DataTable } from "@eunomia/ui";
import { Badge, BadgeRole } from "@eunomia/ui";

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
  const navigate = useNavigate();

  if (isLoading && !participants) {
    return (
      <PageLayout>
        <PageHeader title="Participants" />
        <Skeleton className="h-16 w-full rounded-lg" />
        <Skeleton className="h-48 w-full rounded-lg" />
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

      <PageSection title="Known participants">
        <DataTable
          data={participants?.data}
          onRowClick={(p) =>
            navigate({
              to: "/participants/$participantId",
              params: { participantId: p.participant_id! },
            })
          }
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
                <span className="font-medium capitalize">{p.participant_nick || "Unknown"}</span>
              ),
            },
            {
              header: "Type",
              accessorKey: "participant_type",
              className: "w-32",
              cell: (p) => (
                <Badge variant="role" dsrole={p.participant_type as BadgeRole} size="sm">
                  {p.participant_type}
                </Badge>
              ),
            },
            {
              header: "DID",
              accessorKey: "participant_id",
              cell: (p) => <Identifier value={p.participant_id} tail={12} />,
            },
            {
              header: "Last interaction",
              accessorKey: "last_interaction",
              sortKey: "last_interaction",
              className: "w-44 whitespace-nowrap",
              cell: (p: Participant) =>
                p.last_interaction ? <FormatDate date={p.last_interaction} /> : "Never",
            },
          ]}
        />
      </PageSection>
    </PageLayout>
  );
}
