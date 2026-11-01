import React from "react";
import { createFileRoute, useNavigate } from "@tanstack/react-router";
import { keepPreviousData } from "@tanstack/react-query";
import { useGetTransferProcesses } from "shared/src/data/orval/transfers/transfers";
import { GetTransferProcessesParams, TransferProcessDto } from "shared/src/data/orval/model";
import { DataTable } from "@eunomia/ui";
import { PageLayout } from "@eunomia/ui";
import { PageHeader } from "@eunomia/ui";
import { Badge } from "@eunomia/ui";
import { FormatDate, RelativeTime } from "@eunomia/ui";
import { SegmentedControl } from "@eunomia/ui";
import { ParticipantLabel } from "@eunomia/ui";
import { Identifier } from "@eunomia/ui";
import { DatasetLabel } from "shared/src/components/identity/DatasetLabel";
import { useAgreementDirectory } from "shared/src/data/useAgreementDirectory";
import { useDatasetDirectory } from "shared/src/data/useDatasetDirectory";
import { TransferProcessActions } from "shared/src/components/actions/TransferProcessActions";
import { TransferProcessBusinessActions } from "shared/src/components/actions/TransferProcessBusinessActions";
import { ACTIONS_MODE_OPTIONS, useActionsMode } from "shared/src/hooks/useActionsMode";
import { useTableQueryParams } from "@eunomia/ui";

export const Route = createFileRoute("/transfer-process/")({
  component: TransfersPage,
});

const STATES = ["REQUESTED", "STARTED", "SUSPENDED", "COMPLETED", "TERMINATED"];

function TransfersPage() {
  const navigate = useNavigate();
  const [mode, setMode] = useActionsMode();
  const { params, apiParams, onQueryChange } = useTableQueryParams({
    defaultLimit: 10,
    defaultSort: "created_at_desc",
  });
  const { search: _search, ...queryParams } = apiParams;

  const { data, isFetching } = useGetTransferProcesses(queryParams as GetTransferProcessesParams, {
    query: { placeholderData: keepPreviousData },
  });
  const page = data?.status === 200 ? data.data : undefined;
  // Transfers carry only the agreement id: agreements then datasets, one batch call each.
  const agreements = useAgreementDirectory(page?.items.map((tp) => tp.correlation.agreementId) ?? []);
  const datasets = useDatasetDirectory([...agreements.byId.values()].map((a) => a.target));

  return (
    <PageLayout>
      <PageHeader
        title="Transfers"
        description="Data transfers run under an agreement, through the embedded dataplane (DSP 9)."
        actions={
          <SegmentedControl
            value={mode}
            onChange={setMode}
            options={ACTIONS_MODE_OPTIONS}
            ariaLabel="Actions mode"
          />
        }
      />
      <DataTable<TransferProcessDto>
        data={page}
        className="table-fixed [&_td]:overflow-hidden"
        serverSide
        loading={isFetching}
        queryParams={params}
        onQueryChange={onQueryChange}
        keyExtractor={(tp) => tp.id}
        onRowClick={(tp) =>
          navigate({
            to: "/transfer-process/$transferProcessId",
            params: { transferProcessId: tp.id },
          })
        }
        searchable={false}
        emptyMessage="No transfers yet. Request one from an active agreement."
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
              { label: "Provider", value: "provider" },
              { label: "Consumer", value: "consumer" },
            ],
          },
        ]}
        columns={[
          {
            header: "Dataset",
            cell: (tp) => {
              const agreement = agreements.byId.get(tp.correlation.agreementId ?? "");
              if (!agreements.isLoading && !agreement) {
                return <Identifier value={tp.correlation.agreementId} />;
              }
              return (
                <DatasetLabel
                  id={agreement?.target}
                  loading={agreements.isLoading}
                  directory={datasets}
                  compact
                />
              );
            },
          },
          {
            header: "Counterparty",
            cell: (tp) => <ParticipantLabel id={tp.correlation.peerParticipantId} compact />,
          },
          {
            header: "My role",
            className: "w-28",
            cell: (tp) => {
              const role = tp.role === "provider" ? "Provider" : "Consumer";
              return (
                <Badge variant="role" dsrole={role} size="sm">
                  {role}
                </Badge>
              );
            },
          },
          {
            header: "State",
            className: "w-32",
            cell: (tp) => (
              <Badge variant="status" state={tp.state} size="sm">
                {tp.state}
              </Badge>
            ),
          },
          { header: "Transfer", cell: (tp) => <Identifier value={tp.id} /> },
          {
            header: "Last change",
            className: "w-28",
            cell: (tp) => <RelativeTime date={tp.updatedAt} className="whitespace-nowrap text-ink/70" />,
          },
          {
            header: "Requested",
            className: "w-36 whitespace-nowrap",
            accessorKey: "createdAt",
            sortKey: "created_at",
            cell: (tp) => <FormatDate date={tp.createdAt} />,
          },
          {
            header: "",
            className: "w-64 text-right",
            cell: (tp) => (
              <span
                onClick={(e: React.MouseEvent) => e.stopPropagation()}
                className="inline-flex h-7 items-center justify-end"
              >
                {mode === "business" ? (
                  <TransferProcessBusinessActions process={tp} tiny={true} />
                ) : (
                  <TransferProcessActions process={tp} tiny={true} />
                )}
              </span>
            ),
          },
        ]}
      />
    </PageLayout>
  );
}
