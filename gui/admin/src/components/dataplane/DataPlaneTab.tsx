import React, { useMemo, useState } from "react";
import dayjs from "dayjs";
import {
  ArrowDownToLine,
  ArrowRight,
  ArrowUpFromLine,
  Braces,
  CheckCircle2,
  Eye,
  EyeOff,
  Lock,
  RefreshCw,
  ShieldAlert,
  XCircle,
} from "lucide-react";
import {
  Badge,
  Button,
  FieldGrid,
  Identifier,
  JsonTable,
  PageSection,
  RelativeTime,
  Skeleton,
  cn,
} from "@eunomia/ui";
import {
  DataplaneInfoResponse,
  DataplaneTransferDto,
  DataplaneTransferLogDto,
  TransferEventDto,
} from "shared/src/data/orval/model";
import {
  dataplaneStateVariant,
  extractFlowControlSummary,
  isFailureLog,
  maskSensitiveValues,
  parseAdapterConfig,
  parseReason,
  parseTransferEventData,
  roleLabel,
} from "./utils/dataplaneState";
import { CopyButton } from "./CopyButton";
import { DataplanePipeline } from "./DataplanePipeline";

interface DataPlaneTabProps {
  dp: DataplaneTransferDto | null;
  info: DataplaneInfoResponse | null;
  logs?: DataplaneTransferLogDto[];
  events?: TransferEventDto[];
  tpState?: string;
  isLoading?: boolean;
  isRefreshing?: boolean;
  onRefresh?: () => void;
}

function isLiteralSecretToken(token?: string): token is string {
  return Boolean(token && !token.includes("{{__") && token !== "[REDACTED]");
}

function shellQuote(value: string): string {
  return `'${value.replace(/'/g, "'\\''")}'`;
}

function EndpointCard({
  title,
  subtitle,
  icon,
  parsed,
  showSecrets,
}: {
  title: string;
  subtitle: string;
  icon: React.ReactNode;
  parsed: ReturnType<typeof parseAdapterConfig>;
  showSecrets: boolean;
}) {
  const extraEntries = Object.entries(parsed.extra);
  const maskedToken = parsed.token
    ? !isLiteralSecretToken(parsed.token)
      ? parsed.token
      : showSecrets
        ? parsed.token
        : "••••••••••••••••"
    : undefined;

  return (
    <div className="flex flex-col rounded-lg border border-ink/10 bg-ink/[0.015] p-4">
      <div className="mb-3 flex items-center justify-between gap-2">
        <div className="flex items-center gap-2">
          <span className="inline-flex h-7 w-7 items-center justify-center rounded-md bg-ink/[0.06] text-ink/75">
            {icon}
          </span>
          <div>
            <div className="text-xs font-semibold uppercase tracking-wide text-ink/85">{title}</div>
            <div className="text-xs text-ink/50">{subtitle}</div>
          </div>
        </div>
        <Badge variant={parsed.kind === "NoOp" ? "secondary" : "info"} size="sm">
          {parsed.kind}
        </Badge>
      </div>

      <dl className="space-y-2.5 text-xs">
        <div>
          <dt className="mb-0.5 font-medium text-ink/50">Path / target</dt>
          <dd className="min-w-0">
            {parsed.endpoint ? (
              <div className="flex items-center gap-1.5 rounded border border-ink/10 bg-background/60 px-2.5 py-1.5">
                <span className="min-w-0 flex-1 break-all font-mono text-ink/90">
                  {parsed.endpoint}
                </span>
                <CopyButton text={parsed.endpoint} />
              </div>
            ) : (
              <span className="italic text-ink/40">
                {parsed.kind === "NoOp" ? "Inactive (NoOp)" : "—"}
              </span>
            )}
          </dd>
        </div>

        {(parsed.tokenType || parsed.token) && (
          <div className="grid grid-cols-1 gap-2 sm:grid-cols-2">
            <div>
              <dt className="mb-0.5 font-medium text-ink/50">Token scheme</dt>
              <dd className="font-mono text-ink/85">{parsed.tokenType ?? "—"}</dd>
            </div>
            {parsed.token && (
              <div>
                <dt className="mb-0.5 flex items-center gap-1 font-medium text-ink/50">
                  <Lock className="h-3 w-3" />
                  Credential token
                </dt>
                <dd className="flex items-center gap-1 font-mono text-ink/85">
                  <span className="truncate" title={showSecrets ? parsed.token : "Hidden"}>
                    {maskedToken}
                  </span>
                  {showSecrets && <CopyButton text={parsed.token} />}
                </dd>
              </div>
            )}
          </div>
        )}

        {extraEntries.length > 0 && (
          <div className="border-t border-ink/10 pt-2">
            <dt className="mb-1 font-medium text-ink/50">Parameters</dt>
            <dd className="flex flex-wrap gap-1.5">
              {extraEntries.map(([k, v]) => (
                <span
                  key={k}
                  className="inline-flex max-w-full items-center gap-1 rounded border border-ink/10 bg-ink/[0.03] px-2 py-0.5 font-mono text-xs"
                >
                  <span className="text-ink/50">{k}:</span>
                  <span className="truncate text-ink/85">
                    {typeof v === "object" ? JSON.stringify(v) : String(v)}
                  </span>
                </span>
              ))}
            </dd>
          </div>
        )}
      </dl>
    </div>
  );
}

/** The dataplane side of a transfer: lifecycle progress, data path, policy, and raw payloads. */
export function DataPlaneTab({
  dp,
  info,
  logs = [],
  events = [],
  tpState,
  isLoading,
  isRefreshing,
  onRefresh,
}: DataPlaneTabProps) {
  const [showSecrets, setShowSecrets] = useState(false);
  const [showRaw, setShowRaw] = useState(false);

  const ingressParsed = useMemo(() => parseAdapterConfig(dp?.ingressConfig), [dp?.ingressConfig]);
  const egressParsed = useMemo(() => parseAdapterConfig(dp?.egressConfig), [dp?.egressConfig]);
  const flowSummary = useMemo(() => extractFlowControlSummary(dp?.flowControl), [dp?.flowControl]);

  const maskedIngress = useMemo(
    () => maskSensitiveValues(dp?.ingressConfig, showSecrets),
    [dp?.ingressConfig, showSecrets],
  );
  const maskedEgress = useMemo(
    () => maskSensitiveValues(dp?.egressConfig, showSecrets),
    [dp?.egressConfig, showSecrets],
  );
  const maskedFlowControl = useMemo(
    () => maskSensitiveValues(dp?.flowControl, showSecrets),
    [dp?.flowControl, showSecrets],
  );
  const maskedFields = useMemo(
    () => maskSensitiveValues(dp?.fields, showSecrets),
    [dp?.fields, showSecrets],
  );

  const hasAnySecrets =
    maskedIngress.hasSecrets ||
    maskedEgress.hasSecrets ||
    maskedFlowControl.hasSecrets ||
    maskedFields.hasSecrets;

  const diagnosticBanner = useMemo(() => {
    if (!dp) return null;
    const latestFailureLog = [...logs].reverse().find((l) => isFailureLog(l));
    const latestProblemEvent = events.find((e) => e.level === "Error" || e.level === "Warn");
    const lastLog = logs[logs.length - 1];
    const isTerminal = dp.state === "Terminated" || dp.state === "Error";
    const isFailed =
      dp.state === "Error" ||
      !!latestFailureLog ||
      (isTerminal && tpState?.toUpperCase() === "TERMINATED");

    if (isFailed) {
      const parsed = parseReason(latestFailureLog?.reason ?? lastLog?.reason);
      const eventDetails = latestProblemEvent
        ? parseTransferEventData(latestProblemEvent.data)
        : undefined;
      return {
        tone: "danger" as const,
        code: parsed.code,
        message:
          parsed.message ||
          latestProblemEvent?.message ||
          "The dataplane entered a terminal failure state.",
        trigger: latestFailureLog?.trigger ?? lastLog?.trigger,
        when: latestFailureLog?.created_at ?? lastLog?.created_at,
        eventHint: eventDetails?.target
          ? `${eventDetails.method ?? "REQ"}${eventDetails.status ? ` ${eventDetails.status}` : ""} -> ${eventDetails.target}`
          : undefined,
      };
    }

    if (dp.state === "Terminated") {
      const parsed = parseReason(lastLog?.reason);
      return {
        tone: "success" as const,
        code: parsed.code,
        message: parsed.message || "Dataplane session finished cleanly.",
        trigger: lastLog?.trigger,
        when: lastLog?.created_at,
        eventHint: undefined,
      };
    }

    return null;
  }, [dp, logs, events, tpState]);

  if (isLoading && !dp) {
    return (
      <div className="space-y-4">
        <Skeleton className="h-24 w-full rounded-lg" />
        <Skeleton className="h-44 w-full rounded-lg" />
      </div>
    );
  }

  if (!dp) {
    return (
      <div className="rounded-lg border border-dashed border-ink/15 px-6 py-12 text-center">
        <p className="text-sm font-medium text-ink/70">No dataplane process provisioned yet</p>
        <p className="mt-1 text-xs text-ink/50">
          The dataplane process is created once the transfer protocol moves past initial
          negotiation.
        </p>
      </div>
    );
  }

  const fieldEntries = Object.entries(dp.fields ?? {});
  const errorEventCount = events.filter((e) => e.level === "Error" || e.level === "Warn").length;
  const ingressScheme = ingressParsed.tokenType?.trim() || "Bearer";
  const curlCommand =
    info?.ingress_url && isLiteralSecretToken(ingressParsed.token)
      ? `curl -H ${shellQuote(`Authorization: ${ingressScheme} ${ingressParsed.token}`)} ${shellQuote(info.ingress_url)}`
      : undefined;

  return (
    <div className="space-y-6">
      {diagnosticBanner && (
        <div
          className={cn(
            "rounded-lg border p-4",
            diagnosticBanner.tone === "danger"
              ? "border-danger-500/30 bg-danger-500/10"
              : "border-success-500/30 bg-success-500/10",
          )}
        >
          <div className="flex items-start gap-3">
            {diagnosticBanner.tone === "danger" ? (
              <XCircle className="mt-0.5 h-4 w-4 shrink-0 text-danger-600 dark:text-danger-400" />
            ) : (
              <CheckCircle2 className="mt-0.5 h-4 w-4 shrink-0 text-success-700 dark:text-success-400" />
            )}
            <div className="min-w-0 flex-1 space-y-1.5">
              <div className="flex flex-wrap items-center gap-2">
                <span
                  className={cn(
                    "text-sm font-semibold",
                    diagnosticBanner.tone === "danger"
                      ? "text-danger-800 dark:text-danger-200"
                      : "text-success-800 dark:text-success-200",
                  )}
                >
                  {diagnosticBanner.tone === "danger"
                    ? "Dataplane failure diagnosed"
                    : "Dataplane completed"}
                </span>
                {diagnosticBanner.code && <Badge variant="code">{diagnosticBanner.code}</Badge>}
                {diagnosticBanner.trigger && (
                  <span className="rounded bg-ink/[0.07] px-1.5 py-0.5 font-mono text-xs text-ink/70">
                    {diagnosticBanner.trigger}
                  </span>
                )}
                {diagnosticBanner.when && (
                  <span className="text-xs text-ink/55">
                    <RelativeTime date={diagnosticBanner.when} />
                  </span>
                )}
              </div>
              <p className="break-words text-sm text-ink/90">{diagnosticBanner.message}</p>
              {diagnosticBanner.eventHint && (
                <p className="break-all font-mono text-xs text-ink/70">
                  Last rejected hop: {diagnosticBanner.eventHint}
                </p>
              )}
            </div>
          </div>
        </div>
      )}

      <div className="flex flex-wrap items-center justify-between gap-2">
        <div className="flex flex-wrap items-center gap-2">
          <Badge variant="status" state={dataplaneStateVariant(dp.state)}>
            {dp.state}
          </Badge>
          <Badge variant="role" dsrole={roleLabel(dp.role)}>
            {roleLabel(dp.role)}
          </Badge>
          <Badge variant="info">{dp.interactionMode}</Badge>
          {flowSummary.lifetime && (
            <Badge variant={flowSummary.lifetime === "FINITE" ? "warn" : "secondary"}>
              {flowSummary.lifetime}
            </Badge>
          )}
        </div>

        <div className="flex flex-wrap items-center gap-2">
          {hasAnySecrets && (
            <Button
              type="button"
              variant="outline"
              size="xs"
              onClick={() => setShowSecrets((v) => !v)}
            >
              {showSecrets ? <EyeOff className="h-3.5 w-3.5" /> : <Eye className="h-3.5 w-3.5" />}
              {showSecrets ? "Hide secrets" : "Reveal secrets"}
            </Button>
          )}
          <Button
            type="button"
            variant={showRaw ? "secondary" : "outline"}
            size="xs"
            onClick={() => setShowRaw((v) => !v)}
          >
            <Braces className="h-3.5 w-3.5" />
            {showRaw ? "Hide raw JSON" : "Inspect raw JSON"}
          </Button>
          {onRefresh && (
            <Button
              type="button"
              variant="outline"
              size="xs"
              onClick={onRefresh}
              disabled={isRefreshing}
            >
              <RefreshCw className={cn("h-3.5 w-3.5", isRefreshing && "animate-spin")} />
              Refresh
            </Button>
          )}
        </div>
      </div>

      <DataplanePipeline
        mode={dp.interactionMode}
        currentState={dp.state}
        logs={logs}
        tpState={tpState}
        createdAt={dp.createdAt}
        updatedAt={dp.updatedAt}
      />

      <FieldGrid
        columns={3}
        fields={[
          {
            label: "Public proxy endpoint",
            span: 3,
            value: info?.ingress_url ? (
              <div className="space-y-1.5">
                <span className="flex min-w-0 items-center gap-2">
                  <span className="min-w-0 break-all font-mono text-xs text-brand-sky">
                    {info.ingress_url}
                  </span>
                  <CopyButton text={info.ingress_url} />
                </span>
                {curlCommand &&
                  (showSecrets ? (
                    <div className="flex flex-wrap items-center gap-2 rounded border border-ink/10 bg-background/60 px-2.5 py-1.5">
                      <code className="min-w-0 flex-1 break-all font-mono text-xs text-ink/85">
                        {curlCommand}
                      </code>
                      <CopyButton text={curlCommand} label="Copy curl" />
                    </div>
                  ) : (
                    <div className="flex flex-wrap items-center gap-1.5 text-xs text-ink/55">
                      <Lock className="h-3 w-3 shrink-0" />
                      <span>
                        Requires <span className="font-mono text-ink/75">{ingressScheme}</span>{" "}
                        ingress token.
                      </span>
                      <button
                        type="button"
                        onClick={() => setShowSecrets(true)}
                        className="font-medium text-brand-sky hover:underline"
                      >
                        Reveal to copy curl
                      </button>
                    </div>
                  ))}
              </div>
            ) : (
              <span className="text-xs italic text-ink/45">
                No active HTTP proxy URL exposed for this role/mode
              </span>
            ),
          },
          { label: "Dataplane process", value: <Identifier value={dp.id} /> },
          {
            label: "Connector instance",
            value: dp.connectorInstanceId ? (
              <Identifier value={dp.connectorInstanceId} />
            ) : (
              <span className="text-xs italic text-ink/45">None</span>
            ),
          },
          {
            label: "Telemetry",
            value: (
              <span className="inline-flex flex-wrap items-center gap-2 text-xs">
                <span>{logs.length} transitions</span>
                <span className="text-ink/30">·</span>
                <span>{events.length} events</span>
                {errorEventCount > 0 && (
                  <Badge variant="warn" size="xs">
                    {errorEventCount} warn/err
                  </Badge>
                )}
              </span>
            ),
          },
          {
            label: "Created",
            value: (
              <span title={dayjs(dp.createdAt).format("YYYY-MM-DD HH:mm:ss.SSS")}>
                <RelativeTime date={dp.createdAt} />{" "}
                <span className="font-mono text-xs text-ink/45">
                  ({dayjs(dp.createdAt).format("HH:mm:ss")})
                </span>
              </span>
            ),
          },
          {
            label: "Last updated",
            span: 2,
            value: (
              <span title={dayjs(dp.updatedAt ?? dp.createdAt).format("YYYY-MM-DD HH:mm:ss.SSS")}>
                <RelativeTime date={dp.updatedAt ?? dp.createdAt} />{" "}
                <span className="font-mono text-xs text-ink/45">
                  ({dayjs(dp.updatedAt ?? dp.createdAt).format("HH:mm:ss")})
                </span>
              </span>
            ),
          },
        ]}
      />

      <PageSection title="Data path">
        <div className="grid grid-cols-1 items-stretch gap-3 lg:grid-cols-[1fr_auto_1fr]">
          <EndpointCard
            title="Ingress"
            subtitle="Where data enters this dataplane"
            icon={<ArrowDownToLine className="h-4 w-4" />}
            parsed={ingressParsed}
            showSecrets={showSecrets}
          />
          <div className="flex flex-row items-center justify-center gap-2 py-1 text-ink/45 lg:flex-col lg:px-2">
            <span className="rounded-full border border-ink/10 bg-ink/[0.03] px-2.5 py-0.5 text-[0.6875rem] font-semibold uppercase tracking-wide text-ink/60">
              {dp.interactionMode}
            </span>
            <ArrowRight className="h-4 w-4 rotate-90 lg:rotate-0" />
          </div>
          <EndpointCard
            title="Egress"
            subtitle="Where data is forwarded upstream"
            icon={<ArrowUpFromLine className="h-4 w-4" />}
            parsed={egressParsed}
            showSecrets={showSecrets}
          />
        </div>
      </PageSection>

      {(!!dp.flowControl || fieldEntries.length > 0) && (
        <PageSection title="Runtime policy & metadata">
          <FieldGrid
            columns={3}
            fields={[
              {
                label: "Delivery lifetime",
                hidden: !flowSummary.lifetime,
                value: flowSummary.lifetime && (
                  <Badge variant={flowSummary.lifetime === "FINITE" ? "warn" : "secondary"}>
                    {flowSummary.lifetime}
                  </Badge>
                ),
              },
              {
                label: "Max retries",
                hidden: flowSummary.maxRetries === undefined,
                value:
                  flowSummary.maxRetries !== undefined ? (
                    <span className="font-mono text-xs tabular-nums">{flowSummary.maxRetries}</span>
                  ) : undefined,
              },
              {
                label: "Backend auth",
                hidden: !flowSummary.authType,
                value: flowSummary.authType && (
                  <span className="inline-flex flex-wrap items-center gap-1.5">
                    <Badge variant="outline" size="sm">
                      {flowSummary.authType}
                    </Badge>
                    {flowSummary.isVaultedAuth && (
                      <Badge variant="success" size="sm">
                        <ShieldAlert className="h-3 w-3" />
                        Vaulted
                      </Badge>
                    )}
                  </span>
                ),
              },
              {
                label: "Subscription hooks",
                hidden: !flowSummary.hasSubscription && !flowSummary.hasUnsubscription,
                value: (
                  <span className="text-xs text-ink/80">
                    {[
                      flowSummary.hasSubscription ? "Subscribe" : null,
                      flowSummary.hasUnsubscription ? "Unsubscribe" : null,
                    ]
                      .filter(Boolean)
                      .join(" · ")}
                  </span>
                ),
              },
              ...fieldEntries.map(([key, val]) => ({
                label: key,
                value: <span className="break-all font-mono text-xs">{val}</span>,
              })),
            ]}
          />
        </PageSection>
      )}

      {showRaw && (
        <div className="grid grid-cols-1 gap-6 xl:grid-cols-2">
          <PageSection title="Raw ingress">
            <JsonTable value={maskedIngress.masked} emptyText="No ingress configuration." />
          </PageSection>
          <PageSection title="Raw egress">
            <JsonTable value={maskedEgress.masked} emptyText="No egress configuration." />
          </PageSection>
          {!!dp.flowControl && (
            <PageSection title="Raw flow control">
              <JsonTable value={maskedFlowControl.masked} />
            </PageSection>
          )}
          {fieldEntries.length > 0 && (
            <PageSection title="Raw fields">
              <JsonTable value={maskedFields.masked} />
            </PageSection>
          )}
        </div>
      )}
    </div>
  );
}
