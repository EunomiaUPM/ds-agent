import { ArrowRight, Plus } from "lucide-react";
import { useEffect, useState } from "react";
import { DataTable } from "@eunomia/ui";
import { PageHeader } from "@eunomia/ui";
import { PageLayout } from "@eunomia/ui";
import { PageSection } from "@eunomia/ui";
import { Badge } from "@eunomia/ui";
import { Button } from "@eunomia/ui";
import { FormatDate } from "@eunomia/ui";
import { keepPreviousData, useQuery } from "@tanstack/react-query";
import { customInstance } from "shared/src/data/orval-mutator";
import { formatIdentifier, getFriendlyVCType } from "@eunomia/ui";
import { useTableQueryParams } from "@eunomia/ui";

import { createFileRoute, Link } from "@tanstack/react-router";
import { useGetAllParticipants } from "shared/src/data/orval/participants/participants";
import { WizardEndDialog } from "@eunomia/ui";

interface SentGrant {
  id: string;
  participant_id: string;
  participant_nick: string;
  grant_endpoint: string;
  kind: string;
  status: string;
  final_token?: string | null;
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

/**
 * Route for listing all VC requests to an authority.
 */
export const Route = createFileRoute("/authority/")({
  component: AuthorityRequestsPage,
});

function AuthorityRequestsPage() {
  const { params: queryParams, apiParams, onQueryChange } = useTableQueryParams({
    defaultLimit: 10,
    defaultSort: "created_at_desc",
  });

  const { data: response, isFetching } = useQuery({
    queryKey: ["vc-requests-list", apiParams],
    queryFn: () =>
      customInstance<any>("/vc-request/all", {
        method: "GET",
        params: apiParams,
      }),
    placeholderData: keepPreviousData,
  });
  const { data: participantsResponse } = useGetAllParticipants();

  const [showCongrats, setShowCongrats] = useState(false);
  const requests = response?.status === 200 ? response.data : [];
  const requestCount = Array.isArray(requests)
    ? requests.length
    : Array.isArray((requests as any)?.items)
      ? (requests as any).items.length
      : 0;

  useEffect(() => {
    try {
      const justJoined = sessionStorage.getItem("justJoinedDataspace");
      if (justJoined === "true" && requestCount === 0) {
        setShowCongrats(true);
        sessionStorage.removeItem("justJoinedDataspace");
      }
    } catch (e) {
      // ignore storage errors
    }
  }, [participantsResponse, requestCount]);

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
      <PageHeader
        title="VC Requests"
        description="Verifiable credentials this connector asked authorities for."
        actions={
          <Link to="/authority/new">
            <Button>
              <Plus />
              Request credential
            </Button>
          </Link>
        }
      />

      <PageSection>
        <DataTable<SentGrant>
          className="text-sm"
          data={requests}
          serverSide={true}
          loading={isFetching}
          queryParams={queryParams}
          onQueryChange={onQueryChange}
          keyExtractor={(a) => a.id}
          emptyMessage="No credential requests yet"
          searchPlaceholder="Filter credential requests by authority, ID, or status..."
          filters={[
            {
              id: "status",
              label: "Status",
              options: [
                { label: "Processing", value: "Processing" },
                { label: "Pending", value: "Pending" },
                { label: "Approved", value: "Approved" },
                { label: "Rejected", value: "Rejected" },
                { label: "Finalized", value: "Finalized" },
              ],
            },
          ]}
          columns={[
            {
              header: "Authority",
              accessorKey: "participant_nick",
              cell: (a) => a.participant_nick || "-",
            },
            {
              header: "Request ID",
              accessorKey: "id",
              cell: (a) => <Badge variant="info">{formatIdentifier(a.id)}</Badge>,
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
              accessorKey: "auto",
              cell: (a) =>
                a.auto ? (
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
              cell: (a) => (
                <Badge variant="status" state={a.status}>
                  {a.status || "-"}
                </Badge>
              ),
            },
            {
              header: "Created at",
              accessorKey: "created_at",
              sortKey: "created_at",
              cell: (a) => (a.created_at ? <FormatDate date={a.created_at} /> : "-"),
            },
            {
              header: "Details",
              sortable: false,
              searchable: false,
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
