import React from "react";
import { createFileRoute } from "@tanstack/react-router";
import { FormatDate } from "shared/src/components/ui/format-date";
import { InfoList, InfoListItem } from "shared/src/components/ui/info-list";
import { Badge } from "shared/src/components/ui/badge";
import { PageLayout } from "shared/src/components/layout/PageLayout";
import { PageHeader } from "shared/src/components/layout/PageHeader";
import { PageSection } from "shared/src/components/layout/PageSection";
import { InfoGrid } from "shared/src/components/layout/InfoGrid";
import {
  useGetConnectorInstanceByDistribution,
  getGetConnectorInstanceByDistributionQueryOptions,
} from "shared/src/data/orval/connector/connector";
import {
  useGetDistributionById,
  getGetDistributionByIdQueryOptions,
} from "shared/src/data/orval/distributions/distributions";
import { formatUrn } from "shared/src/lib/utils.ts";
import { ConnectorInstanceDto, PushLifecycle } from "shared/src/data/orval/model";
import { CopyButton } from "@/components/dataplane/CopyButton";
import Heading from "shared/components/ui/heading";

// =============================================================================
// HELPERS
// =============================================================================

const METHOD_COLORS: Record<string, string> = {
  GET: "text-success-300 border-success-500/40",
  POST: "text-sky-300 border-sky-500/40",
  PUT: "text-warn-300 border-warn-500/40",
  PATCH: "text-warn-300 border-warn-500/40",
  DELETE: "text-danger-300 border-danger-500/40",
};

const AUTH_LABELS: Record<string, string> = {
  NO_AUTH: "No Auth",
  BASIC: "Basic Auth",
  BEARER: "Bearer Token",
  API_KEY: "API Key",
  OAUTH2: "OAuth 2.0",
  OAUTH2_CLIENT_CRED: "OAuth 2.0 Client Credentials",
};

// =============================================================================
// SUB-COMPONENTS
// =============================================================================

function MethodBadge({ method }: { method?: string | string[] }) {
  if (!method) return <>—</>;

  const methods = Array.isArray(method) ? method : [method];

  return (
    <>
      {methods.map((m) => {
        const key = String(m).toUpperCase();
        const classes = METHOD_COLORS[key] ?? "text-foreground/80 border-white/10";
        return (
          <Badge key={key} variant="info" className={`${classes} mr-2`}>
            {key}
          </Badge>
        );
      })}
    </>
  );
}

function ProtocolBadge({ protocol }: { protocol?: string | string[] }) {
  if (!protocol) return null;

  const protocols = Array.isArray(protocol) ? protocol : [protocol];

  return (
    <>
      {protocols.map((p) => (
        <Badge key={p} variant="info" className="text-primary-300 border-primary-500/40">
          {p}
        </Badge>
      ))}
    </>
  );
}

/** Displays a URL template in monospace with a copy button */
function UrlRow({ url }: { url: string }) {
  return (
    <div className="flex items-center gap-1 min-w-0">
      <span className="font-mono text-xs text-sky-400 break-all">{url}</span>
      <CopyButton text={url} />
    </div>
  );
}

/** Renders a request step (subscribe / unsubscribe / dataAccess) */
function RequestStep({ label, step }: { label: string; step: Record<string, unknown> }) {
  const protocol = step.protocol as string | undefined;
  const method = step.method as string | undefined;
  const urlTemplate = step.urlTemplate as string | undefined;
  const headers = step.headers as Record<string, unknown> | undefined;
  const bodyTemplate = step.bodyTemplate as string | undefined;
  const rest = Object.fromEntries(
    Object.entries(step).filter(
      ([k]) => !["protocol", "method", "urlTemplate", "headers", "bodyTemplate"].includes(k),
    ),
  );

  return (
    <div>
      <div className="text-xs font-semibold text-muted-foreground uppercase tracking-wide mb-2">
        {label}
      </div>
      <div className="rounded-md border border-white/10 bg-muted/20 divide-y divide-white/5">
        {/* Protocol + Method */}
        <div className="flex items-center gap-5 px-3 py-2">
          <InfoListItem
            label="protocol"
            value={
              protocol
                ? { type: "custom", content: <ProtocolBadge protocol={protocol} /> }
                : undefined
            }
          />

          <InfoListItem
            label="method/s"
            value={
              protocol ? { type: "custom", content: <MethodBadge method={method} /> } : undefined
            }
          />
        </div>

        {/* URL Template */}
        {urlTemplate && (
          <div className="flex items-center gap-5 px-3 py-2">
            <InfoListItem
              label="URL Template"
              value={
                urlTemplate
                  ? {
                      type: "custom",
                      content: <UrlRow url={urlTemplate} />,
                    }
                  : undefined
              }
            />
          </div>
        )}

        {/* Body Template */}
        {bodyTemplate && (
          <div className="px-3 py-2">
            <InfoListItem
              label="Body Template"
              value={
                urlTemplate
                  ? {
                      type: "custom",
                      content: (
                        <pre className="font-mono text-xs bg-gray-800/60 text-foreground/60 border border-secondary-600/20 whitespace-pre-wrap break-all  rounded p-2 mt-1">
                          {(() => {
                            try {
                              return JSON.stringify(JSON.parse(String(bodyTemplate)), null, 2);
                            } catch {
                              return String(bodyTemplate);
                            }
                          })()}
                        </pre>
                      ),
                    }
                  : undefined
              }
            />
          </div>
        )}

        {/* Headers */}
        {headers && typeof headers === "object" && Object.keys(headers).length > 0 && (
          <div className="px-3 py-2">
            <InfoListItem
              label="Headers"
              value={{
                type: "custom",
                content: (
                  <>
                    {Object.entries(headers as Record<string, unknown>).map(([k, v]) => (
                      <div key={k} className="flex gap-2 font-mono text-xs">
                        <span className="text-foreground/80 shrink-0 mt-0.5">{k}:</span>
                        <Badge variant="code">{String(v)}</Badge>
                      </div>
                    ))}
                  </>
                ),
              }}
            />
          </div>
        )}

        {/* Extra fields */}
        {Object.keys(rest).length > 0 && (
          <div className="px-3 py-2 space-y-0.5">
            {Object.entries(rest).map(([k, v]) => (
              <div key={k} className="flex gap-2 font-mono text-xs">
                <span className="text-muted-foreground shrink-0 min-w-[120px]">{k}</span>
                <span className="break-all">
                  {typeof v === "object" ? JSON.stringify(v) : String(v ?? "—")}
                </span>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

// =============================================================================
// ROUTE COMPONENT
// =============================================================================

function RouteComponent() {
  const { distributionId } = Route.useParams();
  const { data: distributionData } = useGetDistributionById(distributionId);
  const { data: connectorData } = useGetConnectorInstanceByDistribution(distributionId);

  const distribution = distributionData?.status === 200 ? distributionData.data : undefined;
  const connector =
    connectorData?.status === 200 ? (connectorData.data as ConnectorInstanceDto) : undefined;

  // // Only to test styles with other parameters
  // const altConnector = {
  //   authenticationConfig: {
  //     type: "BEARER",
  //     username: "example_user",

  //     tokenType: "VAULT REF",
  //     path: "/path/to/token",
  //     key: "bearer_token",
  //   },
  //   interaction: {
  //     mode: "PUSH",
  //     subscribe: {
  //       protocol: "HTTPS",
  //       method: "POST",
  //       urlTemplate: "https://example.com/subscribe",
  //       headers: {
  //         "Content-Type": "application/json",
  //       },
  //       bodyTemplate: JSON.stringify({ datasetId: "example-dataset" }),
  //     },
  //     unsubscribe: {
  //       protocol: "HTTPS",
  //       method: "POST",
  //       urlTemplate: "https://example.com/unsubscribe",
  //       headers: {
  //         "Content-Type": "application/json",
  //       },
  //       bodyTemplate: JSON.stringify({ datasetId: "example-dataset" }),
  //     },
  //   },
  // };

  const auth = connector?.authenticationConfig as { type?: string } | undefined;

  // the Record object accepts all the other attributes that aren't "type"
  // const altAuth = altConnector?.authenticationConfig as
  //   | ({ type?: string } & Record<string, any>)
  //   | undefined;
  // const altInteraction = altConnector?.interaction as
  //   | (PushLifecycle & Record<string, unknown>)
  //   | undefined;
  //  const isPushAlt = altInteraction?.mode === "PUSH";

  const interaction = connector?.interaction as
    | (PushLifecycle & Record<string, unknown>)
    | undefined;

  const isPush = interaction?.mode === "PUSH";

  return (
    <PageLayout>
      <div className=" max-w-[33dvw] ">
        <Heading level="h2">Connector for {distribution?.dctTitle || ""} Distribution</Heading>

        <div className="grid grid-cols-1 lg:grid-cols-2 gap-4 w-full">
          {/* ── LEFT COLUMN ─────────────────────────────────────────────────── */}
          <div className="space-y-4">
            {/* Distribution */}
            {/* <PageSection title="Distribution">
            <InfoGrid>
              <InfoList
                items={[
                  {
                    label: "Title",
                    value: distribution?.dctTitle ?? "—",
                  },
                  {
                    label: "Created",
                    value: {
                      type: "custom",
                      content: <FormatDate date={distribution?.dctIssued} />,
                    },
                  },
                ]}
              />
            </InfoGrid>
          </PageSection> */}

            {/* Connector metadata */}
            {connector ? (
              <PageSection title="Connector Instance">
                <InfoGrid>
                  <InfoList
                    items={[
                      // {
                      //   label: "ID",
                      //   value: { type: "urn", value: connector.id },
                      // },
                      {
                        label: "Name",
                        value: { type: "urn", value: connector.name },
                      },
                      {
                        label: "Version",
                        value: connector.version ?? "Version not specified",
                      },
                      // {
                      //   label: "Author",
                      //   value: connector.author ?? "Author not specified",
                      // },
                      // {
                      //   label: "Description",
                      //   value: connector.description ?? "Description of Connector",
                      // },
                      // {
                      //   label: "Created",
                      //   value: {
                      //     type: "custom",
                      //     content: <FormatDate date={connector.createdAt} />,
                      //   },
                      // },
                      // {
                      //   label: "Distribution ID",
                      //   value: { type: "urn", value: connector.distributionId },
                      // },
                    ]}
                  />
                </InfoGrid>
              </PageSection>
            ) : (
              <p className="italic">Connector not found</p>
            )}
          </div>
        </div>

        {/* ── RIGHT COLUMN ────────────────────────────────────────────────── */}
        <div className="space-y-4"></div>
      </div>
      <div className="h-3"></div>
      {connector && (
        <div className="bg-background-400/5 rounded-md border border-white/15 p-4 mt-6 grid grid-cols-3">
          <div className="grid-span-1 border-r border-white/10 ">
            {auth && (
              <>
                <Heading level="h6" className="text-base font-semibold mb-2">
                  Authentication
                </Heading>
                {auth.type === "NO_AUTH" && (
                  <InfoList
                    items={[
                      {
                        label: "Type",
                        value: {
                          type: "custom",
                          content: (
                            <Badge
                              variant="info"
                              key={auth.type}
                              className="text-yellow-300 border-yellow-500/40"
                            >
                              {AUTH_LABELS[auth.type ?? ""] ?? auth.type ?? "None"}
                            </Badge>
                          ),
                        },
                      },
                    ]}
                  />
                )}
              </>
            )}
          </div>
          <div className="grid-span-1  border-r border-white/10 px-4">
            {interaction && (
              <>
                <Heading level="h6" className="text-base font-semibold mb-2">
                  Interaction
                </Heading>
                <div className="flex items-center gap-2 mb-4">
                  <InfoList
                    items={[
                      {
                        label: "Mode",
                        value: {
                          type: "custom",
                          content: (
                            <Badge
                              variant="info"
                              className={
                                isPush
                                  ? "text-orange-300 border-orange-500/40"
                                  : "text-sky-300 border-sky-500/40"
                              }
                            >
                              {interaction.mode as string}
                            </Badge>
                          ),
                        },
                      },
                    ]}
                  />
                </div>

                <div className="space-y-4">
                  {isPush && (interaction as PushLifecycle).subscribe && (
                    <RequestStep
                      label="Subscribe"
                      step={(interaction as PushLifecycle).subscribe as Record<string, unknown>}
                    />
                  )}
                  {isPush && (interaction as PushLifecycle).unsubscribe && (
                    <RequestStep
                      label="Unsubscribe"
                      step={(interaction as PushLifecycle).unsubscribe as Record<string, unknown>}
                    />
                  )}
                  {!isPush && Boolean((interaction as Record<string, unknown>).dataAccess) && (
                    <RequestStep
                      label="Data Access"
                      step={
                        (interaction as Record<string, unknown>).dataAccess as Record<
                          string,
                          unknown
                        >
                      }
                    />
                  )}
                </div>
              </>
            )}
          </div>
          <div className="grid-span-1 px-4">
            <Heading level="h6" className="text-base font-semibold mb-2">
              Parameters
            </Heading>
            <InfoList
              items={[
                {
                  label: "Target Host",
                  value: {
                    type: "custom",
                    content: <p>api.example.com</p>,
                  },
                },
                {
                  label: "Target Port",
                  value: {
                    type: "custom",
                    content: <p>8080</p>,
                  },
                },
              ]}
            />
          </div>
        </div>
      )}
      {/* {altConnector && (
        <div className="bg-background-400/5 rounded-md border border-white/15  mt-6 grid grid-cols-3">
          <div className="grid-span-1 border-r border-white/10 p-4">
            {altAuth && (
              <>
                <Heading level="h6" className="text-base font-semibold">
                  Authentication
                </Heading>
                {Object.entries(altAuth)
                  .filter(([k]) => k !== "type")
                  .map(([k, v]) => (
                    <div key={k} className="flex gap-2 font-mono text-xs mt-1p x-4">
                      <span className="text-muted-foreground shrink-0 min-w-[120px]">{k}</span>
                      <span className="break-all">{String(v ?? "None")}</span>
                    </div>
                  ))}
              </>
            )}
            {altAuth?.type === "BEARER" && (
              <>
                <InfoList
                  items={[
                    {
                      label: "Type",
                      value: {
                        type: "custom",
                        content: (
                          <Badge
                            variant="info"
                            key={altAuth.type}
                            className="text-yellow-300 border-yellow-500/40"
                          >
                            {AUTH_LABELS[altAuth.type ?? ""] ?? altAuth.type ?? "None"}
                          </Badge>
                        ),
                      },
                    },
                    {
                      label: "Token Type",
                      value: {
                        type: "custom",
                        content: <p>{altAuth.tokenType}</p>,
                      },
                    },
                    {
                      label: "Username",
                      value: {
                        type: "custom",
                        content: <p>{altAuth.username}</p>,
                      },
                    },
                    {
                      label: "Path",
                      value: {
                        type: "custom",
                        content: <p>{altAuth.path}</p>,
                      },
                    },
                    {
                      label: "Key",
                      value: {
                        type: "custom",
                        content: <p>{altAuth.key}</p>,
                      },
                    },
                  ]}
                />
              </>
            )}
          </div>
          <div className="grid-span-1  border-r border-white/10 p-4">
            {altInteraction && (
              <>
                <Heading level="h6" className="text-base font-semibold mb-2">
                  Interaction
                </Heading>
                <div className="flex items-center gap-2 mb-4">
                  <InfoList
                    items={[
                      {
                        label: "Mode",
                        value: {
                          type: "custom",
                          content: (
                            <Badge
                              variant="info"
                              className={
                                isPush
                                  ? "text-orange-300 border-orange-500/40"
                                  : "text-sky-300 border-sky-500/40"
                              }
                            >
                              {altInteraction.mode as string}
                            </Badge>
                          ),
                        },
                      },
                    ]}
                  />
                </div>
                <div className="space-y-4">
                  {isPushAlt && (altInteraction as PushLifecycle).subscribe && (
                    <RequestStep
                      label="Subscribe"
                      step={(altInteraction as PushLifecycle).subscribe as Record<string, unknown>}
                    />
                  )}
                  {isPushAlt && (altInteraction as PushLifecycle).unsubscribe && (
                    <RequestStep
                      label="Unsubscribe"
                      step={
                        (altInteraction as PushLifecycle).unsubscribe as Record<string, unknown>
                      }
                    />
                  )}
                  {!isPushAlt &&
                    Boolean((altInteraction as Record<string, unknown>).dataAccess) && (
                      <RequestStep
                        label="Data Access"
                        step={
                          (altInteraction as Record<string, unknown>).dataAccess as Record<
                            string,
                            unknown
                          >
                        }
                      />
                    )}
                </div>
              </>
            )}
          </div>
          <div className="grid-span-1 p-4">
            <Heading level="h6" className="text-base font-semibold mb-2">
              Parameters
            </Heading>
            <InfoList
              items={[
                {
                  label: "Target Host",
                  value: {
                    type: "custom",
                    content: <p>api.example.com</p>,
                  },
                },
                {
                  label: "Target Port",
                  value: {
                    type: "custom",
                    content: <p>8080</p>,
                  },
                },
              ]}
            />
          </div>
        </div>
      )} */}
    </PageLayout>
  );
}

/**
 * Route for displaying distribution connector details.
 */
export const Route = createFileRoute("/catalog/$catalogId/distribution-connector/$distributionId")({
  component: RouteComponent,
  pendingComponent: () => <div>Loading...</div>,
  loader: async ({ context: { queryClient }, params: { distributionId } }) => {
    await queryClient.ensureQueryData(getGetDistributionByIdQueryOptions(distributionId));
    return queryClient.ensureQueryData(
      getGetConnectorInstanceByDistributionQueryOptions(distributionId),
    );
  },
});
