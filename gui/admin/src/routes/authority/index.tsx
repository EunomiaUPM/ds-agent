import { ArrowRight, Plus } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { DataTable } from "shared/src/components/DataTable";
import { SortableHeader, SortConfig } from "shared/src/components/SortableHeader";
import { PageHeader } from "shared/src/components/layout/PageHeader";
import { PageLayout } from "shared/src/components/layout/PageLayout";
import { PageSection } from "shared/src/components/layout/PageSection";
import { Badge } from "shared/src/components/ui/badge";
import { Button } from "shared/src/components/ui/button";
import { FormatDate } from "shared/src/components/ui/format-date";
import { useQuery } from "@tanstack/react-query";
import { customInstance } from "shared/src/data/orval-mutator";
import { formatIdentifier, getFriendlyVCType } from "shared/src/lib/utils";

import { createFileRoute, Link } from "@tanstack/react-router";
import { useGetAllParticipants } from "shared/src/data/orval/participants/participants";
import WizardEndDialog from "shared/src/components/WizardEndDialog";

interface SentGrant {
  id: string;
  participant_id: string;
  participant_nick: string;
  grant_endpoint: string;
  kind: string;
  status: string;
  token?: string | null;
  /**
   * Each entry is the string id of a VcTypeConfig, e.g. "gx_VatId_jwt_vc_json".
   * Comes from the backend's `impl_serde_via_str!(VcTypeConfig)`.
   */
  vc_type_config?: string[] | null;
  vc_uri?: string | null;
  as_assigned_id?: string | null;
  auto: boolean;
  created_at: string;
  ended_at?: string | null;
}

interface GrantsResponse {
  status: number;
  data: SentGrant[];
}

/**
 * Route for listing all VC requests to an authority.
 */
export const Route = createFileRoute("/authority/")({
  component: AuthorityRequestsPage,
});

function AuthorityRequestsPage() {
  const { data: response } = useQuery({
    queryKey: ["vc-requests-list"],
    queryFn: () => customInstance<GrantsResponse>("/vc-request/all", { method: "GET" }),
  });
  const { data: participantsResponse } = useGetAllParticipants();

  const [showCongrats, setShowCongrats] = useState(false);
  const rawRequests = response?.status === 200 ? response.data : [];
  const [sortConfig, setSortConfig] = useState<SortConfig<keyof SentGrant & string> | null>(null);

  const requests = useMemo(() => {
    const items = [...rawRequests];
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
  }, [rawRequests, sortConfig]);

  const handleSort = (key: keyof SentGrant & string) => {
    let direction: "asc" | "desc" = "asc";
    if (sortConfig && sortConfig.key === key && sortConfig.direction === "asc") {
      direction = "desc";
    }
    setSortConfig({ key, direction });
  };

  useEffect(() => {
    try {
      const justJoined = sessionStorage.getItem("justJoinedDataspace");
      if (justJoined === "true" && requests.length === 0) {
        setShowCongrats(true);
        sessionStorage.removeItem("justJoinedDataspace");
      }
    } catch (e) {
      // ignore storage errors
    }
  }, [participantsResponse, requests.length]);

  return (
    <PageLayout>
      <WizardEndDialog
        open={showCongrats}
        sectionTitle="Dataspace Sign Up Tutorial Completed"
        onClose={() => setShowCongrats(false)}
        title={"Congratulations"}
        content={
          <>
            Congratulations, you are part now of the Dataspace of Heimdall
            <br />
            You can now browse the catalogs in the dataspace.
          </>
        }
        actionHref={"/catalog/"}
        actionLabel={"See dataspace"}
      />
      <PageHeader title="Credential Requests">
        <div className="flex justify-end mb-4">
          <Link to="/authority/new">
            <Button>
              <Plus className="mr-2 h-4 w-4" />
              Request New Credential
            </Button>
          </Link>
        </div>
      </PageHeader>

      <PageSection>
        <DataTable
          className="text-sm"
          data={requests}
          keyExtractor={(a) => a.id}
          emptyMessage="No credential requests yet"
          columns={[
            {
              header: (
                <SortableHeader
                  label="Authority"
                  sortKey="participant_nick"
                  sortConfig={sortConfig}
                  onSort={handleSort}
                />
              ),
              cell: (a) => a.participant_nick || "-",
            },
            {
              header: "Request ID",
              cell: (a) => <Badge variant={"info"}>{formatIdentifier(a.id)}</Badge>,
            },
            {
              header: "Credential Types",
              cell: (a) => {
                const configs = a.vc_type_config ?? [];
                if (configs.length === 0) return <span className="text-muted-foreground">—</span>;
                return (
                  <div className="flex flex-wrap gap-1">
                    {configs.map((cfg, idx) => (
                      <Badge key={idx} variant="role">
                        {getFriendlyVCType(cfg)}
                      </Badge>
                    ))}
                  </div>
                );
              },
            },
            {
              header: "Auto",
              cell: (a) =>
                a.auto ? (
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
              cell: (a) => (
                <Badge variant={"status"} state={a.status}>
                  {a.status || "-"}
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
              cell: (a) => (a.created_at ? <FormatDate date={a.created_at} /> : "-"),
            },
            {
              header: "Details",
              cell: (a) => (
                // @ts-ignore
                <Link to="/authority/request-details" search={{ requestId: a.id }}>
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

