import { createFileRoute, Link } from "@tanstack/react-router";
import { formatIdentifier } from "shared/src/lib/utils";
import { DataTable } from "shared/src/components/DataTable";
import { FormatDate } from "shared/src/components/ui/format-date";
import { Button } from "shared/src/components/ui/button";
import { Badge } from "shared/src/components/ui/badge";
import { TransferProcessActions } from "shared/src/components/actions/TransferProcessActions.tsx";
import { TransferProcessBusinessActions } from "shared/src/components/actions/TransferProcessBusinessActions.tsx";
import { ArrowRight } from "lucide-react";
import { useMemo, useState } from "react";

type ActionsMode = "business" | "standard";
import { mergeStateAndAttribute } from "shared/src/lib/utils.ts";
import { PageLayout } from "shared/src/components/layout/PageLayout";
import { PageHeader } from "shared/src/components/layout/PageHeader";
import { PageSection } from "shared/src/components/layout/PageSection";
import { useGetTransferProcesses } from "shared/src/data/orval/transfers/transfers";
import { Skeleton } from "shared/components/ui/skeleton";
import { useTableQueryParams } from "shared/src/hooks/useTableQueryParams";

import { keepPreviousData } from "@tanstack/react-query";

/**
 * Route for listing transfer processes.
 */
export const Route = createFileRoute("/transfer-process/")({
  component: RouteComponent,
});

function RouteComponent() {
  const [mode, setMode] = useState<ActionsMode>("business");
  const { params: queryParams, apiParams, onQueryChange } = useTableQueryParams({
    defaultLimit: 10,
    defaultSort: "created_at_desc",
  });

  const {
    data: transferProcessesResponse,
    isLoading: isTransferProcessesLoading,
    isFetching,
  } = useGetTransferProcesses({
    query: {
      queryKey: ["/transfers/transfer-processes", apiParams],
      placeholderData: keepPreviousData,
    },
    request: {
      params: apiParams,
    },
  });
  const transferProcesses =
    transferProcessesResponse?.status === 200 ? transferProcessesResponse.data : undefined;

  if (isTransferProcessesLoading && !transferProcesses) {
    return (
      <PageLayout>
        <PageHeader title="Transfer Processes" badge={<Skeleton className="h-8 w-48" />} />
        <div>Loading...</div>
      </PageLayout>
    );
  }

  return (
    <PageLayout>
      <PageHeader title="Transfer Processes" className="flex items-center justify-between">
        <div className="flex gap-1 mt-2 p-0.5 rounded-md bg-ink/5 w-fit text-xs">
          <button
            onClick={() => setMode("business")}
            className={`px-3 py-1 rounded transition-colors ${
              mode === "business"
                ? "bg-ink/15 text-ink font-medium"
                : "text-ink/50 hover:text-ink/80"
            }`}
          >
            Business
          </button>
          <button
            onClick={() => setMode("standard")}
            className={`px-3 py-1 rounded transition-colors ${
              mode === "standard"
                ? "bg-ink/15 text-ink font-medium"
                : "text-ink/50 hover:text-ink/80"
            }`}
          >
            Standard
          </button>
        </div>
      </PageHeader>
      <PageSection>
        <DataTable
          className="text-sm"
          data={transferProcesses}
          serverSide={true}
          loading={isFetching}
          queryParams={queryParams}
          onQueryChange={onQueryChange}
          keyExtractor={(tp) => tp.id!}
          searchPlaceholder="Filter transfers by process ID, role, or state..."
          filters={[
            {
              id: "state",
              label: "State",
              options: [
                { label: "All States", value: "all" },
                { label: "Requested", value: "REQUESTED" },
                { label: "Started", value: "STARTED" },
                { label: "Completed", value: "COMPLETED" },
                { label: "Suspended", value: "SUSPENDED" },
                { label: "Terminated", value: "TERMINATED" },
              ],
              filterFn: (tp, val) => (tp.state ?? "").toUpperCase().includes(val),
            },
            {
              id: "role",
              label: "Role",
              accessorKey: "role",
              options: [
                { label: "All Roles", value: "all" },
                { label: "Provider", value: "Provider" },
                { label: "Consumer", value: "Consumer" },
              ],
            },
          ]}
          columns={[
            {
              header: "Process ID",
              accessorKey: "id",
              cell: (tp) => <Badge variant="info">{formatIdentifier(tp.id)}</Badge>,
            },
            {
              header: "State",
              accessorKey: "state",
              cell: (tp) => (
                <Badge variant="status" state={tp.state}>
                  {mergeStateAndAttribute(tp.state ?? "", tp.stateAttribute ?? "")}
                </Badge>
              ),
            },
            {
              header: "Role",
              accessorKey: "role",
              cell: (tp) => <Badge variant="info">{tp.role}</Badge>,
            },
            {
              header: "Created at",
              accessorKey: "createdAt",
              sortKey: "created_at",
              sortValue: (tp) => new Date(tp.createdAt!).getTime(),
              cell: (tp) => <FormatDate date={tp.createdAt} />,
            },
            {
              header: "Updated at",
              accessorKey: "updatedAt",
              sortKey: "updated_at",
              sortValue: (tp) => new Date(tp.updatedAt!).getTime(),
              cell: (tp) => <FormatDate date={tp.updatedAt} />,
            },
            {
              header: "Actions",
              sortable: false,
              searchable: false,
              cell: (tp) =>
                mode === "business" ? (
                  <TransferProcessBusinessActions process={tp} tiny={true} />
                ) : (
                  <TransferProcessActions process={tp} tiny={true} />
                ),
            },
            {
              header: "Link",
              sortable: false,
              searchable: false,
              cell: (tp) => (
                <Link
                  to="/transfer-process/$transferProcessId"
                  params={{ transferProcessId: tp.id! }}
                >
                  <Button variant="link">
                    See details
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
