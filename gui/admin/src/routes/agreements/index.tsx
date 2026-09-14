import { createFileRoute, Link } from "@tanstack/react-router";
import { formatIdentifier } from "shared/src/lib/utils";
import { useGetAgreements } from "shared/src/data/orval/negotiations/negotiations";
import { DataTable } from "shared/src/components/DataTable";
import { FormatDate } from "shared/src/components/ui/format-date";
import { Button } from "shared/src/components/ui/button";
import { Badge } from "shared/src/components/ui/badge";
import { ArrowRight } from "lucide-react";
import { PageLayout } from "shared/src/components/layout/PageLayout";
import { PageHeader } from "shared/src/components/layout/PageHeader";
import { PageSection } from "shared/src/components/layout/PageSection";
import { AgreementActions } from "shared/src/components/actions/AgreementActions";
import { useTableQueryParams } from "shared/src/hooks/useTableQueryParams";

import { keepPreviousData } from "@tanstack/react-query";

/**
 * Route for listing all agreements.
 */
export const Route = createFileRoute("/agreements/")({
  component: RouteComponent,
});

function RouteComponent() {
  const { params: queryParams, apiParams, onQueryChange } = useTableQueryParams({
    defaultLimit: 10,
    defaultSort: "created_at_desc",
  });

  const { data: response, isFetching } = useGetAgreements({
    query: {
      queryKey: ["/negotiations/agreements", apiParams],
      placeholderData: keepPreviousData,
    },
    request: {
      params: apiParams,
    },
  });
  const agreements = response?.status === 200 ? response.data : [];

  return (
    <PageLayout>
      <PageHeader title="Agreements" />
      <PageSection>
        <DataTable
          className="text-sm"
          data={agreements ?? []}
          serverSide={true}
          loading={isFetching}
          queryParams={queryParams}
          onQueryChange={onQueryChange}
          keyExtractor={(a) => a.id}
          searchPlaceholder="Filter agreements by ID, participant, or state..."
          filters={[
            {
              id: "state",
              label: "Status",
              options: [
                { label: "All Statuses", value: "all" },
                { label: "Finalized", value: "FINALIZED" },
                { label: "Active", value: "ACTIVE" },
                { label: "Terminated", value: "TERMINATED" },
              ],
            },
          ]}
          columns={[
            {
              header: "Provider",
              accessorKey: "providerParticipantId" as any,
              sortValue: (a: any) => a.providerParticipantId,
              cell: (a: any) => (
                <p className="capitalize">{formatIdentifier(a.providerParticipantId, 3)}</p>
              ),
            },
            {
              header: "Consumer",
              accessorKey: "consumerParticipantId" as any,
              sortValue: (a: any) => a.consumerParticipantId,
              cell: (a: any) => (
                <p className="capitalize">{formatIdentifier(a.consumerParticipantId, 3)}</p>
              ),
            },
            {
              header: "Agreement Id",
              accessorKey: "id" as any,
              cell: (a: any) => <Badge variant="info">{formatIdentifier(a.id)}</Badge>,
            },
            {
              header: "Status",
              accessorKey: "state" as any,
              cell: (a: any) => (
                <Badge variant="status" state={a.state ? "ACTIVE" : "PAUSE"}>
                  {a.state}
                </Badge>
              ),
            },
            {
              header: "Created at",
              accessorKey: "createdAt" as any,
              sortKey: "created_at",
              sortValue: (a: any) => new Date(a.createdAt).getTime(),
              cell: (a: any) => <FormatDate date={a.createdAt} />,
            },
            {
              header: "Actions",
              sortable: false,
              searchable: false,
              cell: (p: any) => <AgreementActions process={p} tiny={true} />,
            },
            {
              header: "Link",
              sortable: false,
              searchable: false,
              cell: (a: any) => (
                <Link to="/agreements/$agreementId" params={{ agreementId: a.id }}>
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
