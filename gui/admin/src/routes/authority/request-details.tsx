import {
  AlertCircle,
  Check,
  CheckCircle2,
  Clock,
  Copy,
  Eye,
  EyeOff,
  ExternalLink,
  FileJson,
  Info,
  Key,
  Loader2,
  Shield,
} from "lucide-react";
import { useState } from "react";
import QRCode from "react-qr-code";
import { PageHeader } from "shared/src/components/layout/PageHeader";
import { PageLayout } from "shared/src/components/layout/PageLayout";
import { PageSection } from "shared/src/components/layout/PageSection";
import { Badge } from "shared/src/components/ui/badge";
import { Button } from "shared/src/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "shared/src/components/ui/card";
import { FormatDate } from "shared/src/components/ui/format-date";
import { customInstance } from "shared/src/data/orval-mutator";
import { formatUrn, getFriendlyVCType } from "shared/src/lib/utils";
import { z } from "zod";

import { useQuery, useQueryClient } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";

const searchSchema = z.object({
  requestId: z.string().optional(),
});

interface SentGrant {
  id: string;
  participant_id: string;
  participant_nick: string;
  grant_endpoint: string;
  kind: string;
  status: string;
  token?: string | null;
  /**
   * Each entry is the string id of a VcTypeConfig, e.g. "gx_VatId_jwt_vc_json".
   * Comes from the backend's `impl_serde_via_str!(VcTypeConfig)`.
   */
  vc_type_config?: string[] | null;
  vc_uri?: string | null;
  as_assigned_id?: string | null;
  auto: boolean;
  created_at: string;
  ended_at?: string | null;
}

interface SentInteraction {
  id: string;
  start: any[];
  method: string;
  callback_uri: string;
  client_nonce: string;
  hash_method: any;
  hints?: string | null;
  continue_endpoint?: string | null;
  continue_token?: string | null;
  continue_wait?: number | null;
  as_nonce?: string | null;
  oidc_vp_uri?: string | null;
  interact_ref?: string | null;
  hash?: string | null;
}

interface SentVerification {
  id: string;
  uri: string;
  scheme: string;
  response_type: string;
  client_id: string;
  response_mode: string;
  pd_uri: string;
  client_id_scheme: string;
  nonce: string;
  response_uri: string;
  status: string;
  created_at: string;
  ended_at?: string | null;
}

interface DetailsResponse {
  status: number;
  data: {
    grant: SentGrant;
    interaction: SentInteraction | null;
    verification: SentVerification | null;
  };
}


/**
 * Route for viewing details of a specific VC request.
 * Path: /authority/request-details
 */
// @ts-ignore
export const Route = createFileRoute("/authority/request-details")({
  validateSearch: (search) => searchSchema.parse(search),
  component: RequestDetailsPage,
});

function RequestDetailsPage() {
  const { requestId } = Route.useSearch();
  const queryClient = useQueryClient();
  const [isProcessing, setIsProcessing] = useState(false);

  const queryKey = ["vc-request-details", requestId];
  const { data: response, isLoading } = useQuery({
    queryKey,
    queryFn: () =>
      customInstance<DetailsResponse>(`/vc-request/${encodeURIComponent(requestId!)}/details`, {
        method: "GET",
      }),
    enabled: !!requestId,
  });

  const details = response?.status === 200 ? response.data : null;
  const grant = details?.grant ?? null;
  const interaction = details?.interaction ?? null;
  const verification = details?.verification ?? null;

  const timelineData = grant ? getTimelineData(grant) : null;

  const handleAction = async (path: string, uri: string) => {
    if (!grant) return;
    setIsProcessing(true);
    try {
      await customInstance(`${path}/${encodeURIComponent(grant.id)}`, {
        method: "POST",
        data: { uri },
      });
      await queryClient.invalidateQueries({ queryKey });
    } catch (err) {
      console.error(err);
    } finally {
      setIsProcessing(false);
    }
  };

  if (isLoading) {
    return (
      <PageLayout>
        <PageHeader title="Loading request..." />
        <PageSection>
          <div className="flex justify-center py-20">
            <Badge variant="infoLighter" className="animate-pulse">
              Loading details...
            </Badge>
          </div>
        </PageSection>
      </PageLayout>
    );
  }

  if (!grant) {
    return (
      <PageLayout>
        <PageHeader title="Request Not Found" />
        <PageSection>
          <div className="flex flex-col items-center justify-center py-20 text-muted-foreground">
            <AlertCircle className="h-12 w-12 mb-4 opacity-20" />
            <p>We couldn't find a VC request with the ID: {requestId}</p>
          </div>
        </PageSection>
      </PageLayout>
    );
  }

  return (
    <PageLayout>
      <PageHeader title={`Request: ${grant.participant_nick || "Authority"}`}>
        <div className="text-xs text-muted-foreground font-mono mt-1">ID: {grant.id}</div>
      </PageHeader>

      <PageSection>
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
          {/* ===== Grant ============================================================= */}
          <Card className="md:col-span-2">
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Info className="h-5 w-5 text-primary" />
                Grant
              </CardTitle>
              <CardDescription>The credential request sent to the authority.</CardDescription>
            </CardHeader>
            <CardContent className="space-y-6">
              <div className="grid grid-cols-1 sm:grid-cols-2 gap-y-6 gap-x-4">
                <DetailItem label="Status">
                  <Badge variant={"status"} state={grant.status}>
                    {grant.status || "-"}
                  </Badge>
                </DetailItem>
                <DetailItem label="Kind">
                  <Badge variant="info" className="font-mono">
                    {grant.kind}
                  </Badge>
                </DetailItem>
                <DetailItem label="Credential Types">
                  {(grant.vc_type_config ?? []).length === 0 ? (
                    <span className="text-xs text-muted-foreground">—</span>
                  ) : (
                    <div className="flex flex-wrap gap-1">
                      {(grant.vc_type_config ?? []).map((cfg, idx) => (
                        <Badge key={idx} variant="role">
                          {getFriendlyVCType(cfg)}
                        </Badge>
                      ))}
                    </div>
                  )}
                </DetailItem>
                <DetailItem label="Auto-claim">
                  <Badge variant={grant.auto ? "default" : "info"} className="text-[10px]">
                    {grant.auto ? "ON" : "OFF"}
                  </Badge>
                </DetailItem>
                <DetailItem label="Authority DID">
                  <span className="font-mono text-xs break-all">
                    {formatUrn(grant.participant_id || "")}
                  </span>
                </DetailItem>
                <DetailItem label="Authority Nick">{grant.participant_nick || "-"}</DetailItem>
                <DetailItem label="Grant Endpoint">
                  <div className="flex items-center gap-1 text-xs text-muted-foreground break-all">
                    {grant.grant_endpoint || "-"}
                    {grant.grant_endpoint && <ExternalLink className="h-3 w-3" />}
                  </div>
                </DetailItem>
                <DetailItem label="AS Assigned ID">
                  <span className="font-mono text-xs">{grant.as_assigned_id || "-"}</span>
                </DetailItem>
                <DetailItem label="Created At">
                  {grant.created_at ? <FormatDate date={grant.created_at} /> : "-"}
                </DetailItem>
                <DetailItem label="Ended At">
                  {grant.ended_at ? <FormatDate date={grant.ended_at} /> : "-"}
                </DetailItem>
              </div>

              {grant.vc_uri && grant.status?.toLowerCase() === "approved" && (
                <div className="pt-4 border-t space-y-6">
                  <DetailItem label="VC URI (Claiming)" labelClassName="text-green-500">
                    <div className="mt-2 flex flex-col sm:flex-row gap-6 items-start">
                      <div className="p-3 bg-white rounded-lg shadow-sm border border-stroke flex-shrink-0">
                        <QRCode value={grant.vc_uri} size={120} />
                      </div>
                      <div className="flex-1 w-full space-y-3">
                        <p className="text-xs text-muted-foreground italic">
                          Scan this QR to claim your Verifiable Credential directly in your wallet.
                        </p>
                        <UriDisplay uri={grant.vc_uri} />
                        <Button
                          className="w-full sm:w-auto bg-green-600 hover:bg-green-700 text-white"
                          size="sm"
                          onClick={() => handleAction("/vc-request/oid4vci", grant.vc_uri!)}
                          disabled={isProcessing}
                        >
                          {isProcessing ? (
                            <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                          ) : (
                            <Shield className="mr-2 h-4 w-4" />
                          )}
                          Claim in Agent
                        </Button>
                      </div>
                    </div>
                  </DetailItem>
                </div>
              )}
            </CardContent>
          </Card>

          {/* ===== Timeline ============================================================ */}
          <Card>
            <CardHeader>
              <CardTitle>Request Timeline</CardTitle>
            </CardHeader>
            <CardContent className="space-y-6">
              {timelineData && timelineData.pastEvents.length > 0 && (
                <div className="relative pl-8 border-l-[3px] border-primary/10 space-y-8 ml-2">
                  {timelineData.pastEvents.map((event) => (
                    <div key={event.id} className="relative">
                      <span className="absolute -left-[43.5px] top-1 h-5 w-5 rounded-full bg-primary border-[5px] border-background shadow-sm" />
                      <div className="space-y-1">
                        <p className="text-sm font-medium text-muted-foreground">{event.title}</p>
                        {event.date && (
                          <p className="text-[10px] text-muted-foreground/60 flex items-center gap-1 font-mono">
                            <Clock className="h-3 w-3" />
                            <FormatDate date={event.date} />
                          </p>
                        )}
                      </div>
                    </div>
                  ))}
                </div>
              )}

              {timelineData && (
                <div className="pt-4 border-t border-stroke space-y-4">
                  <div className="flex justify-between items-center text-sm">
                    <span className="text-muted-foreground font-semibold uppercase tracking-wider text-xs">
                      Current State:
                    </span>
                    <Badge variant={"status"} state={grant.status}>
                      {grant.status}
                    </Badge>
                  </div>
                  <div className="p-4 rounded-lg bg-muted/30 border border-stroke text-sm text-foreground/90 leading-relaxed">
                    {timelineData.instruction}
                  </div>
                </div>
              )}
            </CardContent>
          </Card>
        </div>

        {/* ===== Interaction ========================================================= */}
        {interaction ? (
          <Card className="mt-6">
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Key className="h-5 w-5 text-primary" />
                Interaction
              </CardTitle>
              <CardDescription>GNAP interaction handshake associated to this grant.</CardDescription>
            </CardHeader>
            <CardContent>
              <div className="grid grid-cols-1 sm:grid-cols-2 gap-y-6 gap-x-4">
                <DetailItem label="Method">
                  <Badge variant="info" className="font-mono">
                    {String(interaction.method)}
                  </Badge>
                </DetailItem>
                <DetailItem label="Start">
                  <div className="flex flex-wrap gap-1">
                    {(interaction.start ?? []).map((s, idx) => (
                      <Badge key={idx} variant="role">
                        {typeof s === "string" ? s : Object.keys(s ?? {})[0] ?? "?"}
                      </Badge>
                    ))}
                  </div>
                </DetailItem>
                <DetailItem label="Callback URI">
                  <span className="font-mono text-[10px] break-all">{interaction.callback_uri}</span>
                </DetailItem>
                <DetailItem label="Continue Endpoint">
                  <span className="font-mono text-[10px] break-all">
                    {interaction.continue_endpoint || "—"}
                  </span>
                </DetailItem>
                <DetailItem label="Hash Method">
                  <span className="font-mono text-[10px]">
                    {typeof interaction.hash_method === "string"
                      ? interaction.hash_method
                      : Object.keys(interaction.hash_method ?? {})[0] ?? "—"}
                  </span>
                </DetailItem>
                <DetailItem label="Continue Wait">
                  <span className="font-mono text-[10px]">{interaction.continue_wait ?? "—"}</span>
                </DetailItem>
                <DetailItem label="Interact Ref">
                  <SecretField value={interaction.interact_ref} />
                </DetailItem>
                <DetailItem label="OIDC4VP URI">
                  <span className="font-mono text-[10px] break-all">
                    {interaction.oidc_vp_uri || "—"}
                  </span>
                </DetailItem>
              </div>
            </CardContent>
          </Card>
        ) : (
          <Card className="mt-6 opacity-50 border-dashed">
            <CardHeader>
              <CardTitle className="flex items-center gap-2 text-muted-foreground">
                <Key className="h-5 w-5" />
                Interaction
              </CardTitle>
              <CardDescription>
                Not required — the authority approved this grant directly without a GNAP handshake.
              </CardDescription>
            </CardHeader>
          </Card>
        )}

        {/* ===== Verification ======================================================== */}
        {verification ? (
          <Card className="mt-6">
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <CheckCircle2 className="h-5 w-5 text-amber-500" />
                Verification (OIDC4VP)
              </CardTitle>
              <CardDescription>
                The authority requested a presentation before issuing the credential.
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-6">
              <div className="grid grid-cols-1 sm:grid-cols-2 gap-y-6 gap-x-4">
                <DetailItem label="Status">
                  <Badge variant={"status"} state={verification.status}>
                    {verification.status}
                  </Badge>
                </DetailItem>
                <DetailItem label="Response Type">
                  <span className="font-mono text-[10px]">{verification.response_type}</span>
                </DetailItem>
                <DetailItem label="Client ID">
                  <span className="font-mono text-[10px] break-all">{verification.client_id}</span>
                </DetailItem>
                <DetailItem label="Client ID Scheme">
                  <span className="font-mono text-[10px]">{verification.client_id_scheme}</span>
                </DetailItem>
                <DetailItem label="Response Mode">
                  <span className="font-mono text-[10px]">{verification.response_mode}</span>
                </DetailItem>
                <DetailItem label="Response URI">
                  <span className="font-mono text-[10px] break-all">
                    {verification.response_uri}
                  </span>
                </DetailItem>
                <DetailItem label="PD URI">
                  <span className="font-mono text-[10px] break-all">{verification.pd_uri}</span>
                </DetailItem>
                <DetailItem label="Nonce">
                  <SecretField value={verification.nonce} />
                </DetailItem>
                <DetailItem label="Created At">
                  <FormatDate date={verification.created_at} />
                </DetailItem>
                <DetailItem label="Ended At">
                  {verification.ended_at ? <FormatDate date={verification.ended_at} /> : "—"}
                </DetailItem>
              </div>

              {verification.uri &&
                verification.status?.toLowerCase() !== "verified" &&
                verification.status?.toLowerCase() !== "failed" && (
                <div className="pt-4 border-t">
                  <DetailItem
                    label="Verification URI (Authentication)"
                    labelClassName="text-amber-500"
                  >
                    <div className="mt-2 flex flex-col sm:flex-row gap-6 items-start">
                      <div className="p-3 bg-white rounded-lg shadow-sm border border-stroke flex-shrink-0">
                        <QRCode value={verification.uri} size={120} />
                      </div>
                      <div className="flex-1 w-full space-y-3">
                        <p className="text-xs text-muted-foreground italic">
                          Use this QR if you need to authenticate with the authority before
                          receiving the VC.
                        </p>
                        <UriDisplay uri={verification.uri} />
                        <Button
                          className="w-full sm:w-auto bg-amber-600 hover:bg-amber-700 text-white"
                          size="sm"
                          onClick={() => handleAction("/vc-request/oid4vp", verification.uri)}
                          disabled={isProcessing}
                        >
                          {isProcessing ? (
                            <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                          ) : (
                            <Key className="mr-2 h-4 w-4" />
                          )}
                          Present in Agent
                        </Button>
                      </div>
                    </div>
                  </DetailItem>
                </div>
              )}
            </CardContent>
          </Card>
        ) : interaction ? (
          <Card className="mt-6 opacity-50 border-dashed">
            <CardHeader>
              <CardTitle className="flex items-center gap-2 text-muted-foreground">
                <CheckCircle2 className="h-5 w-5" />
                Verification (OIDC4VP)
              </CardTitle>
              <CardDescription>
                Not required — the authority didn't ask for a presentation.
              </CardDescription>
            </CardHeader>
          </Card>
        ) : null}

        {/* ===== Raw JSON (debugging) =============================================== */}
        <RawDetails details={details} />
      </PageSection>
    </PageLayout>
  );
}

function getTimelineData(req: SentGrant) {
  const status = req.status?.toLowerCase() || "";
  const pastEvents: { id: string; title: string; date?: string | null }[] = [
    { id: "created", title: "Request Created", date: req.created_at },
  ];

  if (status === "pending") {
    pastEvents.push({ id: "processing", title: "Processing" });
  } else if (status === "rejected") {
    pastEvents.push({ id: "processing", title: "Processing" });
    pastEvents.push({ id: "pending", title: "Pending" });
  } else if (status === "approved") {
    pastEvents.push({ id: "processing", title: "Processing" });
    pastEvents.push({ id: "pending", title: "Pending" });
  } else if (status === "finalized") {
    pastEvents.push({ id: "processing", title: "Processing" });
    pastEvents.push({ id: "pending", title: "Pending" });
    pastEvents.push({ id: "approved", title: "Approved" });
  }

  let instruction = "";
  switch (status) {
    case "processing":
      instruction = "The Authorization Server (AS) has not yet evaluated the request.";
      break;
    case "pending":
      instruction =
        "Waiting for your authentication. Scan the QR in the Verification section to authenticate with the authority.";
      break;
    case "rejected":
      instruction = "Your request has been rejected. No further action can be taken.";
      break;
    case "approved":
      instruction =
        "The request has been approved. You can now claim your Verifiable Credential.";
      break;
    case "finalized":
      instruction =
        "You have successfully claimed the Verifiable Credential. The process is complete.";
      break;
    default:
      instruction = "Unknown state.";
      break;
  }

  return { pastEvents, instruction };
}

function UriDisplay({ uri }: { uri: string }) {
  const [copied, setCopied] = useState(false);

  const handleCopy = () => {
    navigator.clipboard.writeText(uri);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const truncatedUri =
    uri.length > 50 ? `${uri.substring(0, 25)}...${uri.substring(uri.length - 20)}` : uri;

  return (
    <div className="flex items-center gap-2 p-2 bg-muted/50 rounded border border-stroke overflow-hidden">
      <span className="font-mono text-[10px] truncate flex-1">{truncatedUri}</span>
      <Button variant="ghost" size="icon" className="h-7 w-7" onClick={handleCopy}>
        {copied ? <Check className="h-3 w-3 text-green-500" /> : <Copy className="h-3 w-3" />}
      </Button>
    </div>
  );
}

function SecretField({ value }: { value?: string | null }) {
  const [revealed, setRevealed] = useState(false);
  if (!value) return <span className="font-mono text-[10px] text-muted-foreground">—</span>;
  return (
    <div className="flex items-center gap-2">
      <span className="font-mono text-[10px] break-all flex-1 select-all">
        {revealed ? value : "•".repeat(Math.min(value.length, 24))}
      </span>
      <button
        type="button"
        onClick={() => setRevealed((v) => !v)}
        className="text-muted-foreground hover:text-foreground transition-colors p-1 rounded hover:bg-white/5"
        aria-label={revealed ? "Hide value" : "Reveal value"}
        title={revealed ? "Hide" : "Reveal"}
      >
        {revealed ? <EyeOff className="h-3 w-3" /> : <Eye className="h-3 w-3" />}
      </button>
    </div>
  );
}

function DetailItem({
  label,
  children,
  labelClassName,
}: {
  label: string;
  children: React.ReactNode;
  labelClassName?: string;
}) {
  return (
    <div className="flex flex-col gap-1.5">
      <span
        className={`text-xs font-semibold uppercase tracking-wider ${
          labelClassName || "text-muted-foreground"
        }`}
      >
        {label}
      </span>
      <div className="text-sm font-medium">{children}</div>
    </div>
  );
}

function RawDetails({ details }: { details: DetailsResponse["data"] | null }) {
  const [open, setOpen] = useState(false);
  if (!details) return null;
  return (
    <div className="mt-6 border border-white/10 rounded-xl overflow-hidden bg-white/[0.02]">
      <button
        onClick={() => setOpen(!open)}
        className="w-full flex items-center justify-between p-3 text-xs font-mono uppercase tracking-widest text-muted-foreground hover:bg-white/[0.04] transition-colors"
      >
        <span className="flex items-center gap-2">
          <FileJson className="h-3 w-3" />
          Raw JSON
        </span>
        <span>{open ? "−" : "+"}</span>
      </button>
      {open && (
        <pre className="p-4 bg-black/40 font-mono text-[11px] text-muted-foreground/90 whitespace-pre-wrap break-all leading-relaxed overflow-x-auto">
          {JSON.stringify(details, null, 2)}
        </pre>
      )}
    </div>
  );
}
