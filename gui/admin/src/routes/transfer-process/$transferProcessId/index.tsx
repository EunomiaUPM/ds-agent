import React, { useMemo } from "react";
import { createFileRoute, Link } from "@tanstack/react-router";
import { ArrowUpRight } from "lucide-react";
import { useGetTransferMessagesByProcessId } from "shared/src/data/orval/transfers/transfers";
import { useGetAgreementByIdS } from "shared/src/data/orval/negotiations/negotiations";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@eunomia/ui";
import { TransferProcessActions } from "shared/src/components/actions/TransferProcessActions";
import { PageLayout } from "@eunomia/ui";
import { PageHeader } from "@eunomia/ui";
import { PageSection } from "@eunomia/ui";
import { FieldGrid } from "@eunomia/ui";
import { Badge } from "@eunomia/ui";
import { Skeleton } from "@eunomia/ui";
import { CountBadge } from "@eunomia/ui";
import { RelativeTime } from "@eunomia/ui";
import { ParticipantLabel } from "@eunomia/ui";
import { DatasetLabel } from "shared/src/components/identity/DatasetLabel";
import { Identifier } from "@eunomia/ui";
import { JsonTable } from "@eunomia/ui";
import { StateStepper } from "@eunomia/ui";
import { ProcessMessagesTable } from "shared/src/components/ProcessMessagesTable";
import { GeneralErrorComponent } from "@eunomia/ui";
import { useTransferProcessDetail } from "@/components/dataplane/hooks/useTransferProcessDetail";
import { DataPlaneTab } from "@/components/dataplane/DataPlaneTab";
import { DataplaneLogsTab } from "@/components/dataplane/DataplaneLogsTab";
import { TransferEventsTab } from "@/components/dataplane/TransferEventsTab";

export const Route = createFileRoute("/transfer-process/$transferProcessId/")({
  component: TransferProcessPage,
});

/** DSP 9.1: the path every transfer follows, and where it may leave it. */
const STEPS = ["REQUESTED", "STARTED", "COMPLETED"];
const DETOURS = {
  SUSPENDED: { at: "STARTED", tone: "warn" as const },
  TERMINATED: { tone: "danger" as const },
};

function TransferProcessPage() {
  const { transferProcessId } = Route.useParams();
  const { isLoading, tp, dp, info, logs, events } = useTransferProcessDetail(transferProcessId);
  const { data: messagesData } = useGetTransferMessagesByProcessId(transferProcessId, {
    limit: 100,
    sort: "created_at_asc",
  });
  const agreementId = tp?.correlation.agreementId ?? "";
  const { data: agreementData } = useGetAgreementByIdS(agreementId, {
    query: { enabled: !!agreementId },
  });

  const messages = messagesData?.status === 200 ? messagesData.data.items : [];
  const agreement = agreementData?.status === 200 ? agreementData.data : undefined;
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
        <PageHeader eyebrow="Transfer process" title={<Skeleton className="h-7 w-64" />} />
      </PageLayout>
    );
  }
  if (!tp) {
    return (
      <GeneralErrorComponent error={new Error("Transfer process not found")} reset={() => {}} />
    );
  }

  const role = tp.role === "provider" ? "Provider" : "Consumer";
  const attribute = tp.stateMetadata.attribute?.replace(/_/g, " ").toLowerCase();
  const reason = tp.stateMetadata.reason?.join("; ");

  return (
    <PageLayout>
      <PageHeader
        eyebrow="Transfer process"
        title={
          agreement ? <DatasetLabel id={agreement.target} compact /> : <Identifier value={tp.id} />
        }
        badge={
          <>
            <Badge variant="status" state={tp.state}>
              {attribute && tp.state !== "REQUESTED" ? `${tp.state} · ${attribute}` : tp.state}
            </Badge>
            <Badge variant="role" dsrole={role}>
              {role}
            </Badge>
          </>
        }
        description={
          <span className="inline-flex flex-wrap items-center gap-x-1.5">
            With <ParticipantLabel id={tp.correlation.peerParticipantId} compact /> · requested{" "}
            <RelativeTime date={tp.createdAt} /> · last change <RelativeTime date={tp.updatedAt} />
          </span>
        }
        actions={<TransferProcessActions process={tp} tiny={true} />}
      />

      <StateStepper steps={STEPS} current={tp.state} visited={visited} detours={DETOURS} />

      <Tabs defaultValue="overview" className="mt-6 w-full">
        <TabsList>
          <TabsTrigger value="overview">Overview</TabsTrigger>
          <TabsTrigger value="messages">
            Messages <CountBadge n={messages.length} />
          </TabsTrigger>
          <TabsTrigger value="dataplane">Dataplane</TabsTrigger>
          <TabsTrigger value="logs">
            Dataplane logs <CountBadge n={logs.length} />
          </TabsTrigger>
          <TabsTrigger value="events">
            Data events <CountBadge n={events.length} />
          </TabsTrigger>
        </TabsList>

        <TabsContent value="overview" className="space-y-6 pt-5">
          <FieldGrid
            columns={3}
            fields={[
              {
                label: "Counterparty",
                value: <ParticipantLabel id={tp.correlation.peerParticipantId} />,
              },
              {
                label: "Agreement",
                value: agreementId ? (
                  <Link
                    to="/agreements/$agreementId"
                    params={{ agreementId }}
                    className="inline-flex items-center gap-1 text-brand-sky hover:underline"
                  >
                    <Identifier value={agreementId} copyable={false} />
                    <ArrowUpRight className="h-3.5 w-3.5" />
                  </Link>
                ) : undefined,
              },
              { label: "Dataset", value: agreement && <DatasetLabel id={agreement.target} /> },
              { label: "Consumer PID", value: <Identifier value={tp.correlation.consumerPid} /> },
              { label: "Provider PID", value: <Identifier value={tp.correlation.providerPid} /> },
              {
                label: "Protocol",
                value: tp.protocol === "dsp2025_1" ? "DSP 2025-1" : tp.protocol,
              },
              {
                label: "Callback address",
                span: 2,
                value: tp.correlation.callbackAddress && (
                  <span className="font-mono text-xs">{tp.correlation.callbackAddress}</span>
                ),
              },
              { label: "Transfer process", value: <Identifier value={tp.id} /> },
              {
                label: "State reason",
                span: 3,
                hidden: !reason && !tp.stateMetadata.code,
                value: [tp.stateMetadata.code, reason].filter(Boolean).join(" — "),
              },
            ]}
          />
          {!!tp.errorDetails && (
            <PageSection title="Last error">
              <JsonTable value={tp.errorDetails} />
            </PageSection>
          )}
        </TabsContent>

        <TabsContent value="messages" className="pt-5">
          <ProcessMessagesTable messages={messages} role={role} />
        </TabsContent>

        <TabsContent value="dataplane" className="pt-5">
          <DataPlaneTab dp={dp} info={info} />
        </TabsContent>

        <TabsContent value="logs" className="pt-5">
          <DataplaneLogsTab logs={logs} />
        </TabsContent>

        <TabsContent value="events" className="pt-5">
          <TransferEventsTab events={events} />
        </TabsContent>
      </Tabs>
    </PageLayout>
  );
}
