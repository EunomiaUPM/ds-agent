import React from "react";
import { createFileRoute } from "@tanstack/react-router";
import { Tabs, TabsList, TabsTrigger } from "shared/src/components/ui/tabs";
import { TransferProcessActions } from "shared/src/components/actions/TransferProcessActions";
import { PageLayout } from "shared/src/components/layout/PageLayout";
import { PageSection } from "shared/src/components/layout/PageSection";
import { PageHeader } from "shared/components/layout/PageHeader";
import { Skeleton } from "shared/components/ui/skeleton";
import { GeneralErrorComponent } from "@/components/GeneralErrorComponent";
import { useTransferProcessDetail } from "@/components/dataplane/hooks/useTransferProcessDetail";
import { ControlPlaneTab } from "@/components/dataplane/ControlPlaneTab";
import { DataPlaneTab } from "@/components/dataplane/DataPlaneTab";
import { DataplaneLogsTab } from "@/components/dataplane/DataplaneLogsTab";
import { TransferEventsTab } from "@/components/dataplane/TransferEventsTab";

export const Route = createFileRoute("/transfer-process/$transferProcessId/")({
  component: RouteComponent,
});

function RouteComponent() {
  const { transferProcessId } = Route.useParams();
  const { isLoading, tp, dp, info, logs, events } = useTransferProcessDetail(transferProcessId);

  if (isLoading) {
    return (
      <PageLayout>
        <PageHeader title="Transfer Process" badge={<Skeleton className="h-8 w-48" />} />
        <div>Loading...</div>
      </PageLayout>
    );
  }

  if (!tp) {
    return (
      <GeneralErrorComponent error={new Error("Transfer process not found")} reset={() => {}} />
    );
  }

  return (
    <PageLayout>
      <PageSection className="w-full">
        <Tabs defaultValue="control-plane" className="w-full">
          <TabsList className="grid w-full grid-cols-4">
            <TabsTrigger value="control-plane">Control Plane</TabsTrigger>
            <TabsTrigger value="data-plane">Data Plane</TabsTrigger>
            <TabsTrigger value="dataplane-logs">Dataplane Logs</TabsTrigger>
            <TabsTrigger value="transfer-events">Transfer Events</TabsTrigger>
          </TabsList>

          <ControlPlaneTab tp={tp} />
          <DataPlaneTab dp={dp} info={info} />
          <DataplaneLogsTab logs={logs} />
          <TransferEventsTab events={events} />
        </Tabs>
      </PageSection>

      <TransferProcessActions process={tp} tiny={false} />
    </PageLayout>
  );
}
