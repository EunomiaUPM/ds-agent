/**
 * participants/index.tsx
 *
 * Participants listing page with different layouts based on participant type:
 * - Agent + isMe=true: InfoList (my agent info)
 * - Agent + isMe=false: DataTable (other agents)
 * - Authority: InfoList (authority info)
 *
 * @example
 * Used as the index route for /participants/
 */

import { createFileRoute, Link } from "@tanstack/react-router";
import { formatIdentifier, formatUrn } from "shared/src/lib/utils";
import { DataTable } from "shared/src/components/DataTable";
import { Button } from "shared/src/components/ui/button.tsx";
import { Badge, BadgeRole } from "shared/src/components/ui/badge";
import { buttonVariants } from "shared/src/components/ui/button";

// Icons
import { ArrowRight } from "lucide-react";
import { PageLayout } from "shared/src/components/layout/PageLayout";
import { PageHeader } from "shared/src/components/layout/PageHeader";
import { PageSection } from "shared/src/components/layout/PageSection";
import { useGetAllParticipants } from "shared/data/orval/participants/participants";
import { GeneralErrorComponent } from "@/components/GeneralErrorComponent";
import { ParticipantDto } from "shared/data/orval/model/participantDto";
import { useTableQueryParams } from "shared/src/hooks/useTableQueryParams";
import dayjs from "dayjs";
import { Card, CardContent, CardHeader, CardTitle } from "shared/src/components/ui/card";
import { Skeleton } from "shared/src/components/ui/skeleton";

import { keepPreviousData } from "@tanstack/react-query";

interface Participant extends ParticipantDto {
  last_interaction?: string;
  saved_at?: string;
  extra_fields?: any;
}

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
  const { params: queryParams, apiParams, onQueryChange } = useTableQueryParams({
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
  const rawParticipants = (
    Array.isArray(participants?.data)
      ? participants.data
      : (participants?.data as any)?.items || []
  ) as Participant[];
  const allParticipants = rawParticipants;

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

  const myAgent = allParticipants.find((p) => p.is_me);

  return (
    <PageLayout>
      <PageHeader
        title="Participants"
        badge={
          <Badge size="lg" className="uppercase font-medium">
            {allParticipants.length} total
          </Badge>
        }
      />

      {/* Quick Stats / My Agent info */}
      {myAgent && (
        <div className="mb-8">
          <Card className="bg-background-200/15 overflow-hidden relative">
            <CardHeader className="pb-2">
              <div className="flex justify-between items-center">
                <div>
                  <p className="text-xs font-semibold text-brand-sky uppercase tracking-wider mb-1">
                    My Local Agent
                  </p>
                  <CardTitle className="text-2xl">
                    {myAgent.participant_nick || "Unnamed Agent"}
                  </CardTitle>
                </div>
                <Badge variant="status" state="active">
                  Active
                </Badge>
              </div>
            </CardHeader>
            <CardContent>
              <div className="flex justify-between flex-wrap gap-8 mt-2">
                <div className="space-y-1">
                  <p className="text-xs text-muted-foreground uppercase">DID Identifier</p>
                  <Badge variant="info">
                    {formatIdentifier(myAgent.participant_id, 0, true, 40)}
                  </Badge>
                </div>
                <div className="space-y-1">
                  <p className="text-xs text-muted-foreground uppercase">System Role</p>
                  <Badge variant="role" dsrole={myAgent.participant_type as BadgeRole}>
                    {myAgent.participant_type}
                  </Badge>
                </div>
                <div className="space-y-1">
                  <p className="text-xs text-muted-foreground uppercase">Base URL</p>
                  <Badge variant="info">{myAgent.base_url}</Badge>
                </div>
                <div className="flex-1 flex items-end justify-end">
                  <Link
                    to="/participants/$participantId"
                    params={{ participantId: myAgent.participant_id! }}
                  >
                    <Button variant="link">
                      View My Profile
                      <ArrowRight />
                    </Button>
                  </Link>
                </div>
              </div>
            </CardContent>
          </Card>
        </div>
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
                { label: "All Types", value: "all" },
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
                  <div
                    className={`w-8 h-8 rounded-lg flex items-center justify-center font-bold text-xs ${p.is_me ? "bg-brand-sky text-white" : "bg-background-200 text-muted-foreground"}`}
                  >
                    {(p.participant_nick || "U").charAt(0).toUpperCase()}
                  </div>
                  <div className="flex items-baseline gap-2">
                    <span className="font-medium capitalize">
                      {p.participant_nick || "Unknown"}
                    </span>
                    {p.is_me && <Badge size="sm">IT'S ME</Badge>}
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
                <Badge variant="info">{formatIdentifier(p.participant_id, 0, true, 40)}</Badge>
              ),
            },
            {
              header: "Last interaction",
              accessorKey: "last_interaction",
              sortKey: "last_interaction",
              cell: (p: Participant) => (
                <p>
                  {p.last_interaction
                    ? dayjs(p.last_interaction).format("DD MM YYYY - HH:mm")
                    : "Never"}
                </p>
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
