/**
 * participants/index.tsx
 *
 * Participants listing page with different layouts based on participant type:
 * - Agent + isMe=true: InfoList (my agent info)
 * - Agent + isMe=false: DataTable (other agents)
 * - Authority: InfoList (authority info)
 *
 * @example
 * Used as the index route for /participants/
 */

import { createFileRoute, Link } from "@tanstack/react-router";
import { formatUrn } from "shared/src/lib/utils";
import { DataTable } from "shared/src/components/DataTable";
import { useContext, useMemo } from "react";
import { Button } from "shared/src/components/ui/button.tsx";
import { Badge, BadgeRole } from "shared/src/components/ui/badge";
import Heading from "shared/src/components/ui/heading";
import { InfoList } from "shared/src/components/ui/info-list";

// Icons
import { ArrowRight } from "lucide-react";
import { GlobalInfoContext, GlobalInfoContextType } from "shared/src/context/GlobalInfoContext.tsx";
import { PageLayout } from "shared/src/components/layout/PageLayout";
import { PageHeader } from "shared/src/components/layout/PageHeader";
import { PageSection } from "shared/src/components/layout/PageSection";
import { useGetAllParticipants } from "shared/data/orval/participants/participants";
import { GeneralErrorComponent } from "@/components/GeneralErrorComponent";
import { ParticipantDto } from "shared/data/orval/model/participantDto";

// =============================================================================
// ROUTE
// =============================================================================

/**
 * Route for listing participants with type-based layouts.
 */
export const Route = createFileRoute("/participants/")({
  component: RouteComponent,
});

// =============================================================================
// COMPONENT
// =============================================================================

import { Skeleton } from "shared/src/components/ui/skeleton";

function RouteComponent() {
  const { data: participants, isLoading, isError, error } = useGetAllParticipants();
  const { api_gateway } = useContext<GlobalInfoContextType | null>(GlobalInfoContext)!;

  if (isLoading) {
    return (
      <PageLayout>
        <PageHeader title="Participants" />
        <div className="space-y-6">
          Loading...
        </div>
      </PageLayout>
    );
  }

  if (isError || !participants || participants.status !== 200) {
    const finalError = error instanceof Error ? error : new Error("Participants not found");
    return <GeneralErrorComponent error={finalError} reset={() => { }} />;
  }

  // ---------------------------------------------------------------------------
  // Computed: Categorize participants
  // ---------------------------------------------------------------------------
  /** My agent (Agent + isMe=true) */
  const myAgent = useMemo(() => {
    return participants.data.find((p) => p.participant_type === "Agent" && p.is_me === true);
  }, [participants]);

  /** Other agents (Agent + isMe=false) */
  const otherAgents = useMemo(() => {
    return participants.data.filter((p) => p.participant_type === "Agent" && p.is_me !== true);
  }, [participants]);

  /** Authorities */
  const authorities = useMemo(() => {
    return participants.data.filter((p) => p.participant_type === "Authority");
  }, [participants]);

  // ---------------------------------------------------------------------------
  // Handlers
  // ---------------------------------------------------------------------------

  const handleOnboard = async () => {
    // await onboardAsync({ api_gateway });
  };

  // ---------------------------------------------------------------------------
  // Render: InfoList for a single participant
  // ---------------------------------------------------------------------------

  const renderParticipantInfoList = (participant: ParticipantDto, title: string) => (
    <PageSection title={title}>
      <div className="max-w-screen-md">
        <InfoList
          items={[
            { label: "Participant ID", value: { type: "urn", value: participant.participant_id } },
            { label: "Identity Token", value: { type: "urn", value: participant.token } },
            {
              label: "Participant Type",
              value: { type: "role", value: participant.participant_type! },
            },
            { label: "Base URL", value: { type: "urn", value: participant.base_url } },
          ]}
        />
      </div>
    </PageSection>
  );

  // ---------------------------------------------------------------------------
  // Render: Table for agents
  // ---------------------------------------------------------------------------

  const renderAgentsTable = (agents: ParticipantDto[]) => (
    <PageSection title="Other Agents">
      <DataTable
        className="text-sm"
        data={agents}
        keyExtractor={(p) => p.participant_id!}
        columns={[
          {
            header: "Participant ID",
            cell: (p) => <Badge variant={"info"}>{formatUrn(p.participant_id)}</Badge>,
          },
          {
            header: "Identity Token",
            cell: (p) => <Badge variant={"info"}>{formatUrn(p.token)}</Badge>,
          },
          {
            header: "Participant Type",
            cell: (p) => (
              <Badge variant={"role"} dsrole={p.participant_type as BadgeRole}>
                {p.participant_type}
              </Badge>
            ),
          },
          {
            header: "Base URL",
            cell: (p) => <Badge variant={"info"}>{p.base_url}</Badge>,
          },
          {
            header: "Link",
            cell: (p) => (
              <Link to="/participants/$participantId" params={{ participantId: p.participant_id }}>
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
  );

  // ---------------------------------------------------------------------------
  // Render
  // ---------------------------------------------------------------------------

  return (
    <PageLayout>
      {/* NO WALLET - Onboarding prompt */}
      {!myAgent && (
        <div className="p-8 py-6 mx-auto w-fit max-w-[70ch] bg-brand-sky/5 border border-stroke rounded-md">
          <Heading level="h3">Missing wallet...</Heading>
          <Heading level="h5">
            Your wallet is not yet connected as Provider. <br /> Please complete the onboarding
            process to get started.
          </Heading>
          <Button size={"lg"} className="w-full mt-4" onClick={handleOnboard}>
            Onboard wallet
          </Button>
        </div>
      )}

      {/* WALLET OK */}
      {myAgent && (
        <>
          <PageHeader title="Participants" />

          {/* My Agent (Agent + isMe=true) → InfoList */}
          {renderParticipantInfoList(myAgent, "My Agent")}

          {/* Other Agents (Agent + isMe=false) → Table */}
          {otherAgents.length > 0 && renderAgentsTable(otherAgents)}

          {/* Authority → InfoList for each */}
          {authorities.map((authority) => (
            <div key={authority.participant_id}>
              {renderParticipantInfoList(authority, `Authority`)}
            </div>
          ))}
        </>
      )}
    </PageLayout>
  );
}
