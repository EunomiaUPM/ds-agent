import { ArrowRight, Inbox } from "lucide-react";
import { useMemo, useState } from "react";
import { DataTable } from "shared/src/components/DataTable";
import { SortableHeader, SortConfig } from "shared/src/components/SortableHeader";
import { PageSection } from "shared/src/components/layout/PageSection";
import { Badge } from "shared/src/components/ui/badge";
import { Button } from "shared/src/components/ui/button";
import { FormatDate } from "shared/src/components/ui/format-date";
import { formatUrn } from "shared/src/lib/utils";
import { customInstance } from "shared/src/data/orval-mutator";

import { useQuery } from "@tanstack/react-query";
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

export const Route = createFileRoute("/connections/received/")({
  component: ReceivedConnectionsPage,
});

function ReceivedConnectionsPage() {
  const { data: response } = useQuery({
    queryKey: ["gate-received"],
    queryFn: () =>
      customInstance<{ status: number; data: RecvGrant[] }>("/gate/request/all", {
        method: "GET",
      }),
  });

  const [sortConfig, setSortConfig] = useState<SortConfig<keyof RecvGrant & string> | null>(null);

  const requests = useMemo(() => {
    const items = Array.isArray(response?.data) ? [...response.data] : [];
    if (sortConfig !== null) {
      items.sort((a, b) => {
        const aVal = a[sortConfig.key];
        const bVal = b[sortConfig.key];
        if (aVal === bVal) return 0;
        if (aVal === null || aVal === undefined) return sortConfig.direction === "asc" ? -1 : 1;
        if (bVal === null || bVal === undefined) return sortConfig.direction === "asc" ? 1 : -1;
        const aString = String(aVal).toLowerCase();
        const bString = String(bVal).toLowerCase();
        if (aString < bString) return sortConfig.direction === "asc" ? -1 : 1;
        if (aString > bString) return sortConfig.direction === "asc" ? 1 : -1;
        return 0;
      });
    }
    return items;
  }, [response?.data, sortConfig]);

  const handleSort = (key: keyof RecvGrant & string) => {
    let direction: "asc" | "desc" = "asc";
    if (sortConfig && sortConfig.key === key && sortConfig.direction === "asc") {
      direction = "desc";
    }
    setSortConfig({ key, direction });
  };

  return (
    <PageSection title="Incoming Requests">
      <p className="text-xs text-muted-foreground mb-4 flex items-center gap-2">
        <Inbox className="h-3 w-3" />
        Connection requests sent to this agent by external peers.
      </p>
      <DataTable
        className="text-sm text-white"
        data={requests}
        keyExtractor={(r) => r.id}
        emptyMessage="No incoming connections yet"
        columns={[
          {
            header: (
              <SortableHeader
                label="Peer"
                sortKey="participant_nick"
                sortConfig={sortConfig}
                onSort={handleSort}
              />
            ),
            cell: (r) => r.participant_nick || "-",
          },
          {
            header: "Request ID",
            cell: (r) => <Badge variant={"info"}>{formatUrn(r.id)}</Badge>,
          },
          {
            header: (
              <SortableHeader
                label="Status"
                sortKey="status"
                sortConfig={sortConfig}
                onSort={handleSort}
              />
            ),
            cell: (r) => (
              <Badge variant={"status"} state={r.status}>
                {r.status || "-"}
              </Badge>
            ),
          },
          {
            header: (
              <SortableHeader
                label="Received at"
                sortKey="created_at"
                sortConfig={sortConfig}
                onSort={handleSort}
              />
            ),
            cell: (r) => (r.created_at ? <FormatDate date={r.created_at} /> : "-"),
          },
          {
            header: "Details",
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
