import React from "react";
import { TabsContent } from "shared/src/components/ui/tabs";
import { InfoList } from "shared/src/components/ui/info-list";
import { FormatDate } from "shared/src/components/ui/format-date";
import { Badge } from "shared/components/ui/badge";
import { PageSection } from "shared/src/components/layout/PageSection";
import { InfoGrid } from "shared/src/components/layout/InfoGrid";
import { DataplaneInfoResponse, DataplaneTransferDto } from "shared/src/data/orval/model";
import { dataplaneStateVariant, roleLabel } from "./utils/dataplaneState";
import { ConfigGrid } from "./ConfigGrid";
import { CopyButton } from "./CopyButton";

interface DataPlaneTabProps {
  dp: DataplaneTransferDto | null;
  info: DataplaneInfoResponse | null;
}

export function DataPlaneTab({ dp, info }: DataPlaneTabProps) {
  if (!dp) {
    return (
      <TabsContent value="data-plane" className="w-full">
        <div className="text-muted-foreground p-8 text-center border rounded-md border-dashed mt-4">
          No dataplane process found for this transfer.
        </div>
      </TabsContent>
    );
  }

  return (
    <TabsContent value="data-plane" className="w-full">
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-4 mt-4">
        {/* Left: process details */}
        <PageSection title="Dataplane Process">
          <InfoGrid>
            <InfoList
              items={[
                {
                  label: "ID",
                  value: { type: "urn", value: dp.id },
                },
                {
                  label: "Transfer Process ID",
                  value: { type: "urn", value: dp.transferProcessId },
                },
                {
                  label: "State",
                  value: {
                    type: "custom",
                    content: (
                      <Badge variant="status" state={dataplaneStateVariant(dp.state)}>
                        {dp.state ?? "—"}
                      </Badge>
                    ),
                  },
                },
                {
                  label: "Role",
                  value: {
                    type: "custom",
                    content: (
                      <Badge variant="role" dsrole={roleLabel(dp.role)}>
                        {dp.role ?? "—"}
                      </Badge>
                    ),
                  },
                },
                {
                  label: "Interaction Mode",
                  value: {
                    type: "custom",
                    content: dp.interactionMode ? (
                      <Badge
                        variant="info"
                        className={
                          dp.interactionMode === "PULL"
                            ? "text-sky-300 border-sky-500/40"
                            : "text-orange-300 border-orange-500/40"
                        }
                      >
                        {dp.interactionMode}
                      </Badge>
                    ) : (
                      <span className="text-muted-foreground">—</span>
                    ),
                  },
                },
                {
                  label: "Connector Instance",
                  value: {
                    type: "urn",
                    value: dp.connectorInstanceId ?? undefined,
                  },
                },
                {
                  label: "Created At",
                  value: {
                    type: "custom",
                    content: <FormatDate date={dp.createdAt} />,
                  },
                },
                {
                  label: "Updated At",
                  value: {
                    type: "custom",
                    content: <FormatDate date={dp.updatedAt} />,
                  },
                },
                ...(dp.fields
                  ? Object.entries(dp.fields).map(([key, value]) => ({
                      label: key,
                      value: {
                        type: "custom" as const,
                        content: <span className="font-mono text-xs break-all">{value}</span>,
                      },
                    }))
                  : []),
              ]}
            />
          </InfoGrid>
        </PageSection>

        {/* Right: Dataplane Info + Transfer Config */}
        <div className="space-y-4">
          <PageSection title="Dataplane Info">
            {info ? (
              <InfoGrid>
                <InfoList
                  items={[
                    {
                      label: "ID",
                      value: { type: "urn", value: info.id },
                    },
                    {
                      label: "Interaction Mode",
                      value: {
                        type: "custom",
                        content: info.interaction_mode ? (
                          <Badge variant="info">{info.interaction_mode}</Badge>
                        ) : (
                          <span className="text-muted-foreground">—</span>
                        ),
                      },
                    },
                    {
                      label: "Ingress URL",
                      value: {
                        type: "custom",
                        content: info.ingress_url ? (
                          <div className="flex items-center gap-1 min-w-0">
                            <span className="font-mono text-xs break-all text-sky-400">
                              {info.ingress_url}
                            </span>
                            <CopyButton text={info.ingress_url} />
                          </div>
                        ) : (
                          <span className="text-muted-foreground">—</span>
                        ),
                      },
                    },
                  ]}
                />
              </InfoGrid>
            ) : (
              <div className="text-muted-foreground p-4 text-center border rounded-md border-dashed">
                No dataplane info available
              </div>
            )}
          </PageSection>

          {(dp.ingressConfig || dp.egressConfig || dp.flowControl) && (
            <PageSection title="Transfer Config">
              <div className="space-y-4">
                {dp.ingressConfig && Object.keys(dp.ingressConfig).length > 0 && (
                  <div>
                    <div className="text-xs font-semibold text-muted-foreground uppercase tracking-wide mb-2">
                      Ingress Config
                    </div>
                    <div className="rounded-md border border-white/10 bg-muted/30 px-3 py-1">
                      <ConfigGrid data={dp.ingressConfig as Record<string, unknown>} />
                    </div>
                  </div>
                )}
                {dp.egressConfig && Object.keys(dp.egressConfig).length > 0 && (
                  <div>
                    <div className="text-xs font-semibold text-muted-foreground uppercase tracking-wide mb-2">
                      Egress Config
                    </div>
                    <div className="rounded-md border border-white/10 bg-muted/30 px-3 py-1">
                      <ConfigGrid data={dp.egressConfig as Record<string, unknown>} />
                    </div>
                  </div>
                )}
                {dp.flowControl && (
                  <div>
                    <div className="text-xs font-semibold text-muted-foreground uppercase tracking-wide mb-2">
                      Flow Control
                    </div>
                    <div className="rounded-md border border-white/10 bg-muted/30 px-3 py-1">
                      <ConfigGrid data={dp.flowControl as Record<string, unknown>} />
                    </div>
                  </div>
                )}
              </div>
            </PageSection>
          )}
        </div>
      </div>
    </TabsContent>
  );
}
