import { createFileRoute, Link } from "@tanstack/react-router";
import { useQuery } from "@tanstack/react-query";
import { customInstance } from "shared/src/data/orval-mutator";
import { DataTable } from "shared/src/components/DataTable";
import { FormatDate } from "shared/src/components/ui/format-date";
import { Button } from "shared/src/components/ui/button";
import { Badge } from "shared/src/components/ui/badge";
import { ArrowRight, Plus, ArrowUpDown, ArrowUp, ArrowDown } from "lucide-react";
import { useState, useMemo } from "react";
import { PageLayout } from "shared/src/components/layout/PageLayout";
import { PageHeader } from "shared/src/components/layout/PageHeader";
import { PageSection } from "shared/src/components/layout/PageSection";
import { formatIdentifier } from "shared/src/lib/utils";

const truncateId = (id?: string) => {
  if (!id) return "N/A";
  if (id.length <= 40) return id;
  return `${id.slice(0, 20)}...${id.slice(-15)}`;
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
              header:
                /* (
                <Button
                  variant="ghost"
                  onClick={() => handleSort("provider_slug")}
                  className="p-0 h-auto font-semibold"
                >
                  Provider Name {getSortIcon("provider_slug")}
                </Button>
              ), */ "Provider Name",
              cell: (r) => r.provider_slug || "-",
            },
            {
              header:
                /* (
                <Button
                  variant="ghost"
                  onClick={() => handleSort("id")}
                  className="p-0 h-auto font-semibold"
                >
                  Request ID {getSortIcon("id")}
                </Button>
              ), */ "Request ID",
              cell: (r) => <Badge variant={"info"}>{formatIdentifier(r.id)}</Badge>,
            },
            {
              header:
                /* (
                <Button
                  variant="ghost"
                  onClick={() => handleSort("provider_id")}
                  className="p-0 h-auto font-semibold"
                >
                  Provider DID {getSortIcon("provider_id")}
                </Button>
              ), */ "Provider DID",
              cell: (r) => (
                <div className="flex flex-col gap-1">
                  <Badge variant={"info"}>{formatIdentifier(r.provider_id)}</Badge>
                </div>
              ),
            },
            {
              header:
                /* (
                <Button variant="ghost" onClick={() => handleSort('status')} className="p-0 h-auto font-semibold">
                  Status {getSortIcon('status')}
                </Button>
              ), */ "Status",
              cell: (r) => (
                <Badge variant={"status"} state={r.status}>
                  {r.status || "-"}
                </Badge>
              ),
            },
            {
              header:
                /* (
                <Button
                  variant="ghost"
                  onClick={() => handleSort("created_at")}
                  className="p-0 h-auto font-semibold"
                >
                  Created at {getSortIcon("created_at")}
                </Button>
              ), */ "Created at",
              cell: (r) => (r.created_at ? <FormatDate date={r.created_at} /> : "-"),
            },
            {
              header: "Details",
              cell: (r) => (
                // @ts-ignore
                <Link to="/providers/request-details" search={{ requestId: r.id }}>
                  <Button variant="link" size={"sm"}>
                    Details
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
