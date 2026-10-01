import { ArrowRight, Inbox } from "lucide-react";
import { DataTable } from "shared/src/components/DataTable";
import { PageSection } from "shared/src/components/layout/PageSection";
import { Badge } from "shared/src/components/ui/badge";
import { Button } from "shared/src/components/ui/button";
import { FormatDate } from "shared/src/components/ui/format-date";
import { formatUrn } from "shared/src/lib/utils";
import { customInstance } from "shared/src/data/orval-mutator";

import { keepPreviousData, useQuery } from "@tanstack/react-query";
import { createFileRoute, Link } from "@tanstack/react-router";

/**
 * Maps the backend `recv_grant::Model` for AccessToken grants we received from peers.
 */
export interface RecvGrant {
  id: string;
  participant_nick: string;
  kind: string;
  token?: string | null;
  vc_type_config?: string[] | null;
  status: string;
  created_at: string;
  ended_at?: string | null;
}

import { useTableQueryParams } from "shared/src/hooks/useTableQueryParams";

export const Route = createFileRoute("/connections/received/")({
  component: ReceivedConnectionsPage,
});

function ReceivedConnectionsPage() {
  const { params: queryParams, apiParams, onQueryChange } = useTableQueryParams({
    defaultLimit: 10,
    defaultSort: "created_at_desc",
  });

  const { data: response, isFetching } = useQuery({
    queryKey: ["gate-received", apiParams],
    queryFn: () =>
      customInstance<{ status: number; data: any }>("/gate/request/all", {
        method: "GET",
        params: apiParams,
      }),
    placeholderData: keepPreviousData,
  });

  return (
    <PageSection title="Incoming Requests">
      <p className="text-xs text-muted-foreground mb-4 flex items-center gap-2">
        <Inbox className="h-3 w-3" />
        Connection requests sent to this agent by external peers.
      </p>
      <DataTable
        className="text-sm text-ink"
        data={response?.data}
        serverSide={true}
        loading={isFetching}
        queryParams={queryParams}
        onQueryChange={onQueryChange}
        keyExtractor={(r) => r.id}
        emptyMessage="No incoming connections yet"
        searchPlaceholder="Filter connections by peer, ID, or status..."
        filters={[
          {
            id: "status",
            label: "Status",
            accessorKey: "status",
            options: [
              { label: "All Statuses", value: "all" },
              { label: "Approved", value: "Approved" },
              { label: "Pending", value: "Pending" },
              { label: "Rejected", value: "Rejected" },
            ],
          },
        ]}
        columns={[
          {
            header: "Peer",
            accessorKey: "participant_nick",
            cell: (r) => r.participant_nick || "-",
          },
          {
            header: "Request ID",
            accessorKey: "id",
            cell: (r) => <Badge variant="info">{formatUrn(r.id)}</Badge>,
          },
          {
            header: "Status",
            accessorKey: "status",
            cell: (r) => (
              <Badge variant="status" state={r.status}>
                {r.status || "-"}
              </Badge>
            ),
          },
          {
            header: "Received at",
            accessorKey: "created_at",
            sortKey: "created_at",
            cell: (r) => (r.created_at ? <FormatDate date={r.created_at} /> : "-"),
          },
          {
            header: "Details",
            sortable: false,
            searchable: false,
            cell: (r) => (
              // @ts-ignore
              <Link to="/connections/received/request-details" search={{ requestId: r.id }}>
                <Button variant="link">
                  Details
                  <ArrowRight />
                </Button>
              </Link>
            ),
          },
        ]}
      />
    </PageSection>
  );
}
