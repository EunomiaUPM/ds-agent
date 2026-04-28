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
import { useContext, useMemo } from "react";
import { Button } from "shared/src/components/ui/button.tsx";
import { Badge, BadgeRole } from "shared/src/components/ui/badge";
import Heading from "shared/src/components/ui/heading";
import { buttonVariants } from "shared/src/components/ui/button";
import { InfoList } from "shared/src/components/ui/info-list";

// Icons
import { ArrowRight } from "lucide-react";
import { GlobalInfoContext, GlobalInfoContextType } from "shared/src/context/GlobalInfoContext.tsx";
import { PageLayout } from "shared/src/components/layout/PageLayout";
import { PageHeader } from "shared/src/components/layout/PageHeader";
import { PageSection } from "shared/src/components/layout/PageSection";
import { useGetAllParticipants } from "shared/data/orval/participants/participants";
import { GeneralErrorComponent } from "@/components/GeneralErrorComponent";
import { ParticipantDto } from "shared/data/orval/model/participantDto";
import dayjs from "dayjs";
import { Card, CardContent, CardHeader, CardTitle } from "shared/src/components/ui/card";
import { Skeleton } from "shared/src/components/ui/skeleton";

interface Participant extends ParticipantDto {
  last_interaction?: string;
  saved_at?: string;
  extra_fields?: any;
}

// =============================================================================
// ROUTE
// =============================================================================

/**
 * Route for listing participants with type-based layouts.
 */
export const Route = createFileRoute("/participants/")({
  component: RouteComponent,
});

function RouteComponent() {
  const { data: participants, isLoading, isError, error } = useGetAllParticipants();

  if (isLoading) {
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

  const allParticipants = (participants.data || []) as Participant[];
  const myAgent = allParticipants.find((p) => p.is_me);

  return (
    <PageLayout>
      <PageHeader
        title="Participants"
        badge={
          <Badge variant="info" size="lg">
            {allParticipants.length} total
          </Badge>
        }
      />

      {/* Quick Stats / My Agent info */}
      {myAgent && (
        <div className="mb-8">
          <Card className="overflow-hidden relative">
            <CardHeader className="pb-2">
              <div className="flex justify-between items-center">
                <div>
                  <p className="text-xs font-semibold text-brand-sky uppercase tracking-wider mb-1">
                    My Local Agent
                  </p>
                  <CardTitle className="text-2xl">
                    {myAgent.participant_slug || "Unnamed Agent"}
                  </CardTitle>
                </div>
                <Badge variant={"status"} state="ACTIVE" className="uppercase">
                  Active
                </Badge>
              </div>
            </CardHeader>
            <CardContent>
              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4 mt-2">
                <div className="space-y-1">
                  <p className="text-2xs text-muted-foreground uppercase">DID Identifier</p>
                  <Badge variant={"info"} title={myAgent.participant_id}>
                    {formatIdentifier(myAgent.participant_id)}
                  </Badge>
                </div>
                <div className="space-y-1">
                  <p className="text-[10px] text-muted-foreground uppercase">System Role</p>
                  <div className="flex pt-1">
                    <Badge variant="role" dsrole={myAgent.participant_type as BadgeRole}>
                      {myAgent.participant_type}
                    </Badge>
                  </div>
                </div>
                <div className="space-y-1">
                  <p className="text-[10px] text-muted-foreground uppercase">Base URL</p>
                  <Badge variant="info">{myAgent.base_url}</Badge>
                </div>
                <div className="flex items-end justify-end">
                  <Link
                    to="/participants/$participantId"
                    params={{ participantId: myAgent.participant_id! }}
                  >
                    <Button>
                      View My Agent
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
          data={allParticipants}
          keyExtractor={(p) => p.participant_id!}
          columns={[
            {
              header: "Name",
              accessorKey: "participant_slug",
              cell: (p) => (
                <div className="flex items-center gap-3">
                  <div
                    className={`w-8 h-8 rounded-full flex items-center justify-center font-bold text-sm ${p.is_me ? "bg-brand-purple text-white" : "bg-background-200 text-muted-foreground"}`}
                  >
                    {(p.participant_slug || "U").charAt(0).toUpperCase()}
                  </div>
                  <div className="flex items-center gap-2">
                    <span className="">{p.participant_slug || "Unknown"}</span>
                    {p.is_me && (
                      <Badge variant="detail" className="text-2xs text-brand-sky">
                        IT'S ME
                      </Badge>
                    )}
                  </div>
                </div>
              ),
            },
            {
              header: "Type",
              accessorKey: "participant_type",
              cell: (p) => (
                <Badge variant="role" dsrole={p.participant_type as BadgeRole}>
                  {p.participant_type}
                </Badge>
              ),
            },
            {
              header: "DID / ID",
              accessorKey: "participant_id",
              cell: (p) => (
                <Badge variant="info" title={p.participant_id}>
                  {formatIdentifier(p.participant_id)}
                </Badge>
              ),
            },
            {
              header: "Last Active",
              accessorKey: "last_interaction",
              cell: (p) => (
                <div>
                  {p.last_interaction
                    ? dayjs(p.last_interaction).format("DD/MM/YYYY - HH:mm")
                    : "Never"}
                </div>
              ),
            },
            {
              header: "Actions",
              cell: (p) => (
                <Link
                  to="/participants/$participantId"
                  params={{ participantId: p.participant_id! }}
                >
                  <Button variant="link" size={"sm"}>
                    See participant
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
