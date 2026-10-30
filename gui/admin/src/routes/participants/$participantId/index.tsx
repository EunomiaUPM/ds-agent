import { createFileRoute } from "@tanstack/react-router";
import { formatUrn } from "@eunomia/ui";
import { encodePathId } from "@eunomia/ui";
import { PageLayout } from "@eunomia/ui";
import { PageHeader } from "@eunomia/ui";
import { PageSection } from "@eunomia/ui";

// Components
import { Heading } from "@eunomia/ui";
import { Badge, BadgeRole } from "@eunomia/ui";
import { InfoList } from "@eunomia/ui";
import { DataTable } from "@eunomia/ui";
import { FormatDate } from "@eunomia/ui";
import { useGetParticipantById } from "shared/data/orval/participants/participants.ts";
import { GeneralErrorComponent } from "@eunomia/ui";
import { useGetAgreementsByParticipantId } from "shared/data/orval/negotiations/negotiations.ts";
import { Skeleton } from "@eunomia/ui";
import { Card, CardContent, CardHeader, CardTitle } from "@eunomia/ui";
import { Separator } from "@eunomia/ui";
import { History, Shield, Globe, Cpu, Key, Calendar, Activity, Users } from "lucide-react";
import { useQuery } from "@tanstack/react-query";

import { ParticipantDto } from "shared/data/orval/model/participantDto";
import { useGetDidFromUrl } from "shared/src/data/orval/did/did";
import { customInstance } from "shared/src/data/orval-mutator";

type Participant = ParticipantDto;

/** Who added the participant, as `GET /mates/{id}/relations` gives it. */
interface ParticipantRelation {
  user_id: string;
  username?: string | null;
  participant_id: string;
  role: string;
  visibility: "Private" | "Anonymous" | "Public";
}

/** Public relations, each one; with none, one anonymous entry stands for all the anonymous. */
const shownRelations = (relations: ParticipantRelation[]): ParticipantRelation[] => {
  const publicOnes = relations.filter((r) => r.visibility === "Public");
  if (publicOnes.length > 0) return publicOnes;
  const anonymous = relations.find((r) => r.visibility === "Anonymous");
  return anonymous ? [anonymous] : [];
};

/**
 * Route for displaying individual participant details.
 */
export const Route = createFileRoute("/participants/$participantId/")({
  component: RouteComponent,
});

function RouteComponent() {
  const { participantId } = Route.useParams();
  const {
    data: participant,
    isLoading: isParticipantLoading,
    isError: isParticipantError,
    error: participantError,
  } = useGetParticipantById(encodePathId(participantId));
  const {
    data: agreements,
    isLoading: isAgreementsLoading,
    isError: isAgreementsError,
    error: agreementsError,
  } = useGetAgreementsByParticipantId(participantId);
  const baseUrl = participant?.status === 200 ? participant.data.base_url : undefined;
  const { data: didDoc } = useGetDidFromUrl(encodeURIComponent(baseUrl ?? ""), {
    query: { enabled: !!baseUrl },
  });
  const relationsPath = `/mates/${encodePathId(participantId)}/relations`;
  const { data: relationsResponse } = useQuery({
    queryKey: [relationsPath],
    queryFn: () =>
      customInstance<{ status: number; data: ParticipantRelation[] }>(relationsPath, {
        method: "GET",
      }),
  });

  if (isParticipantLoading || isAgreementsLoading) {
    return (
      <PageLayout>
        <PageHeader title="Participant Details" />
        <div className="space-y-6">
          <Skeleton className="h-48 w-full rounded-xl" />
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <Skeleton className="h-64 w-full rounded-xl" />
            <Skeleton className="h-64 w-full rounded-xl" />
          </div>
        </div>
      </PageLayout>
    );
  }

  if (isParticipantError || !participant || participant.status !== 200) {
    const error =
      participantError instanceof Error ? participantError : new Error("Participant not found");
    return <GeneralErrorComponent error={error} reset={() => {}} />;
  }

  const p = participant.data as Participant;
  const agreementList = agreements?.status === 200 ? agreements.data.items : [];
  const relations = shownRelations(relationsResponse?.status === 200 ? relationsResponse.data : []);
  const did = didDoc?.status === 200 ? didDoc.data : undefined;

  return (
    <PageLayout>
      <PageHeader
        title={p.participant_nick || "Participant Details"}
        badge={
          <div className="flex gap-2">
            <Badge variant="role" dsrole={p.participant_type as BadgeRole}>
              {p.participant_type}
            </Badge>
          </div>
        }
      />

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        {/* Left Column: Essential Info */}
        <div className="lg:col-span-2 space-y-6">
          <Card>
            <CardHeader className="flex flex-row items-center justify-between pb-2">
              <CardTitle className="text-lg font-semibold flex items-center gap-2">
                <Shield className="h-5 w-5 text-brand-sky" />
                Identity Information
              </CardTitle>
            </CardHeader>
            <CardContent>
              <InfoList
                items={[
                  {
                    label: "Identifier (DID)",
                    value: {
                      type: "custom" as const,
                      content: (
                        <div className="font-mono text-xs break-all bg-background-200 p-2 rounded border border-ink/10">
                          {p.participant_id}
                        </div>
                      ),
                    },
                  },
                  {
                    label: "Base URL",
                    value: {
                      type: "custom" as const,
                      content: (
                        <a
                          href={p.base_url}
                          target="_blank"
                          rel="noreferrer"
                          className="flex items-center gap-1 text-brand-sky hover:underline"
                        >
                          <Globe className="h-3 w-3" />
                          {p.base_url}
                        </a>
                      ),
                    },
                  },
                ]}
              />
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle className="text-lg font-semibold flex items-center gap-2">
                <Users className="h-5 w-5 text-brand-sky" />
                Relations
              </CardTitle>
            </CardHeader>
            <CardContent>
              {relations.length === 0 ? (
                <p className="text-xs italic text-muted-foreground text-center py-4">
                  No public relations with this participant
                </p>
              ) : (
                <ul className="divide-y divide-ink/10">
                  {relations.map((r) =>
                    r.visibility === "Anonymous" ? (
                      <li key="anonymous" className="flex items-center justify-between py-2 text-sm">
                        <span className="text-muted-foreground">Someone in the organization</span>
                        <Badge variant="info">Anonymous</Badge>
                      </li>
                    ) : (
                      <li
                        key={r.user_id}
                        className="flex items-center justify-between gap-3 py-2 text-sm"
                      >
                        <span className="font-medium">{r.username || r.user_id}</span>
                        <div className="flex items-center gap-2">
                          <Badge variant="code">{r.role}</Badge>
                          <Badge variant="info">Public</Badge>
                        </div>
                      </li>
                    ),
                  )}
                </ul>
              )}
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle className="text-lg font-semibold flex items-center gap-2">
                <History className="h-5 w-5 text-brand-sky" />
                Agreements & Contracts
              </CardTitle>
            </CardHeader>
            <CardContent>
              <DataTable
                className="text-sm"
                data={agreementList}
                keyExtractor={(a) => a.id}
                searchPlaceholder="Filter agreements by ID or state..."
                emptyMessage="No active agreements with this participant"
                defaultSortKey="createdAt"
                defaultSortDirection="desc"
                pageSize={5}
                filters={[
                  {
                    id: "state",
                    label: "Status",
                    options: [
                      { label: "All Statuses", value: "all" },
                      { label: "Finalized", value: "FINALIZED" },
                      { label: "Active", value: "ACTIVE" },
                      { label: "Terminated", value: "TERMINATED" },
                    ],
                    filterFn: (a, val) => a.state?.toUpperCase().includes(val) ?? false,
                  },
                ]}
                columns={[
                  {
                    header: "Agreement Id",
                    accessorKey: "id",
                    cell: (a) => <Badge variant="info">{formatUrn(a.id)}</Badge>,
                  },
                  {
                    header: "Status",
                    accessorKey: "state",
                    cell: (a) => (
                      <Badge variant="status" state={a.state}>
                        {a.state}
                      </Badge>
                    ),
                  },
                  {
                    header: "Created at",
                    accessorKey: "createdAt",
                    sortValue: (a) => new Date(a.createdAt).getTime(),
                    cell: (a) => <FormatDate date={a.createdAt} />,
                  },
                ]}
              />
            </CardContent>
          </Card>
        </div>

        {/* Right Column: Metadata & Activity */}
        <div className="space-y-6">
          <Card>
            <CardHeader>
              <CardTitle className="text-base font-semibold flex items-center gap-2">
                <Calendar className="h-4 w-4 text-brand-sky" />
                Timestamps
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="flex justify-between items-center text-sm">
                <span className="text-muted-foreground flex items-center gap-2">
                  <Key className="h-3 w-3" /> First Registered
                </span>
                <span className="font-medium">
                  <FormatDate date={p.saved_at} />
                </span>
              </div>
              <Separator />
              <div className="flex justify-between items-center text-sm">
                <span className="text-muted-foreground flex items-center gap-2">
                  <Activity className="h-3 w-3" /> Last Interaction
                </span>
                <span className="font-medium">
                  {p.last_interaction ? <FormatDate date={p.last_interaction} /> : "None"}
                </span>
              </div>
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle className="text-base font-semibold flex items-center gap-2">
                <Cpu className="h-4 w-4 text-brand-sky" />
                Extended Metadata
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="space-y-2">
                <p className="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
                  DID document
                </p>
                {did ? (
                  <div className="space-y-3 text-xs">
                    {did.controller != null && (
                      <div>
                        <span className="text-muted-foreground">Controller</span>
                        <p className="font-mono break-all">{String(did.controller)}</p>
                      </div>
                    )}
                    <div>
                      <span className="text-muted-foreground">Services</span>
                      {(did.service ?? []).length === 0 ? (
                        <p className="italic text-muted-foreground">None</p>
                      ) : (
                        (did.service ?? []).map((s, i) => (
                          <div
                            key={s.id ?? i}
                            className="mt-1 rounded border border-ink/10 bg-background-200/30 p-2"
                          >
                            <p className="font-medium text-brand-sky">{s.type}</p>
                            <p className="break-all text-ink/70">{s.serviceEndpoint}</p>
                          </div>
                        ))
                      )}
                    </div>
                    <div>
                      <span className="text-muted-foreground">Verification methods</span>
                      {did.verificationMethod.map((m, i) => (
                        <div
                          key={String(m.id ?? i)}
                          className="mt-1 rounded border border-ink/10 bg-background-200/30 p-2"
                        >
                          <p className="font-medium">{String(m.type ?? "-")}</p>
                          <p className="font-mono break-all text-ink/70">{String(m.id ?? "-")}</p>
                        </div>
                      ))}
                    </div>
                  </div>
                ) : (
                  <p className="text-xs italic text-muted-foreground">
                    {baseUrl ? "Could not read the DID document" : "No base URL"}
                  </p>
                )}
              </div>
              <Separator />
              <div className="space-y-2">
                <p className="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
                  Extra fields
                </p>
                {p.extra_fields && Object.keys(p.extra_fields).length > 0 ? (
                  <pre className="text-xs bg-background-300 p-3 rounded-lg overflow-x-auto max-h-[300px]">
                    {JSON.stringify(p.extra_fields, null, 2)}
                  </pre>
                ) : (
                  <p className="text-xs italic text-muted-foreground">No extra fields available</p>
                )}
              </div>
            </CardContent>
          </Card>
        </div>
      </div>
    </PageLayout>
  );
}

const DataTypeSkeleton = () => (
  <div className="space-y-4">
    <Skeleton className="h-8 w-48" />
    <Skeleton className="h-24 w-full" />
  </div>
);
