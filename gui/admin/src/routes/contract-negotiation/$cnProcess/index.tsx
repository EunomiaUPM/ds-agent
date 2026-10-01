import { createFileRoute } from "@tanstack/react-router";
import { ContractNegotiationActions } from "shared/src/components/actions/ContractNegotiationActions";
import { InfoList } from "shared/src/components/ui/info-list";
import { FormatDate } from "shared/src/components/ui/format-date";
import { PageLayout } from "shared/src/components/layout/PageLayout";
import { PageSection } from "shared/src/components/layout/PageSection";
import { InfoGrid } from "shared/src/components/layout/InfoGrid";
import { useGetNegotiationProcessById } from "shared/data/orval/negotiations/negotiations.ts";
import { PageHeader } from "shared/components/layout/PageHeader.tsx";
import { Skeleton } from "shared/components/ui/skeleton.tsx";
import { Badge } from "shared/components/ui/badge.tsx";
import { GeneralErrorComponent } from "@/components/GeneralErrorComponent.tsx";
import { formatIdentifier } from "shared/lib/utils.ts";
import { ProcessMessagesTable } from "shared/src/components/ProcessMessagesTable";

const RouteComponent = () => {
  const { cnProcess } = Route.useParams();
  const { data: process, isLoading: isNegotiationProcessLoading } =
    useGetNegotiationProcessById(cnProcess);

  if (isNegotiationProcessLoading) {
    return (
      <PageLayout>
        <PageHeader
          title="Contract Negotiation Process"
          badge={<Skeleton className="h-8 w-48" />}
        />
        <div>Loading...</div>
      </PageLayout>
    );
  }

  // handle error
  if (!process || process.status !== 200) {
    return (
      <GeneralErrorComponent
        error={new Error("Contract negotiation process not found")}
        reset={() => {}}
      />
    );
  }

  return (
    <PageLayout>
      <PageHeader
        title="Contract Negotiation Process"
        badge={
          <Badge variant="status" state={process.data.state}>
            {process.data.state}
          </Badge>
        }
      />

      <InfoGrid className="mb-6">
        <PageSection title="Contract Negotiation Info">
          <InfoList
            items={[
              {
                label: "Provider PID",
                value: {
                  type: "urn",
                  value: formatIdentifier(process.data.id),
                },
              },
              { label: "State", value: { type: "status", value: process.data.state } },
              {
                label: "Created At",
                value: { type: "custom", content: <FormatDate date={process.data.createdAt} /> },
              },
            ]}
          />
        </PageSection>
      </InfoGrid>

      <PageSection title="Exchange Messages" className="mb-6">
        <ProcessMessagesTable
          messages={process.data.messages || []}
          processId={process.data.id}
          title="Negotiation Messages"
        />
      </PageSection>

      {/* ACTIONS */}
      <ContractNegotiationActions process={process.data} tiny={false} />
    </PageLayout>
  );
};

/**
 * Route for displaying contract negotiation process details.
 */
export const Route = createFileRoute("/contract-negotiation/$cnProcess/")({
  component: RouteComponent,
});

