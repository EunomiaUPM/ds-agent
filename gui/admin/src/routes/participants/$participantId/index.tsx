import { createFileRoute, useNavigate } from "@tanstack/react-router";
import { encodePathId } from "@eunomia/ui";
import { PageLayout } from "@eunomia/ui";
import { PageHeader } from "@eunomia/ui";
import { PageSection } from "@eunomia/ui";
import { Badge, BadgeRole } from "@eunomia/ui";
import { DataTable } from "@eunomia/ui";
import { FieldGrid, Identifier } from "@eunomia/ui";
import { FormatDate } from "@eunomia/ui";
import { JsonTable } from "@eunomia/ui";
import { useGetParticipantById } from "shared/data/orval/participants/participants.ts";
import { GeneralErrorComponent } from "@eunomia/ui";
import { useGetAgreementsByParticipantId } from "shared/data/orval/negotiations/negotiations.ts";
import { Skeleton } from "@eunomia/ui";
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
  const navigate = useNavigate();
  const {
    data: participant,
    isLoading: isParticipantLoading,
    isError: isParticipantError,
    error: participantError,
  } = useGetParticipantById(encodePathId(participantId));
  const { data: agreements, isLoading: isAgreementsLoading } =
    useGetAgreementsByParticipantId(participantId);
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
        <PageHeader eyebrow="Participant" title={<Skeleton className="h-7 w-64" />} />
        <Skeleton className="h-16 w-full rounded-lg" />
        <Skeleton className="h-48 w-full rounded-lg" />
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

  const hasExtraFields = !!p.extra_fields && Object.keys(p.extra_fields).length > 0;

  return (
    <PageLayout>
      <PageHeader
        eyebrow="Participant"
        title={p.participant_nick || <Identifier value={p.participant_id} />}
        badge={
          <Badge variant="role" dsrole={p.participant_type as BadgeRole}>
            {p.participant_type}
          </Badge>
        }
      />

      <FieldGrid
        columns={4}
        fields={[
          { label: "DID", value: <Identifier value={p.participant_id} tail={12} /> },
          {
            label: "Base URL",
            value: p.base_url && (
              <a
                href={p.base_url}
                target="_blank"
                rel="noreferrer"
                className="break-all font-mono text-xs text-brand-sky hover:underline"
              >
                {p.base_url}
              </a>
            ),
          },
          { label: "First registered", value: <FormatDate date={p.saved_at} /> },
          {
            label: "Last interaction",
            value: p.last_interaction ? <FormatDate date={p.last_interaction} /> : "None",
          },
        ]}
      />

      <PageSection title="Relations">
        {relations.length === 0 ? (
          <p className="text-sm text-ink/50">No public relations with this participant.</p>
        ) : (
          <ul className="divide-y divide-ink/[0.06] overflow-hidden rounded-lg border border-ink/10">
            {relations.map((r) =>
              r.visibility === "Anonymous" ? (
                <li key="anonymous" className="flex items-center justify-between px-3 py-2 text-sm">
                  <span className="text-muted-foreground">Someone in the organization</span>
                  <Badge variant="info" size="sm">
                    Anonymous
                  </Badge>
                </li>
              ) : (
                <li
                  key={r.user_id}
                  className="flex items-center justify-between gap-3 px-3 py-2 text-sm"
                >
                  <span className="font-medium">{r.username || r.user_id}</span>
                  <Badge variant="code" size="sm">
                    {r.role}
                  </Badge>
                </li>
              ),
            )}
          </ul>
        )}
      </PageSection>

      <PageSection title="Agreements">
        <DataTable
          data={agreementList}
          keyExtractor={(a) => a.id}
          onRowClick={(a) =>
            navigate({ to: "/agreements/$agreementId", params: { agreementId: a.id } })
          }
          searchable={false}
          emptyMessage="No agreements with this participant"
          defaultSortKey="createdAt"
          defaultSortDirection="desc"
          pageSize={5}
          filters={[
            {
              id: "state",
              label: "State",
              options: [
                { label: "Finalized", value: "FINALIZED" },
                { label: "Active", value: "ACTIVE" },
                { label: "Terminated", value: "TERMINATED" },
              ],
              filterFn: (a, val) => a.state?.toUpperCase().includes(val) ?? false,
            },
          ]}
          columns={[
            {
              header: "Agreement",
              accessorKey: "id",
              cell: (a) => <Identifier value={a.id} />,
            },
            {
              header: "State",
              accessorKey: "state",
              className: "w-32",
              cell: (a) => (
                <Badge variant="status" state={a.state} size="sm">
                  {a.state}
                </Badge>
              ),
            },
            {
              header: "Created",
              accessorKey: "createdAt",
              className: "w-44 whitespace-nowrap",
              sortValue: (a) => new Date(a.createdAt).getTime(),
              cell: (a) => <FormatDate date={a.createdAt} />,
            },
          ]}
        />
      </PageSection>

      <PageSection title="DID document">
        {did ? (
          <FieldGrid
            columns={2}
            fields={[
              {
                label: "Controller",
                span: 2,
                hidden: did.controller == null,
                value: (
                  <span className="break-all font-mono text-xs">{String(did.controller)}</span>
                ),
              },
              {
                label: "Services",
                value:
                  (did.service ?? []).length === 0 ? undefined : (
                    <ul className="space-y-1.5 text-xs">
                      {(did.service ?? []).map((s, i) => (
                        <li key={s.id ?? i}>
                          <span className="font-medium text-brand-sky">{s.type}</span>
                          <span className="block break-all font-mono text-ink/70">
                            {s.serviceEndpoint}
                          </span>
                        </li>
                      ))}
                    </ul>
                  ),
              },
              {
                label: "Verification methods",
                value:
                  did.verificationMethod.length === 0 ? undefined : (
                    <ul className="space-y-1.5 text-xs">
                      {did.verificationMethod.map((m, i) => (
                        <li key={String(m.id ?? i)}>
                          <span className="font-medium">{String(m.type ?? "-")}</span>
                          <span className="block break-all font-mono text-ink/70">
                            {String(m.id ?? "-")}
                          </span>
                        </li>
                      ))}
                    </ul>
                  ),
              },
            ]}
          />
        ) : (
          <p className="text-sm text-ink/50">
            {baseUrl ? "Could not read the DID document." : "No base URL."}
          </p>
        )}
      </PageSection>

      {hasExtraFields && (
        <PageSection title="Extra fields">
          <JsonTable value={p.extra_fields} />
        </PageSection>
      )}
    </PageLayout>
  );
}
