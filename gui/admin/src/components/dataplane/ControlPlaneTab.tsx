import React from "react";
import { TabsContent } from "shared/src/components/ui/tabs";
import { InfoList } from "shared/src/components/ui/info-list";
import { FormatDate } from "shared/src/components/ui/format-date";
import { Badge } from "shared/components/ui/badge";
import { PageSection } from "shared/src/components/layout/PageSection";
import { InfoGrid } from "shared/src/components/layout/InfoGrid";
import { ProcessMessagesTable } from "shared/src/components/ProcessMessagesTable";
import { mergeStateAndAttribute } from "shared/src/lib/utils.ts";
import { TransferProcessDto } from "shared/src/data/orval/model";

export function ControlPlaneTab({ tp }: { tp: TransferProcessDto }) {
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
                value: { type: "urn", value: tp.agreementId },
              },
              {
                label: "State",
                value: {
                  type: "custom",
                  content: (
                    <Badge
                      variant="status"
                      state={mergeStateAndAttribute(tp.state ?? "", tp.stateAttribute ?? "")}
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

      <PageSection title="Exchange Messages">
        <ProcessMessagesTable
          messages={tp.messages || []}
          processId={tp.id}
          title="Transfer Messages"
        />
      </PageSection>
    </TabsContent>
  );
}

