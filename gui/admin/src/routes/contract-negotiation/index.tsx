import React from "react";
import { createFileRoute, useNavigate } from "@tanstack/react-router";
import { keepPreviousData } from "@tanstack/react-query";
import { useGetNegotiationProcesses } from "shared/src/data/orval/negotiations/negotiations";
import { GetNegotiationProcessesParams, NegotiationProcessDto } from "shared/src/data/orval/model";
import { DataTable } from "@eunomia/ui";
import { PageLayout } from "@eunomia/ui";
import { PageHeader } from "@eunomia/ui";
import { Badge } from "@eunomia/ui";
import { FormatDate, RelativeTime } from "@eunomia/ui";
import { SegmentedControl } from "@eunomia/ui";
import { ParticipantLabel } from "@eunomia/ui";
import { Identifier } from "@eunomia/ui";
import { ContractNegotiationActions } from "shared/src/components/actions/ContractNegotiationActions";
import { ContractNegotiationBusinessActions } from "shared/src/components/actions/ContractNegotiationBusinessActions";
import { ACTIONS_MODE_OPTIONS, useActionsMode } from "shared/src/hooks/useActionsMode";
import { useTableQueryParams } from "@eunomia/ui";

export const Route = createFileRoute("/contract-negotiation/")({
  component: NegotiationsPage,
});

const STATES = [
  "REQUESTED",
  "OFFERED",
  "ACCEPTED",
  "AGREED",
  "VERIFIED",
  "FINALIZED",
  "TERMINATED",
];

function NegotiationsPage() {
  const navigate = useNavigate();
  const [mode, setMode] = useActionsMode();
  const { params, apiParams, onQueryChange } = useTableQueryParams({
    defaultLimit: 10,
    defaultSort: "created_at_desc",
  });
  const { search: _search, ...queryParams } = apiParams;

  const { data, isFetching } = useGetNegotiationProcesses(
    queryParams as GetNegotiationProcessesParams,
    {
      query: { placeholderData: keepPreviousData },
    },
  );
  const page = data?.status === 200 ? data.data : undefined;

  return (
    <PageLayout>
      <PageHeader
        title="Contract negotiations"
        description="Offers and agreements negotiated with other participants (DSP 8)."
        actions={
          <SegmentedControl
            value={mode}
            onChange={setMode}
            options={ACTIONS_MODE_OPTIONS}
            ariaLabel="Actions mode"
          />
        }
      />
      <DataTable<NegotiationProcessDto>
        data={page}
        serverSide
        loading={isFetching}
        queryParams={params}
        onQueryChange={onQueryChange}
        keyExtractor={(p) => p.id}
        onRowClick={(p) =>
          navigate({ to: "/contract-negotiation/$cnProcess", params: { cnProcess: p.id } })
        }
        searchable={false}
        emptyMessage="No negotiations yet. Request a dataset from a participant's catalog to start one."
        filters={[
          {
            id: "state",
            label: "State",
            options: [
              ...STATES.map((s) => ({ label: s.charAt(0) + s.slice(1).toLowerCase(), value: s })),
            ],
          },
          {
            id: "role",
            label: "My role",
            options: [
              { label: "Provider", value: "Provider" },
              { label: "Consumer", value: "Consumer" },
            ],
          },
        ]}
        columns={[
          {
            header: "Counterparty",
            cell: (p) => <ParticipantLabel id={p.correlation.peerParticipantId} compact />,
          },
          {
            header: "My role",
            cell: (p) => (
              <Badge variant="role" dsrole={p.role} size="sm">
                {p.role}
              </Badge>
            ),
          },
          {
            header: "State",
            cell: (p) => (
              <Badge variant="status" state={p.state} size="sm">
                {p.state}
              </Badge>
            ),
          },
          { header: "Negotiation", cell: (p) => <Identifier value={p.id} /> },
          {
            header: "Last change",
            cell: (p) => <RelativeTime date={p.updatedAt} className="whitespace-nowrap text-ink/70" />,
          },
          {
            header: "Started",
            accessorKey: "createdAt",
            sortKey: "created_at",
            cell: (p) => <FormatDate date={p.createdAt} />,
          },
          {
            header: "",
            className: "text-right",
            cell: (p) => (
              <span
                onClick={(e: React.MouseEvent) => e.stopPropagation()}
                className="inline-flex justify-end"
              >
                {mode === "business" ? (
                  <ContractNegotiationBusinessActions process={p} tiny={true} />
                ) : (
                  <ContractNegotiationActions process={p} tiny={true} />
                )}
              </span>
            ),
          },
        ]}
      />
    </PageLayout>
  );
}
