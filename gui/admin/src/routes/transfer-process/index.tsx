import { createFileRoute, Link } from "@tanstack/react-router";
import { formatUrn } from "shared/src/lib/utils";
import { DataTable } from "shared/src/components/DataTable";
import { FormatDate } from "shared/src/components/ui/format-date";
import { Button } from "shared/src/components/ui/button.tsx";
import { Badge, BadgeState } from "shared/src/components/ui/badge.tsx";
import { Input } from "shared/src/components/ui/input.tsx";
import { TransferProcessActions } from "shared/src/components/actions/TransferProcessActions.tsx";
import { ArrowRight } from "lucide-react";
import { useMemo } from "react";
import { mergeStateAndAttribute } from "shared/src/lib/utils.ts";
import { PageLayout } from "shared/src/components/layout/PageLayout";
import { PageHeader } from "shared/src/components/layout/PageHeader";
import { PageSection } from "shared/src/components/layout/PageSection";
import { useGetTransferProcesses } from "shared/src/data/orval/transfers/transfers";
import { Skeleton } from "shared/components/ui/skeleton";

/**
 * Route for listing transfer processes.
 */
export const Route = createFileRoute("/transfer-process/")({
  component: RouteComponent,
});

function RouteComponent() {
  const { data: transferProcessesResponse, isLoading: isTransferProcessesLoading } =
    useGetTransferProcesses();
  const transferProcesses =
    transferProcessesResponse?.status === 200 ? transferProcessesResponse.data : undefined;
  const transferProcessesSorted = useMemo(() => {
    if (!transferProcesses) return [];
    return [...transferProcesses].sort((a, b) => {
      return new Date(b.createdAt!).getTime() - new Date(a.createdAt!).getTime();
    });
  }, [transferProcesses]);

  if (isTransferProcessesLoading) {
    return (
      <PageLayout>
        <PageHeader title="Transfer Processes" badge={<Skeleton className="h-8 w-48" />} />
        <div>Loading...</div>
      </PageLayout>
    );
  }

  return (
    <PageLayout>
      <PageHeader title="Transfer Processes" />
      <PageSection>
        <DataTable
          className="text-sm"
          data={transferProcessesSorted ?? []}
          keyExtractor={(tp) => tp.id!}
          columns={[
            {
              header: "Provider pid",
              cell: (tp) => <Badge variant={"info"}>{formatUrn(tp.id)}</Badge>,
            },
            {
              header: "State",
              cell: (tp) => (
                <Badge variant={"status"} state={tp.state as BadgeState}>
                  {mergeStateAndAttribute(tp.state ?? "", tp.stateAttribute ?? "")}
                </Badge>
              ),
            },
            {
              header: "Role",
              cell: (tp) => <Badge variant={"info"}>{tp.role}</Badge>,
            },
            {
              header: "Created at",
              cell: (tp) => <FormatDate date={tp.createdAt} />,
            },
            {
              header: "Updated at",
              cell: (tp) => <FormatDate date={tp.updatedAt} />,
            },
            {
              header: "Actions",
              cell: (tp) => <TransferProcessActions process={tp} tiny={true} />,
            },
            {
              header: "Link",
              cell: (tp) => (
                <Link
                  to="/transfer-process/$transferProcessId"
                  params={{ transferProcessId: tp.id! }}
                >
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
