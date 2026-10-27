import React, { useMemo } from "react";
import { createFileRoute, Link } from "@tanstack/react-router";
import { ArrowUpRight } from "lucide-react";
import {
  useGetAgreementsByNegotiationProcess,
  useGetNegotiationMessagesByProcessId,
  useGetLastOfferByProcessId,
  useGetNegotiationProcessById,
  useGetOffersByProcessId,
} from "shared/src/data/orval/negotiations/negotiations";
import { PageLayout } from "shared/src/components/layout/PageLayout";
import { PageHeader } from "shared/src/components/layout/PageHeader";
import { PageSection } from "shared/src/components/layout/PageSection";
import { FieldGrid } from "shared/src/components/layout/FieldGrid";
import { Badge } from "shared/src/components/ui/badge";
import { Skeleton } from "shared/src/components/ui/skeleton";
import { RelativeTime } from "shared/src/components/ui/format-date";
import { SegmentedControl } from "shared/src/components/ui/segmented";
import { CountBadge } from "shared/src/components/ui/count-badge";
import { ParticipantLabel } from "shared/src/components/identity/ParticipantLabel";
import { DatasetLabel } from "shared/src/components/identity/DatasetLabel";
import { Identifier } from "shared/src/components/identity/Identifier";
import { OdrlRulesView } from "shared/src/components/policy/OdrlRulesView";
import { JsonTable } from "shared/src/components/json/JsonTable";
import { StateStepper } from "shared/src/components/process/StateStepper";
import { ProcessMessagesTable } from "shared/src/components/ProcessMessagesTable";
import { ContractNegotiationActions } from "shared/src/components/actions/ContractNegotiationActions";
import { ContractNegotiationBusinessActions } from "shared/src/components/actions/ContractNegotiationBusinessActions";
import { ACTIONS_MODE_OPTIONS, useActionsMode } from "shared/src/hooks/useActionsMode";
import { GeneralErrorComponent } from "@/components/GeneralErrorComponent";

export const Route = createFileRoute("/contract-negotiation/$cnProcess/")({
  component: NegotiationPage,
});

/** DSP 8.1: the full path of a negotiation; a provider may agree straight after the request. */
const STEPS = ["REQUESTED", "OFFERED", "ACCEPTED", "AGREED", "VERIFIED", "FINALIZED"];
const DETOURS = { TERMINATED: { tone: "danger" as const } };

function NegotiationPage() {
  const { cnProcess } = Route.useParams();
  const [mode, setMode] = useActionsMode();
  const { data: processData, isLoading } = useGetNegotiationProcessById(cnProcess);
  const { data: messagesData } = useGetNegotiationMessagesByProcessId(cnProcess, { limit: 100 });
  const { data: lastOfferData } = useGetLastOfferByProcessId(cnProcess);
  // Only the count is needed from the list.
  const { data: offersData } = useGetOffersByProcessId(cnProcess, { limit: 1 });
  const process = processData?.status === 200 ? processData.data : undefined;
  const { data: agreementsData } = useGetAgreementsByNegotiationProcess(cnProcess, undefined, {
    query: {
      enabled:
        process?.state === "FINALIZED" ||
        process?.state === "VERIFIED" ||
        process?.state === "AGREED",
    },
  });

  const messages = messagesData?.status === 200 ? messagesData.data.items : [];
  const offerCount = offersData?.status === 200 ? (offersData.data.total ?? 0) : 0;
  const lastOffer = lastOfferData?.status === 200 ? lastOfferData.data : undefined;
  const agreement = agreementsData?.status === 200 ? agreementsData.data.items?.[0] : undefined;
  const visited = useMemo(
    () =>
      messagesData
        ? new Set(messages.flatMap((m) => [m.stateTransitionFrom, m.stateTransitionTo]))
        : undefined,
    [messagesData, messages],
  );

  if (isLoading) {
    return (
      <PageLayout>
        <PageHeader eyebrow="Contract negotiation" title={<Skeleton className="h-7 w-64" />} />
      </PageLayout>
    );
  }
  if (!process) {
    return (
      <GeneralErrorComponent error={new Error("Contract negotiation not found")} reset={() => {}} />
    );
  }

  const target =
    agreement?.target ?? (lastOffer?.offerContent as { target?: string } | undefined)?.target;
  const peer = process.correlation.peerParticipantId;

  return (
    <PageLayout>
      <PageHeader
        eyebrow="Contract negotiation"
        title={target ? <DatasetLabel id={target} compact /> : <Identifier value={process.id} />}
        badge={
          <>
            <Badge variant="status" state={process.state}>
              {process.state}
            </Badge>
            <Badge variant="role" dsrole={process.role}>
              {process.role}
            </Badge>
          </>
        }
        description={
          <span className="inline-flex flex-wrap items-center gap-x-1.5">
            With <ParticipantLabel id={peer} compact /> · started{" "}
            <RelativeTime date={process.createdAt} /> · last change{" "}
            <RelativeTime date={process.updatedAt} />
          </span>
        }
        actions={
          <>
            <SegmentedControl
              value={mode}
              onChange={setMode}
              options={ACTIONS_MODE_OPTIONS}
              ariaLabel="Actions mode"
            />
            {mode === "business" ? (
              <ContractNegotiationBusinessActions process={process} tiny={true} />
            ) : (
              <ContractNegotiationActions process={process} tiny={true} />
            )}
          </>
        }
      />

      <StateStepper steps={STEPS} current={process.state} visited={visited} detours={DETOURS} />

      <PageSection title="Summary" className="mt-6">
        <FieldGrid
          columns={3}
          fields={[
            { label: "Counterparty", value: <ParticipantLabel id={peer} /> },
            { label: "Dataset", value: target && <DatasetLabel id={target} /> },
            {
              label: "Agreement",
              value: agreement ? (
                <Link
                  to="/agreements/$agreementId"
                  params={{ agreementId: agreement.id }}
                  className="inline-flex items-center gap-1 text-brand-sky hover:underline"
                >
                  <Identifier value={agreement.id} copyable={false} />
                  <ArrowUpRight className="h-3.5 w-3.5" />
                </Link>
              ) : undefined,
            },
            {
              label: "Consumer PID",
              value: <Identifier value={process.correlation.consumerPid} />,
            },
            {
              label: "Provider PID",
              value: <Identifier value={process.correlation.providerPid} />,
            },
            { label: "Negotiation", value: <Identifier value={process.id} /> },
            {
              label: "Callback address",
              span: 3,
              value: process.correlation.callbackAddress && (
                <span className="font-mono text-xs">{process.correlation.callbackAddress}</span>
              ),
            },
          ]}
        />
        {!!process.errorDetails && (
          <div className="mt-4">
            <JsonTable value={process.errorDetails} />
          </div>
        )}
      </PageSection>

      <PageSection
        title={agreement ? "Agreed policy" : "Last offer"}
        action={<CountBadge n={offerCount > 1 ? offerCount : undefined} />}
      >
        {agreement || lastOffer ? (
          <OdrlRulesView policy={agreement?.agreementContent ?? lastOffer?.offerContent} />
        ) : (
          <p className="text-sm text-ink/50">No offer has been exchanged yet.</p>
        )}
      </PageSection>

      <PageSection title="Messages" action={<CountBadge n={messages.length} />}>
        <ProcessMessagesTable messages={messages} role={process.role} />
      </PageSection>
    </PageLayout>
  );
}
