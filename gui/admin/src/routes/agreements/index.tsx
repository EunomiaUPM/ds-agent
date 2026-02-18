import { createFileRoute, Link } from "@tanstack/react-router";
import { formatUrn } from "shared/src/lib/utils";
import { useGetAgreements } from "shared/src/data/orval/negotiations/negotiations";
import { DataTable } from "shared/src/components/DataTable";
import { FormatDate } from "shared/src/components/ui/format-date";
import { Button } from "shared/src/components/ui/button.tsx";
import { Badge } from "shared/src/components/ui/badge.tsx";
import { Input } from "shared/src/components/ui/input.tsx";
import { ArrowRight } from "lucide-react";
import { PageLayout } from "shared/src/components/layout/PageLayout";
import { PageHeader } from "shared/src/components/layout/PageHeader";
import { PageSection } from "shared/src/components/layout/PageSection";
import { AgreementActions } from "shared/src/components/actions/AgreementActions"

/**
 * Route for listing all agreements.
 */
export const Route = createFileRoute("/agreements/")({
  component: RouteComponent,
});

function RouteComponent() {
  const { data: response } = useGetAgreements();
  const agreements = response?.status === 200 ? response.data : [];

  return (
    <PageLayout>
      <PageHeader title="Agreements" />
      <PageSection>
        <DataTable
          className="text-sm"
          data={agreements ?? []}
          keyExtractor={(a) => a.id}
          columns={[
            {
              header: "Agreement Id",
              cell: (a) => <Badge variant={"info"}>{formatUrn(a.id)}</Badge>,
            },
            {
              header: "Provider Participant Id",
              cell: (a) => (
                <div className="flex flex-col gap-1">
                  <Badge variant={"info"}>{formatUrn(a.providerParticipantId)}</Badge>
                </div>
              ),
            },
            {
              header: "Consumer Participant Id",
              cell: (a) => (
                <div className="flex flex-col gap-1">
                  <Badge variant={"info"}>{formatUrn(a.consumerParticipantId)}</Badge>
                </div>
              ),
            },
            {
              header: "Status",
              cell: (a) => (
                <Badge variant={"status"} state={a.state ? "ACTIVE" : "PAUSE"}>
                  {a.state}
                </Badge>
              ),
            },
            {
              header: "Created at",
              cell: (a) => <FormatDate date={a.createdAt} />,
            },
            {
              header: "Actions",
              cell: (p) => <AgreementActions process={p} tiny={true} />,
            },
            {
              header: "Link",
              cell: (a) => (
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
