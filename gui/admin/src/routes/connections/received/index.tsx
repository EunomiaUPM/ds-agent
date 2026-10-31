import { ArrowRight, Inbox } from "lucide-react";
import { DataTable } from "@eunomia/ui";
import { PageSection } from "@eunomia/ui";
import { Badge } from "@eunomia/ui";
import { Button } from "@eunomia/ui";
import { FormatDate } from "@eunomia/ui";
import { formatUrn, grantBadgeState } from "@eunomia/ui";
import { customInstance } from "shared/src/data/orval-mutator";

import { keepPreviousData, useQuery } from "@tanstack/react-query";
import { createFileRoute, Link } from "@tanstack/react-router";

/**
 * Maps the backend `recv_grant::Model` for AccessToken grants we received from peers.
 */
export interface RecvGrant {
  id: string;
  participant_nick: string;
  /** Role that handles (answers) the grant, e.g. `/admin/upm`; the roles above it too. */
  role: string;
  visibility: string;
  kind: string;
  final_token_hash?: string | null;
  vc_type_config?: string[] | null;
  status: string;
  created_at: string;
  ended_at?: string | null;
}

import { useTableQueryParams } from "@eunomia/ui";

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
              { label: "Pending", value: "Pending" },
              { label: "Approved", value: "Approved" },
              { label: "Finalized", value: "Finalized" },
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
            header: "Handled by",
            accessorKey: "role",
            cell: (r) => <Badge variant="code">{r.role || "-"}</Badge>,
          },
          {
            header: "Visibility",
            accessorKey: "visibility",
            cell: (r) => <Badge variant="info">{r.visibility || "-"}</Badge>,
          },
          {
            header: "Status",
            accessorKey: "status",
            cell: (r) => (
              <Badge variant="status" state={grantBadgeState(r.status)}>
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
