import { createFileRoute, Link } from "@tanstack/react-router";
import { formatUrn } from "shared/src/lib/utils";
import { DataTable } from "shared/src/components/DataTable";
import { FormatDate } from "shared/src/components/ui/format-date";
import { Button } from "shared/src/components/ui/button.tsx";
import { Badge, BadgeState } from "shared/src/components/ui/badge.tsx";
import { Input } from "shared/src/components/ui/input.tsx";
import { useGetNegotiationProcesses } from "shared/src/data/orval/negotiations/negotiations";
import { ContractNegotiationActions } from "shared/src/components/actions/ContractNegotiationActions";
import { useMemo } from "react";
import { ArrowRight } from "lucide-react";
import { PageLayout } from "shared/src/components/layout/PageLayout";
import { PageHeader } from "shared/src/components/layout/PageHeader";
import { PageSection } from "shared/src/components/layout/PageSection";

const RouteComponent = () => {
  const { data: cnProcessesData } = useGetNegotiationProcesses();

  const cnProcesses = cnProcessesData?.status === 200 ? cnProcessesData.data : [];

  const cnProcessesSorted = useMemo(() => {
    if (!cnProcesses) return [];
    return [...cnProcesses].sort((a, b) => {
      // @ts-ignore
      return new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime();
    });
  }, [cnProcesses]);

  return (
    <PageLayout>
      <PageHeader title="Contract Negotiations" />
      <PageSection>
        <DataTable
          className="text-sm"
          data={cnProcessesSorted ?? []}
          keyExtractor={(p) => p.id}
          columns={[
            {
              header: "id",
              cell: (p) => <Badge variant={"info"}>{formatUrn(p.id)}</Badge>,
            },
            {
              header: "Role",
              cell: (p) => <Badge variant={"info"}>{p.role}</Badge>,
            },
            {
              header: "State",
              cell: (p) => (
                <Badge variant={"status"} state={p.state as BadgeState}>
                  {p.state.replace("dspace:", "")}
                </Badge>
              ),
            },
            {
              header: "State Attribute",
              cell: (p) => (
                <Badge variant={"status"} state={p.stateAttribute as BadgeState}>
                  {p.state.replace("dspace:", "")}
                </Badge>
              ),
            },
            {
              header: "Peer",
              cell: (p) => <p className="">{formatUrn(p.associatedAgentPeer)}</p>,
            },
            {
              header: "CreatedAt",
              cell: (p) => <FormatDate date={p.createdAt} />,
            },
            {
              header: "Actions",
              cell: (p) => <ContractNegotiationActions process={p} tiny={true} />,
            },
            {
              header: "Link",
              cell: (p) => (
                <Link to="/contract-negotiation/$cnProcess" params={{ cnProcess: p.id }}>
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
};

/**
 * Route for listing contract negotiation processes.
 */
export const Route = createFileRoute("/contract-negotiation/")({
  component: RouteComponent,
  pendingComponent: () => <div>Loading...</div>,
});
