import React from "react";
import { createFileRoute } from "@tanstack/react-router";
import { FormatDate } from "shared/src/components/ui/format-date";
import { InfoList } from "shared/src/components/ui/info-list";
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

// =============================================================================
// HELPERS
// =============================================================================

const METHOD_COLORS: Record<string, string> = {
  GET:    "text-success-300 border-success-500/40",
  POST:   "text-sky-300 border-sky-500/40",
  PUT:    "text-warn-300 border-warn-500/40",
  PATCH:  "text-warn-300 border-warn-500/40",
  DELETE: "text-danger-300 border-danger-500/40",
};

const AUTH_LABELS: Record<string, string> = {
  NO_AUTH:            "No Auth",
  BASIC:              "Basic Auth",
  BEARER:             "Bearer Token",
  API_KEY:            "API Key",
  OAUTH2:             "OAuth 2.0",
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
        {methods.map((m) => (
            <span key={m}>{m.toUpperCase()}</span>
        ))}
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
        <div className="flex items-center gap-2 px-3 py-2">
          {protocol && <ProtocolBadge protocol={protocol} />}
          {method && <MethodBadge method={method} />}
        </div>

        {/* URL Template */}
        {urlTemplate && (
          <div className="px-3 py-2">
            <div className="text-[10px] text-muted-foreground uppercase tracking-wide mb-1">
              URL Template
            </div>
            <UrlRow url={urlTemplate} />
          </div>
        )}

        {/* Body Template */}
        {bodyTemplate && (
          <div className="px-3 py-2">
            <div className="text-[10px] text-muted-foreground uppercase tracking-wide mb-1">
              Body Template
            </div>
            <pre className="font-mono text-xs text-foreground/80 whitespace-pre-wrap break-all bg-muted/40 rounded p-2 mt-1">
              {(() => {
                try {
                  return JSON.stringify(JSON.parse(String(bodyTemplate)), null, 2);
                } catch {
                  return String(bodyTemplate);
                }
              })()}
            </pre>
          </div>
        )}

        {/* Headers */}
        {headers && typeof headers === "object" && Object.keys(headers).length > 0 && (
          <div className="px-3 py-2">
            <div className="text-[10px] text-muted-foreground uppercase tracking-wide mb-1">
              Headers
            </div>
            {Object.entries(headers as Record<string, unknown>).map(([k, v]) => (
              <div key={k} className="flex gap-2 font-mono text-xs">
                <span className="text-muted-foreground shrink-0">{k}:</span>
                <span className="break-all">{String(v)}</span>
              </div>
            ))}
          </div>
        )}

        {/* Extra fields */}
        {Object.keys(rest).length > 0 && (
          <div className="px-3 py-2 space-y-0.5">
            {Object.entries(rest).map(([k, v]) => (
              <div key={k} className="flex gap-2 font-mono text-xs">
                <span className="text-muted-foreground shrink-0 min-w-[120px]">{k}</span>
                <span className="break-all">{typeof v === "object" ? JSON.stringify(v) : String(v ?? "—")}</span>
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
  const connector = connectorData?.status === 200 ? (connectorData.data as ConnectorInstanceDto) : undefined;

  const auth = connector?.authenticationConfig as { type?: string } | undefined;
  const interaction = connector?.interaction as (PushLifecycle & Record<string, unknown>) | undefined;
  const isPush = interaction?.mode === "PUSH";

  return (
    <PageLayout>
      <PageHeader
        title="Distribution"
        badge={
          <Badge variant="info" size="lg">
            {formatUrn(distributionId)}
          </Badge>
        }
      />

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-4 w-full">
        {/* ── LEFT COLUMN ─────────────────────────────────────────────────── */}
        <div className="space-y-4">
          {/* Distribution */}
          <PageSection title="Distribution">
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
          </PageSection>

          {/* Connector metadata */}
          {connector && (
            <PageSection title="Connector Instance">
              <InfoGrid>
                <InfoList
                  items={[
                    {
                      label: "ID",
                      value: { type: "urn", value: connector.id },
                    },
                    {
                      label: "Name",
                      value: { type: "urn", value: connector.name },
                    },
                    {
                      label: "Version",
                      value: connector.version ?? "—",
                    },
                    {
                      label: "Author",
                      value: connector.author ?? "—",
                    },
                    {
                      label: "Description",
                      value: connector.description ?? "—",
                    },
                    {
                      label: "Created",
                      value: {
                        type: "custom",
                        content: <FormatDate date={connector.createdAt} />,
                      },
                    },
                    {
                      label: "Distribution ID",
                      value: { type: "urn", value: connector.distributionId },
                    },
                  ]}
                />
              </InfoGrid>
            </PageSection>
          )}
        </div>

        {/* ── RIGHT COLUMN ────────────────────────────────────────────────── */}
        <div className="space-y-4">
          {/* Authentication */}
          {auth && (
            <PageSection title="Authentication">
              <div className="flex items-center gap-2 py-1">
                <span className="text-xs text-muted-foreground">Type</span>
                <Badge variant="info">
                  {AUTH_LABELS[auth.type ?? ""] ?? auth.type ?? "—"}
                </Badge>
              </div>

              {/* Extra auth fields (e.g. username for basic, token hint, etc.) */}
              {Object.entries(auth)
                .filter(([k]) => k !== "type")
                .map(([k, v]) => (
                  <div key={k} className="flex gap-2 font-mono text-xs mt-1">
                    <span className="text-muted-foreground shrink-0 min-w-[120px]">{k}</span>
                    <span className="break-all">{String(v ?? "—")}</span>
                  </div>
                ))}
            </PageSection>
          )}

          {/* Interaction */}
          {interaction && (
            <PageSection title="Interaction">
              <div className="flex items-center gap-2 mb-4">
                <span className="text-xs text-muted-foreground">Mode</span>
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
                    step={(interaction as Record<string, unknown>).dataAccess as Record<string, unknown>}
                  />
                )}
              </div>
            </PageSection>
          )}
        </div>
      </div>
    </PageLayout>
  );
}

/**
 * Route for displaying distribution connector details.
 */
export const Route = createFileRoute(
  "/catalog/$catalogId/distribution-connector/$distributionId",
)({
  component: RouteComponent,
  pendingComponent: () => <div>Loading...</div>,
  loader: async ({ context: { queryClient }, params: { distributionId } }) => {
    await queryClient.ensureQueryData(
      getGetDistributionByIdQueryOptions(distributionId),
    );
    return queryClient.ensureQueryData(
      getGetConnectorInstanceByDistributionQueryOptions(distributionId),
    );
  },
});
