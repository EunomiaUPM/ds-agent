import { createFileRoute } from "@tanstack/react-router";
import { InfoList } from "shared/src/components/ui/info-list";
import { FormatDate } from "shared/src/components/ui/format-date";
import { useGetTransferMessageById } from "shared/data/orval/transfers/transfers";
import { GeneralErrorComponent } from "@/components/GeneralErrorComponent";
import { PageLayout } from "shared/components/layout/PageLayout";
import { PageHeader } from "shared/components/layout/PageHeader";
import { Skeleton } from "shared/components/ui/skeleton";

/**
 * Route for specific transfer process message details.
 */
export const Route = createFileRoute(
  "/transfer-process/$transferProcessId/transfer-message/$transferMessageId",
)({
  component: RouteComponent,
});

function RouteComponent() {
  const { transferMessageId, transferProcessId } = Route.useParams();
  const { data: transferMessageResponse, isLoading: isTransferMessageLoading } = useGetTransferMessageById(transferMessageId);


  if (isTransferMessageLoading) {
    return (
      <PageLayout>
        <PageHeader
          title="Transfer Message"
          badge={<Skeleton className="h-8 w-48" />}
        />
        <div>Loading...</div>
      </PageLayout>
    );
  }


  // handle error
  if (!transferMessageResponse || transferMessageResponse.status !== 200) {
    return <GeneralErrorComponent error={new Error("Transfer message not found")} reset={() => { }} />;
  }

  return (
    <div className="space-y-4 pb-4">
      <div>Transfer process message with id : {transferMessageResponse.data.id}</div>
      <div>
        <h2>Transfer message info: </h2>
        <InfoList
          items={[
            { label: "Transfer Message Id", value: transferMessageResponse.data.id },
            { label: "Transfer Process id", value: transferMessageResponse.data.transferAgentProcessId },
            { label: "Message type", value: transferMessageResponse.data.messageType },
            {
              label: "Created at",
              value: { type: "custom", content: <FormatDate date={transferMessageResponse.data.createdAt} /> },
            },
            { label: "From", value: transferMessageResponse.data.stateTransitionFrom },
            { label: "To", value: transferMessageResponse.data.stateTransitionTo },
          ]}
        />
      </div>
      <pre className="whitespace-pre-wrap">{JSON.stringify(transferMessageResponse.data.payload)}</pre>
    </div>
  );
}
