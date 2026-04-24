import { createFileRoute, Link } from "@tanstack/react-router";
import { useQuery } from "@tanstack/react-query";
import { customInstance } from "shared/src/data/orval-mutator";
import { DataTable } from "shared/src/components/DataTable";
import { FormatDate } from "shared/src/components/ui/format-date";
import { Button } from "shared/src/components/ui/button";
import { Badge } from "shared/src/components/ui/badge";
import { ArrowRight, Plus } from "lucide-react";
import { PageLayout } from "shared/src/components/layout/PageLayout";
import { PageHeader } from "shared/src/components/layout/PageHeader";
import { PageSection } from "shared/src/components/layout/PageSection";
import { formatUrn } from "shared/src/lib/utils";

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
      customInstance<{ status: number; data: OnboardRequest[] }>("/onboard/request/all", { method: "GET" })
  });

  const requests = response?.data || [];

  return (
    <PageLayout>
      <PageHeader title="Provider Onboarding">
        <div className="flex justify-end mb-4">
          <Link to="/providers/new">
            <Button>
              <Plus className="mr-2 h-4 w-4" />
              New Provider
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
              header: "Request ID",
              cell: (r) => <Badge variant={"info"}>{formatUrn(r.id)}</Badge>,
            },
            {
              header: "Provider DID",
              cell: (r) => (
                <div className="flex flex-col gap-1">
                  <Badge variant={"info"}>{formatUrn(r.provider_id)}</Badge>
                </div>
              ),
            },
            {
              header: "Provider Name",
              cell: (r) => r.provider_slug || "-",
            },
            {
              header: "Endpoint",
              cell: (r) => (
                <span className="text-xs font-mono text-muted-foreground truncate max-w-[200px] block">
                  {r.grant_endpoint}
                </span>
              ),
            },
            {
              header: "Status",
              cell: (r) => (
                <Badge 
                  variant={"status"} 
                  state={r.status === "Approved" || r.status === "Finalized" ? "ACTIVE" : "PAUSE"}
                >
                  {r.status || "-"}
                </Badge>
              ),
            },
            {
              header: "Created at",
              cell: (r) => (r.created_at ? <FormatDate date={r.created_at} /> : "-"),
            },
            {
              header: "Details",
              cell: (r) => (
                // @ts-ignore
                <Link to="/providers/request-details" search={{ requestId: r.id }}>
                  <Button variant="link">
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
