import { ArrowDown, ArrowRight, ArrowUp, ArrowUpDown, Plus } from "lucide-react";
import { useMemo, useState } from "react";
import { DataTable } from "shared/src/components/DataTable";
import { PageHeader } from "shared/src/components/layout/PageHeader";
import { PageLayout } from "shared/src/components/layout/PageLayout";
import { PageSection } from "shared/src/components/layout/PageSection";
import { Badge } from "shared/src/components/ui/badge";
import { Button } from "shared/src/components/ui/button";
import { FormatDate } from "shared/src/components/ui/format-date";
import { customInstance } from "shared/src/data/orval-mutator";

import { useQuery } from "@tanstack/react-query";
import { createFileRoute, Link } from "@tanstack/react-router";

const truncateId = (id?: string) => {
  if (!id) return "N/A";
  // Only apply the 40-char rule if it looks like a DID
  if (id.startsWith("did:") && id.length < 40) return id;
  // Otherwise truncate aggressively for the table
  return `${id.slice(0, 10)}...${id.slice(-8)}`;
};

/**
 * Onboard request model from backend.
 */
export interface OnboardRequest {
  id: string;
  provider_id: string;
  provider_slug: string;
  grant_endpoint: string;
  auto: boolean;
  assigned_id?: string | null;
  token?: string | null;
  status: string;
  vc_uri?: string | null;
  verification_uri?: string | null;
  created_at: string;
  ended_at?: string | null;
}

/**
 * Route for listing all provider onboard requests.
 */
export const Route = createFileRoute("/providers/")({
  component: ProvidersPage,
});

function ProvidersPage() {
  const { data: response, isLoading } = useQuery({
    queryKey: ["onboard-requests"],
    queryFn: () =>
      customInstance<{ status: number; data: OnboardRequest[] }>("/onboard/request/all", {
        method: "GET",
      }),
  });

  const [sortConfig, setSortConfig] = useState<{
    key: keyof OnboardRequest;
    direction: "asc" | "desc";
  } | null>(null);

  const requests = useMemo(() => {
    let sortableRequests = [...(response?.data || [])];
    if (sortConfig !== null) {
      sortableRequests.sort((a, b) => {
        const aVal = a[sortConfig.key];
        const bVal = b[sortConfig.key];

        if (aVal === bVal) return 0;

        if (aVal === null || aVal === undefined) return sortConfig.direction === "asc" ? -1 : 1;
        if (bVal === null || bVal === undefined) return sortConfig.direction === "asc" ? 1 : -1;

        const aString = String(aVal).toLowerCase();
        const bString = String(bVal).toLowerCase();

        if (aString < bString) {
          return sortConfig.direction === "asc" ? -1 : 1;
        }
        if (aString > bString) {
          return sortConfig.direction === "asc" ? 1 : -1;
        }
        return 0;
      });
    }
    return sortableRequests;
  }, [response?.data, sortConfig]);

  const handleSort = (key: keyof OnboardRequest) => {
    let direction: "asc" | "desc" = "asc";
    if (sortConfig && sortConfig.key === key && sortConfig.direction === "asc") {
      direction = "desc";
    }
    setSortConfig({ key, direction });
  };

  const getSortIcon = (key: keyof OnboardRequest) => {
    if (!sortConfig || sortConfig.key !== key)
      return <ArrowUpDown className="ml-2 h-4 w-4 opacity-50" />;
    return sortConfig.direction === "asc" ? (
      <ArrowUp className="ml-2 h-4 w-4" />
    ) : (
      <ArrowDown className="ml-2 h-4 w-4" />
    );
  };

  return (
    <PageLayout>
      <PageHeader title="Provider Sessions">
        <div className="flex justify-end mb-4">
          <Link to="/providers/new">
            <Button>
              <Plus className="mr-2 h-4 w-4" />
              New Session
            </Button>
          </Link>
        </div>
      </PageHeader>
      <PageSection>
        <DataTable
          className="text-sm"
          data={requests}
          keyExtractor={(r) => r.id}
          columns={[
            {
              header: (
                <Button
                  variant="ghost"
                  onClick={() => handleSort("id")}
                  className="p-0 h-auto font-semibold"
                >
                  Request ID {getSortIcon("id")}
                </Button>
              ),
              cell: (r) => <Badge variant={"info"}>{truncateId(r.id)}</Badge>,
            },
            {
              header: (
                <Button
                  variant="ghost"
                  onClick={() => handleSort("provider_slug")}
                  className="p-0 h-auto font-semibold"
                >
                  Provider Name {getSortIcon("provider_slug")}
                </Button>
              ),
              cell: (r) => r.provider_slug || "-",
            },
            {
              header: (
                <Button
                  variant="ghost"
                  onClick={() => handleSort("status")}
                  className="p-0 h-auto font-semibold"
                >
                  Status {getSortIcon("status")}
                </Button>
              ),
              cell: (r) => (
                <Badge variant={"status"} state={r.status}>
                  {r.status || "-"}
                </Badge>
              ),
            },
            {
              header: (
                <Button
                  variant="ghost"
                  onClick={() => handleSort("created_at")}
                  className="p-0 h-auto font-semibold"
                >
                  Created at {getSortIcon("created_at")}
                </Button>
              ),
              cell: (r) => (r.created_at ? <FormatDate date={r.created_at} /> : "-"),
            },
            {
              header: "Details",
              cell: (r) => (
                // @ts-ignore
                <Link to="/providers/request-details" search={{ requestId: r.id }}>
                  <Button variant="link" size="sm">
                    Details
                    <ArrowRight className="ml-2 h-4 w-4" />
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
