import React from "react";
import { createFileRoute, useNavigate } from "@tanstack/react-router";
import { keepPreviousData } from "@tanstack/react-query";
import { useGetNegotiationProcesses } from "shared/src/data/orval/negotiations/negotiations";
import { GetNegotiationProcessesParams, NegotiationProcessDto } from "shared/src/data/orval/model";
import { DataTable } from "shared/src/components/DataTable";
import { PageLayout } from "shared/src/components/layout/PageLayout";
import { PageHeader } from "shared/src/components/layout/PageHeader";
import { Badge } from "shared/src/components/ui/badge";
import { FormatDate, RelativeTime } from "shared/src/components/ui/format-date";
import { SegmentedControl } from "shared/src/components/ui/segmented";
import { ParticipantLabel } from "shared/src/components/identity/ParticipantLabel";
import { Identifier } from "shared/src/components/identity/Identifier";
import { ContractNegotiationActions } from "shared/src/components/actions/ContractNegotiationActions";
import { ContractNegotiationBusinessActions } from "shared/src/components/actions/ContractNegotiationBusinessActions";
import { ACTIONS_MODE_OPTIONS, useActionsMode } from "shared/src/hooks/useActionsMode";
import { useTableQueryParams } from "shared/src/hooks/useTableQueryParams";

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
  const { search, ...queryParams } = apiParams;

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
              { label: "Any state", value: "all" },
              ...STATES.map((s) => ({ label: s.charAt(0) + s.slice(1).toLowerCase(), value: s })),
            ],
          },
          {
            id: "role",
            label: "My role",
            options: [
              { label: "Any role", value: "all" },
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
