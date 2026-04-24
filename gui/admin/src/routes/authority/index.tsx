import { createFileRoute, Link } from "@tanstack/react-router";
import { useGetAllVCRequests } from "shared/src/data/orval/vc-request/vc-request";
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
 * Route for listing all VC requests to an authority.
 */
export const Route = createFileRoute("/authority/")({
  component: AuthorityRequestsPage,
});

function AuthorityRequestsPage() {
  const { data: response } = useGetAllVCRequests();
  const requests = response?.status === 200 ? response.data : [];

  return (
    <PageLayout>
      <PageHeader title="Authority Requests">
        <div className="flex justify-end mb-4">
          <Link to="/authority/new">
            <Button>
              <Plus className="mr-2 h-4 w-4" />
              New Request
            </Button>
          </Link>
        </div>
      </PageHeader>
      <PageSection>
        <DataTable
          className="text-sm"
          data={requests ?? []}
          keyExtractor={(a) => a.id}
          columns={[
            {
              header: "Request ID",
              cell: (a) => <Badge variant={"info"}>{formatUrn(a.id)}</Badge>,
            },
            {
              header: "Authority ID",
              cell: (a) => (
                <div className="flex flex-col gap-1">
                  <Badge variant={"info"}>{a.authority_id ? formatUrn(a.authority_id) : "-"}</Badge>
                </div>
              ),
            },
            {
              header: "Authority Name",
              cell: (a) => a.authority_slug || "-",
            },
            {
              header: "VC Type",
              cell: (a) => a.vc_type,
            },
            {
              header: "Status",
              cell: (a) => (
                <Badge variant={"status"} state={a.status === "Approved" ? "ACTIVE" : a.status === "Finalized" ? "ACTIVE" : "PAUSE"}>
                  {a.status || "-"}
                </Badge>
              ),
            },
            {
              header: "Created at",
              cell: (a) => (a.created_at ? <FormatDate date={a.created_at} /> : "-"),
            },
            {
              header: "Link",
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
