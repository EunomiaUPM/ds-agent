import React from "react";
import { TabsContent } from "shared/src/components/ui/tabs";
import { InfoList } from "shared/src/components/ui/info-list";
import { FormatDate } from "shared/src/components/ui/format-date";
import { Badge, BadgeState } from "shared/components/ui/badge";
import { PageSection } from "shared/src/components/layout/PageSection";
import { InfoGrid } from "shared/src/components/layout/InfoGrid";
import TransferProcessMessageComponent from "shared/src/components/TransferProcessMessageComponent";
import { mergeStateAndAttribute } from "shared/src/lib/utils.ts";
import { TransferProcessDto } from "shared/src/data/orval/model";

export function ControlPlaneTab({ tp }: { tp: TransferProcessDto }) {
  return (
    <TabsContent value="control-plane" className="w-full">
      <div className="grid grid-cols-1 md:grid-cols-2 gap-4 mt-4">
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
                  value: { type: "urn", value: tp.agreementId },
                },
                {
                  label: "State",
                  value: {
                    type: "custom",
                    content: (
                      <Badge
                        variant="status"
                        state={
                          mergeStateAndAttribute(
                            tp.state ?? "",
                            tp.stateAttribute ?? "",
                          ) as BadgeState
                        }
                      >
                        {mergeStateAndAttribute(tp.state ?? "", tp.stateAttribute ?? "")}
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

        <PageSection title="Messages">
          {tp.messages && tp.messages.length > 0 ? (
            <div className="space-y-3 max-h-[600px] overflow-y-auto pr-2">
              {tp.messages.map((message) => (
                <TransferProcessMessageComponent key={message.id} message={message} />
              ))}
            </div>
          ) : (
            <div className="text-muted-foreground p-4 text-center border rounded-md border-dashed">
              No messages available
            </div>
          )}
        </PageSection>
      </div>
    </TabsContent>
  );
}
