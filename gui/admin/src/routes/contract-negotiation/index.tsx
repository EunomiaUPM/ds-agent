import { createFileRoute, Link } from "@tanstack/react-router";
import { formatIdentifier } from "shared/src/lib/utils";
import { DataTable } from "shared/src/components/DataTable";
import { FormatDate } from "shared/src/components/ui/format-date";
import { Button } from "shared/src/components/ui/button.tsx";
import { Badge, BadgeState } from "shared/src/components/ui/badge.tsx";
import { Input } from "shared/src/components/ui/input.tsx";
import { useGetNegotiationProcesses } from "shared/src/data/orval/negotiations/negotiations";
import { ContractNegotiationActions } from "shared/src/components/actions/ContractNegotiationActions";
import { ContractNegotiationBusinessActions } from "shared/src/components/actions/ContractNegotiationBusinessActions";
import { useMemo, useState } from "react";
import { ArrowRight } from "lucide-react";
import { PageLayout } from "shared/src/components/layout/PageLayout";
import { PageHeader } from "shared/src/components/layout/PageHeader";
import { PageSection } from "shared/src/components/layout/PageSection";
type ActionsMode = "business" | "standard";

const RouteComponent = () => {
  const { data: cnProcessesData } = useGetNegotiationProcesses();
  const [mode, setMode] = useState<ActionsMode>("business");

  const cnProcessesSorted = useMemo(() => {
    const cnProcesses = cnProcessesData?.status === 200 ? cnProcessesData.data : [];
    if (!cnProcesses) return [];
    return [...cnProcesses].sort((a, b) => {
      return new Date(b.createdAt as string).getTime() - new Date(a.createdAt as string).getTime();
    });
  }, [cnProcessesData]);

  return (
    <PageLayout>
      <PageHeader title="Contract Negotiations"  className="flex items-center justify-between">
        <div className="flex gap-1 mt-2 p-0.5 rounded-md bg-white/5 w-fit text-xs">
          <button
            onClick={() => setMode("business")}
            className={`px-3 py-1 rounded transition-colors ${
              mode === "business"
                ? "bg-white/15 text-white font-medium"
                : "text-white/50 hover:text-white/80"
            }`}
          >
            Business
          </button>
          <button
            onClick={() => setMode("standard")}
            className={`px-3 py-1 rounded transition-colors ${
              mode === "standard"
                ? "bg-white/15 text-white font-medium"
                : "text-white/50 hover:text-white/80"
            }`}
          >
            Standard
          </button>
        </div>
      </PageHeader>
      <PageSection>
        <DataTable
          className="text-sm"
          data={cnProcessesSorted}
          keyExtractor={(p) => p.id}
          columns={[
            {
              header: "Process id",
              cell: (p) => <Badge variant={"info"}>{formatIdentifier(p.id)}</Badge>,
            },
            {
              header: "Peer",
              cell: (p) => (
                <p className="flex gap-2 items-baseline">
                  <span className="capitalize min-w-fit">
                    {formatIdentifier(p.associatedAgentPeer, 3)}
                  </span>
                  <span className="text-white/70">as</span>
                  <Badge className="h-fit">{p.role == "Provider" ? "Provider" : "Consumer"}</Badge>
                </p>
              ),
            },
            {
              header: "State",
              cell: (p) => (
                <Badge variant={"status"} state={p.state}>
                  {p.state.replace("dspace:", "")}
                </Badge>
              ),
            },
            {
              header: "Created At",
              cell: (p) => <FormatDate date={p.createdAt} />,
            },
            {
              header: "Actions",
              cell: (p) =>
                mode === "business" ? (
                  <ContractNegotiationBusinessActions process={p} tiny={true} />
                ) : (
                  <ContractNegotiationActions process={p} tiny={true} />
                ),
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
