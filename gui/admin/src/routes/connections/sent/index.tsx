import { ArrowRight, Plus } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import WizardEndDialog from "shared/src/components/WizardEndDialog";
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
 * Maps the backend `sent_grant::Model` for AccessToken grants (peer connections we initiated).
 */
export interface SentGrant {
  id: string;
  participant_id: string;
  participant_nick: string;
  grant_endpoint: string;
  kind: string;
  status: string;
  token?: string | null;
  vc_type_config?: string[] | null;
  vc_uri?: string | null;
  as_assigned_id?: string | null;
  auto: boolean;
  created_at: string;
  ended_at?: string | null;
}

export const Route = createFileRoute("/connections/sent/")({
  component: SentConnectionsPage,
});

function SentConnectionsPage() {
  const { data: response } = useQuery({
    queryKey: ["peer-connection-sent"],
    queryFn: () =>
      customInstance<{ status: number; data: SentGrant[] }>("/peer-connection/request/all", {
        method: "GET",
      }),
  });

  const [sortConfig, setSortConfig] = useState<SortConfig<keyof SentGrant & string> | null>(null);

  const [showCongrats, setShowCongrats] = useState(false);

  useEffect(() => {
    try {
      const justJoined = sessionStorage.getItem("JustAuthenticatedProvider");
      const items = Array.isArray(response?.data) ? response.data : [];
      if (justJoined === "true" && items.length === 1) {
        setShowCongrats(true);
        sessionStorage.removeItem("JustAuthenticatedProvider");
      }
    } catch (e) {
      // ignore storage errors
    }
  }, [response]);

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

  const handleSort = (key: keyof SentGrant & string) => {
    let direction: "asc" | "desc" = "asc";
    if (sortConfig && sortConfig.key === key && sortConfig.direction === "asc") {
      direction = "desc";
    }
    setSortConfig({ key, direction });
  };

  return (
    <>
      <WizardEndDialog
        open={showCongrats}
        onClose={() => setShowCongrats(false)}
        title={"Congratulations"}
        sectionTitle="Connection with Participant Tutorial Completed"
        content={
          <>
            Congratulations — you are now connected to a new participant.
            <br />
            Now you can explore their catalog and datasets.
          </>
        }
        actionHref={"/catalog"}
        actionLabel={"See catalog"}
      />

      <PageSection
        title="Outgoing Requests"
        action={
          <Link to="/connections/sent/new">
            <Button size="sm">
              <Plus className="mr-2 h-4 w-4" />
              New connection
            </Button>
          </Link>
        }
      >
        <DataTable
          className="text-sm text-white"
          data={requests}
          keyExtractor={(r) => r.id}
          columns={[
            {
              header: (
                <SortableHeader
                  label="Provider"
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
              header: "Auto",
              cell: (r) =>
                r.auto ? (
                  <Badge variant="default" className="text-[10px]">
                    ON
                  </Badge>
                ) : (
                  <Badge variant="info" className="text-[10px]">
                    OFF
                  </Badge>
                ),
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
                  label="Created at"
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
                <Link to="/connections/sent/request-details" search={{ requestId: r.id }}>
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
    </>
  );
}
