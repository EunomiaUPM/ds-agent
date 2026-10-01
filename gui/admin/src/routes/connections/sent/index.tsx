import { ArrowRight, Plus } from "lucide-react";
import { useEffect, useState } from "react";
import WizardEndDialog from "shared/src/components/WizardEndDialog";
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

import { useTableQueryParams } from "shared/src/hooks/useTableQueryParams";

export const Route = createFileRoute("/connections/sent/")({
  component: SentConnectionsPage,
});

function SentConnectionsPage() {
  const { params: queryParams, apiParams, onQueryChange } = useTableQueryParams({
    defaultLimit: 10,
    defaultSort: "created_at_desc",
  });

  const { data: response, isFetching } = useQuery({
    queryKey: ["peer-connection-sent", apiParams],
    queryFn: () =>
      customInstance<{ status: number; data: any }>("/peer-connection/request/all", {
        method: "GET",
        params: apiParams,
      }),
    placeholderData: keepPreviousData,
  });

  const [showCongrats, setShowCongrats] = useState(false);

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
          className="text-sm text-ink"
          data={response?.data}
          serverSide={true}
          loading={isFetching}
          queryParams={queryParams}
          onQueryChange={onQueryChange}
          keyExtractor={(r) => r.id}
          emptyMessage="No outgoing connections yet"
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
              header: "Provider",
              accessorKey: "participant_nick",
              cell: (r) => r.participant_nick || "-",
            },
            {
              header: "Request ID",
              accessorKey: "id",
              cell: (r) => <Badge variant="info">{formatUrn(r.id)}</Badge>,
            },
            {
              header: "Auto",
              accessorKey: "auto",
              cell: (r) =>
                r.auto ? (
                  <Badge variant="default" className="text-xs">
                    ON
                  </Badge>
                ) : (
                  <Badge variant="info" className="text-xs">
                    OFF
                  </Badge>
                ),
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
              header: "Created at",
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
