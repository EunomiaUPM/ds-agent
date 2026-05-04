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
import { useMemo, useState } from "react";
import { Button } from "shared/src/components/ui/button.tsx";
import { Badge, BadgeRole } from "shared/src/components/ui/badge";
import { buttonVariants } from "shared/src/components/ui/button";

// Icons
import { ArrowRight, ArrowUpDown, ArrowUp, ArrowDown } from "lucide-react";
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
  const [sortConfig, setSortConfig] = useState<{ key: string; direction: "asc" | "desc" } | null>(
    null,
  );

  const rawParticipants = (participants?.data || []) as Participant[];

  const allParticipants = useMemo(() => {
    let sortableParticipants = [...(rawParticipants || [])];
    if (sortConfig !== null) {
      sortableParticipants.sort((a: any, b: any) => {
        const aVal = a[sortConfig.key];
        const bVal = b[sortConfig.key];

        if (aVal === bVal) return 0;

        if (aVal === null || aVal === undefined) return sortConfig.direction === "asc" ? -1 : 1;
        if (bVal === null || bVal === undefined) return sortConfig.direction === "asc" ? 1 : -1;

        const aString = String(aVal).toLowerCase();
        const bString = String(bVal).toLowerCase();

        if (aString < bString) {
          return sortConfig.direction === "asc" ? -1 : 1;
        }
        if (aString > bString) {
          return sortConfig.direction === "asc" ? 1 : -1;
        }
        return 0;
      });
    }
    return sortableParticipants;
  }, [rawParticipants, sortConfig]);

  const handleSort = (key: string) => {
    let direction: "asc" | "desc" = "asc";
    if (sortConfig && sortConfig.key === key && sortConfig.direction === "asc") {
      direction = "desc";
    }
    setSortConfig({ key, direction });
  };

  const getSortIcon = (key: string) => {
    if (!sortConfig || sortConfig.key !== key)
      return <ArrowUpDown className="ml-2 h-4 w-4 opacity-50" />;
    return sortConfig.direction === "asc" ? (
      <ArrowUp className="ml-2 h-4 w-4" />
    ) : (
      <ArrowDown className="ml-2 h-4 w-4" />
    );
  };

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
                    {myAgent.participant_slug || "Unnamed Agent"}
                  </CardTitle>
                </div>
                <Badge variant="status" state="active">
                  Active
                </Badge>
              </div>
            </CardHeader>
            <CardContent>
              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4 mt-2">
                <div className="space-y-1">
                  <p className="text-[10px] text-muted-foreground uppercase">DID Identifier</p>
                  <Badge variant="info">{formatIdentifier(myAgent.participant_id)}</Badge>
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
          data={allParticipants}
          keyExtractor={(p) => p.participant_id!}
          columns={[
            {
              header: "Participant",
              // (
              //   <Button
              //     variant="ghost"
              //     onClick={() => handleSort("participant_slug")}
              //     className="p-0 h-auto font-semibold"
              //   >
              //     Name {getSortIcon("participant_slug")}
              //   </Button>
              // ),
              accessorKey: "participant_slug",
              cell: (p) => (
                <div className="flex items-center gap-3">
                  <div
                    className={`w-8 h-8 rounded-lg flex items-center justify-center font-bold text-xs ${p.is_me ? "bg-brand-sky text-white" : "bg-background-200 text-muted-foreground"}`}
                  >
                    {(p.participant_slug || "U").charAt(0).toUpperCase()}
                  </div>
                  <div className="flex items-baseline gap-2">
                    <span className="font-medium capitalize">
                      {p.participant_slug || "Unknown"}
                    </span>
                    {p.is_me && <Badge size="sm">IT'S ME</Badge>}
                  </div>
                </div>
              ),
            },
            {
              header: "Participant Type",
              // (
              //   <Button
              //     variant="ghost"
              //     onClick={() => handleSort("participant_type")}
              //     className="p-0 h-auto font-semibold"
              //   >
              //     Type {getSortIcon("participant_type")}
              //   </Button>
              // ),
              accessorKey: "participant_type",
              cell: (p) => (
                <Badge variant="role" dsrole={p.participant_type as BadgeRole}>
                  {p.participant_type}
                </Badge>
              ),
            },
            {
              header: "Participant DID",
              // (
              //   <Button
              //     variant="ghost"
              //     onClick={() => handleSort("participant_id")}
              //     className="p-0 h-auto font-semibold"
              //   >
              //     DID / ID {getSortIcon("participant_id")}
              //   </Button>
              // ),
              accessorKey: "participant_id",
              cell: (p) => <Badge variant="info">{formatIdentifier(p.participant_id)}</Badge>,
            },
            {
              header: "Last interaction",
              // (
              //   <Button
              //     variant="ghost"
              //     onClick={() => handleSort("last_interaction")}
              //     className="p-0 h-auto font-semibold"
              //   >
              //     Last Active {getSortIcon("last_interaction")}
              //   </Button>
              // ),
              accessorKey: "last_interaction",
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
