import { createFileRoute } from "@tanstack/react-router";
import { formatUrn } from "shared/src/lib/utils.ts";
import { PageLayout } from "shared/src/components/layout/PageLayout";
import { PageHeader } from "shared/src/components/layout/PageHeader";
import { PageSection } from "shared/src/components/layout/PageSection";

// Components
import Heading from "../../../../../shared/src/components/ui/heading.tsx";
import { Badge, BadgeRole } from "shared/src/components/ui/badge.tsx";
import { InfoList } from "shared/src/components/ui/info-list";
import { useGetParticipantById } from "shared/data/orval/participants/participants.ts";
import { GeneralErrorComponent } from "@/components/GeneralErrorComponent.tsx";
import { useGetAgreementsByParticipantId } from "shared/data/orval/negotiations/negotiations.ts";
import { Skeleton } from "shared/src/components/ui/skeleton";

// Icons

/**
 * Route for displaying individual participant details.
 */
export const Route = createFileRoute("/participants/$participantId/")({
  component: RouteComponent,
});

function RouteComponent() {
  const { participantId } = Route.useParams();
  const {
    data: participant,
    isLoading: isParticipantLoading,
    isError: isParticipantError,
    error: participantError
  } = useGetParticipantById(participantId);
  const {
    data: agreements,
    isLoading: isAgreementsLoading,
    isError: isAgreementsError,
    error: agreementsError
  } = useGetAgreementsByParticipantId(participantId);

  const scopedListItemKeyClasses = "basis-[28%]";

  if (isParticipantLoading || isAgreementsLoading) {
    return (
      <PageLayout>
        <PageHeader
          title="Participant with id"
          badge={<Skeleton className="h-8 w-48" />}
        />
        <div>Loading...</div>
      </PageLayout>
    );
  }

  if (isParticipantError || !participant || participant.status !== 200) {
    const error = participantError instanceof Error ? participantError : new Error("Participant not found");
    return <GeneralErrorComponent error={error} reset={() => { }} />;
  }

  if (isAgreementsError || !agreements || agreements.status !== 200) {
    const error = agreementsError instanceof Error ? agreementsError : new Error("Agreements not found");
    return <GeneralErrorComponent error={error} reset={() => { }} />;
  }

  return (
    <PageLayout>
      {/* Page Header */}
      <PageHeader
        title="Participant with id"
        badge={
          <Badge
            variant={"info"}
            size={"lg"}
            className="max-w-[50%] truncate text-overflow-ellipsis"
          >
            {formatUrn(participant.data.participant_id)}
          </Badge>
        }
      />
      {/* Page content */}
      <div className="bg-blue-500/0">
        {/* Div Participant Info */}
        <PageSection title="Participant info:">
          <div className="max-w-screen-md bg-green-500/0">
            <InfoList
              items={[
                {
                  label: "Participant ID",
                  value: { type: "urn", value: participant.data.participant_id },
                },
                { label: "Identity Token", value: { type: "urn", value: participant.data.token } },
                {
                  label: "Participant Type",
                  value: { type: "role", value: participant.data.participant_type! },
                },
                { label: "Base URL", value: { type: "urn", value: participant.data.base_url } },
              ]}
            />
          </div>
        </PageSection>

        {/* Div Agreements Info */}
        <div>
          <Heading level="h5">Agreements to be done...</Heading>
          <div>
            {JSON.stringify(agreements.data)}
          </div>
        </div>
      </div>
    </PageLayout>
  );
}

const DataTypeSkeleton = () => (
  <div className="space-y-4">
    <Skeleton className="h-8 w-48" />
    <Skeleton className="h-24 w-full" />
  </div>
);
