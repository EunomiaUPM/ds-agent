import { createFileRoute } from "@tanstack/react-router";
import { InfoList } from "@eunomia/ui";
import { FormatDate } from "@eunomia/ui";
import { useGetTransferMessageById } from "shared/data/orval/transfers/transfers";
import { GeneralErrorComponent } from "@eunomia/ui";
import { PageLayout } from "@eunomia/ui";
import { PageHeader } from "@eunomia/ui";
import { Skeleton } from "@eunomia/ui";

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
  const { data: transferMessageResponse, isLoading: isTransferMessageLoading } =
    useGetTransferMessageById(transferMessageId);

  if (isTransferMessageLoading) {
    return (
      <PageLayout>
        <PageHeader title="Transfer Message" badge={<Skeleton className="h-8 w-48" />} />
        <div>Loading...</div>
      </PageLayout>
    );
  }

  // handle error
  if (!transferMessageResponse || transferMessageResponse.status !== 200) {
    return (
      <GeneralErrorComponent error={new Error("Transfer message not found")} reset={() => {}} />
    );
  }

  return (
    <div className="space-y-4 pb-4">
      <div>Transfer process message with id : {transferMessageResponse.data.id}</div>
      <div>
        <h2>Transfer message info: </h2>
        <InfoList
          items={[
            { label: "Transfer Message Id", value: transferMessageResponse.data.id },
            {
              label: "Transfer Process id",
              value: transferMessageResponse.data.transferProcessId,
            },
            { label: "Message type", value: transferMessageResponse.data.messageType },
            {
              label: "Created at",
              value: {
                type: "custom",
                content: <FormatDate date={transferMessageResponse.data.occurredAt} />,
              },
            },
            { label: "From", value: transferMessageResponse.data.stateTransitionFrom },
            { label: "To", value: transferMessageResponse.data.stateTransitionTo },
          ]}
        />
      </div>
      <pre className="whitespace-pre-wrap">
        {JSON.stringify(transferMessageResponse.data.envelope.payload)}
      </pre>
    </div>
  );
}
