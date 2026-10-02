import React from "react";
import { TabsContent } from "shared/src/components/ui/tabs";
import { InfoList } from "shared/src/components/ui/info-list";
import { FormatDate } from "shared/src/components/ui/format-date";
import { Badge } from "shared/components/ui/badge";
import { PageSection } from "shared/src/components/layout/PageSection";
import { InfoGrid } from "shared/src/components/layout/InfoGrid";
import { ProcessMessagesTable } from "shared/src/components/ProcessMessagesTable";
import { mergeStateAndAttribute } from "shared/src/lib/utils.ts";
import { ProcessMessageItem } from "shared/src/components/ProcessMessagesTable";
import { TransferMessageDto, TransferProcessDto } from "shared/src/data/orval/model";
import { useGetTransferMessagesByProcessId } from "shared/src/data/orval/transfers/transfers";

// Transfer messages keep the DSP message in their envelope; the table reads it as `payload`.
const toMessageItem = (m: TransferMessageDto): ProcessMessageItem => ({
  id: m.id,
  transferAgentProcessId: m.transferProcessId,
  processId: m.transferProcessId,
  createdAt: m.occurredAt,
  direction: m.direction,
  protocol: m.protocol,
  messageType: m.messageType,
  stateTransitionFrom: m.stateTransitionFrom,
  stateTransitionTo: m.stateTransitionTo,
  payload: m.envelope.payload,
});

export function ControlPlaneTab({ tp }: { tp: TransferProcessDto }) {
  const { data: messagesResponse } = useGetTransferMessagesByProcessId(tp.id, {
    sort: "created_at_asc",
    limit: 100,
  });
  const messages =
    messagesResponse?.status === 200 ? messagesResponse.data.items.map(toMessageItem) : [];

  return (
    <TabsContent value="control-plane" className="w-full space-y-6 mt-4">
      <PageSection title="Transfer Process Info">
        <InfoGrid>
          <InfoList
            items={[
              {
                label: "Process PID",
                value: { type: "urn", value: tp.id },
              },
              {
                label: "Agreement ID",
                value: { type: "urn", value: tp.correlation.agreementId ?? "" },
              },
              {
                label: "State",
                value: {
                  type: "custom",
                  content: (
                    <Badge
                      variant="status"
                      state={mergeStateAndAttribute(
                        tp.state ?? "",
                        tp.stateMetadata.attribute ?? "",
                      )}
                    >
                      {mergeStateAndAttribute(tp.state ?? "", tp.stateMetadata.attribute ?? "")}
                    </Badge>
                  ),
                },
              },
              {
                label: "Created At",
                value: {
                  type: "custom",
                  content: <FormatDate date={tp.createdAt} />,
                },
              },
              {
                label: "Updated At",
                value: {
                  type: "custom",
                  content: <FormatDate date={tp.updatedAt} />,
                },
              },
            ]}
          />
        </InfoGrid>
      </PageSection>

      <PageSection title="Exchange Messages">
        <ProcessMessagesTable messages={messages} processId={tp.id} title="Transfer Messages" />
      </PageSection>
    </TabsContent>
  );
}
