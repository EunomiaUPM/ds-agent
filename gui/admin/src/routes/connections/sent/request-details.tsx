import { createFileRoute, Link } from "@tanstack/react-router";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { customInstance } from "shared/src/data/orval-mutator";
import { PageSection } from "shared/src/components/layout/PageSection";
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  CardDescription,
} from "shared/src/components/ui/card";
import { Badge } from "shared/src/components/ui/badge";
import { Button } from "shared/src/components/ui/button";
import * as z from "zod";
import QRCode from "react-qr-code";
import { useState } from "react";
import { FormatDate } from "shared/src/components/ui/format-date";
import {
  AlertCircle,
  ArrowLeft,
  Check,
  CheckCircle2,
  Clock,
  Copy,
  ExternalLink,
  Eye,
  EyeOff,
  FileJson,
  Info,
  Key,
  Layers,
  Loader2,
} from "lucide-react";

interface SentGrant {
  id: string;
  participant_id: string;
  participant_nick: string;
  grant_endpoint: string;
  kind: string;
  status: string;
  token?: string | null;
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

interface ResourceReq {
  id: string;
  type: any;
  actions: any[];
  locations?: string[] | null;
  datatypes?: string[] | null;
  identifier?: string | null;
  privileges?: string[] | null;
  label?: string | null;
  flags?: any[] | null;
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
    resource_req: ResourceReq | null;
    interaction: SentInteraction | null;
    verification: SentVerification | null;
  };
}

const searchSchema = z.object({
  requestId: z.string(),
});

// @ts-ignore
export const Route = createFileRoute("/connections/sent/request-details")({
  validateSearch: (search) => searchSchema.parse(search),
  component: SentRequestDetails,
});

function SentRequestDetails() {
  const { requestId } = Route.useSearch();
  const queryClient = useQueryClient();
  const [isProcessing, setIsProcessing] = useState(false);

  const queryKey = ["peer-connection-sent-details", requestId];
  const { data: response, isLoading } = useQuery({
    queryKey,
    queryFn: () =>
      customInstance<DetailsResponse>(
        `/peer-connection/request/${encodeURIComponent(requestId)}/details`,
        { method: "GET" },
      ),
    enabled: !!requestId,
  });

  const details = response?.status === 200 ? response.data : null;
  const grant = details?.grant ?? null;
  const resourceReq = details?.resource_req ?? null;
  const interaction = details?.interaction ?? null;
  const verification = details?.verification ?? null;
  const timelineData = grant ? getTimelineData(grant) : null;

  const handlePresent = async (uri: string) => {
    if (!grant) return;
    setIsProcessing(true);
    try {
      await customInstance(`/peer-connection/oid4vp/${encodeURIComponent(grant.id)}`, {
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
      <PageSection>
        <div className="flex justify-center py-20">
          <Badge variant="infoLighter" className="animate-pulse">
            Loading details...
          </Badge>
        </div>
      </PageSection>
    );
  }

  if (!grant) {
    return (
      <PageSection>
        <div className="flex flex-col items-center justify-center py-20 text-muted-foreground">
          <AlertCircle className="h-12 w-12 mb-4 opacity-20" />
          <p>We couldn't find a connection request with the ID: {requestId}</p>
          <Link to="/connections/sent" className="mt-4">
            <Button variant="outline">
              <ArrowLeft className="mr-2 h-4 w-4" />
              Back to Sent
            </Button>
          </Link>
        </div>
      </PageSection>
    );
  }

  return (
    <>
      <div className="flex items-center justify-between mb-4">
        <Link to="/connections/sent">
          <Button variant="ghost" size="sm">
            <ArrowLeft className="h-4 w-4 mr-2" />
            Back to Sent
          </Button>
        </Link>
        <div className="text-xs text-muted-foreground font-mono">ID: {grant.id}</div>
      </div>

      <PageSection title={`Connection: ${grant.participant_nick || "Provider"}`}>
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
          {/* ===== Grant ============================================================= */}
          <Card className="md:col-span-2">
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Info className="h-5 w-5 text-primary" />
                Grant
              </CardTitle>
              <CardDescription>The access-token request sent to the provider.</CardDescription>
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
                <DetailItem label="Auto authentication">
                  <Badge variant={grant.auto ? "default" : "info"} className="text-[10px]">
                    {grant.auto ? "ON" : "OFF"}
                  </Badge>
                </DetailItem>
                <DetailItem label="Provider DID">
                  <span className="font-mono text-xs break-all">{grant.participant_id || "-"}</span>
                </DetailItem>
                <DetailItem label="Provider Nick">{grant.participant_nick || "-"}</DetailItem>
                <DetailItem label="Grant Endpoint">
                  <div className="flex items-center gap-1 text-xs text-muted-foreground break-all">
                    {grant.grant_endpoint || "-"}
                    {grant.grant_endpoint && <ExternalLink className="h-3 w-3" />}
                  </div>
                </DetailItem>
                <DetailItem label="AS Assigned ID">
                  <SecretField value={grant.as_assigned_id} />
                </DetailItem>
                <DetailItem label="Access Token">
                  <SecretField value={grant.token} />
                </DetailItem>
                <DetailItem label="Created At">
                  {grant.created_at ? <FormatDate date={grant.created_at} /> : "-"}
                </DetailItem>
                <DetailItem label="Ended At">
                  {grant.ended_at ? <FormatDate date={grant.ended_at} /> : "-"}
                </DetailItem>
              </div>
            </CardContent>
          </Card>

          {/* ===== Timeline =========================================================== */}
          <Card>
            <CardHeader>
              <CardTitle>Timeline</CardTitle>
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

        {/* ===== Resource Request =================================================== */}
        <ResourceReqCard resourceReq={resourceReq} />

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
                Not required — the provider returned the access token directly without a GNAP
                handshake.
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
                Verification (OID4VP)
              </CardTitle>
              <CardDescription>
                The provider requested a presentation before issuing the token.
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
                            Scan this QR or click below to present credentials through the agent
                            wallet.
                          </p>
                          <UriDisplay uri={verification.uri} />
                          <Button
                            className="w-full sm:w-auto bg-amber-600 hover:bg-amber-700 text-white"
                            size="sm"
                            onClick={() => handlePresent(verification.uri)}
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
                Verification (OID4VP)
              </CardTitle>
              <CardDescription>
                Not required — the provider didn't ask for a presentation.
              </CardDescription>
            </CardHeader>
          </Card>
        ) : null}

        <RawDetails details={details} />
      </PageSection>
    </>
  );
}

function getTimelineData(req: SentGrant) {
  const status = req.status?.toLowerCase() || "";
  const pastEvents: { id: string; title: string; date?: string | null }[] = [
    { id: "created", title: "Request Created", date: req.created_at },
  ];

  if (status === "pending") {
    pastEvents.push({ id: "processing", title: "Processing" });
  } else if (status === "approved") {
    pastEvents.push({ id: "processing", title: "Processing" });
    pastEvents.push({ id: "pending", title: "Pending" });
  } else if (status === "finalized") {
    pastEvents.push({ id: "processing", title: "Processing" });
    pastEvents.push({ id: "pending", title: "Pending" });
    pastEvents.push({ id: "approved", title: "Approved" });
  } else if (status === "rejected") {
    pastEvents.push({ id: "processing", title: "Processing" });
    pastEvents.push({ id: "pending", title: "Pending" });
  }

  let instruction = "";
  switch (status) {
    case "processing":
      instruction = "The Authorization Server (AS) has not yet evaluated the request.";
      break;
    case "pending":
      instruction =
        "Authentication required. Use the QR in the Verification section to present credentials.";
      break;
    case "approved":
      instruction = "You have successfully authenticated. An access token is available for use.";
      break;
    case "finalized":
      instruction = "The connection lifecycle has ended (token expired or session closed).";
      break;
    case "rejected":
      instruction = "The request was rejected. No further action can be taken.";
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

function enumLabel(v: any): string {
  if (v === null || v === undefined) return "?";
  if (typeof v === "string") return v;
  if (typeof v === "object") {
    const key = Object.keys(v)[0];
    if (!key) return "?";
    const inner = v[key];
    return typeof inner === "string" ? `${key}(${inner})` : key;
  }
  return String(v);
}

function ResourceReqCard({ resourceReq }: { resourceReq: ResourceReq | null }) {
  if (!resourceReq) {
    return (
      <Card className="mt-6 opacity-50 border-dashed">
        <CardHeader>
          <CardTitle className="flex items-center gap-2 text-muted-foreground">
            <Layers className="h-5 w-5" />
            Resource Request
          </CardTitle>
          <CardDescription>No resource access bound to this grant.</CardDescription>
        </CardHeader>
      </Card>
    );
  }
  return (
    <Card className="mt-6">
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <Layers className="h-5 w-5 text-primary" />
          Resource Request
        </CardTitle>
        <CardDescription>The resource access this grant is asking for.</CardDescription>
      </CardHeader>
      <CardContent>
        <div className="grid grid-cols-1 sm:grid-cols-2 gap-y-6 gap-x-4">
          <DetailItem label="Type">
            <Badge variant="info" className="font-mono">
              {enumLabel(resourceReq.type)}
            </Badge>
          </DetailItem>
          <DetailItem label="Identifier">
            {resourceReq.identifier ? (
              <span className="font-mono text-[10px] break-all">{resourceReq.identifier}</span>
            ) : (
              <span className="text-xs text-muted-foreground">—</span>
            )}
          </DetailItem>
          <DetailItem label="Label">
            {resourceReq.label ? (
              <span className="text-sm">{resourceReq.label}</span>
            ) : (
              <span className="text-xs text-muted-foreground">—</span>
            )}
          </DetailItem>
          <DetailItem label="Actions">
            {(resourceReq.actions ?? []).length === 0 ? (
              <span className="text-xs text-muted-foreground">—</span>
            ) : (
              <div className="flex flex-wrap gap-1">
                {resourceReq.actions.map((a, idx) => (
                  <Badge key={idx} variant="role">
                    {enumLabel(a)}
                  </Badge>
                ))}
              </div>
            )}
          </DetailItem>
          <DetailItem label="Locations">
            {(resourceReq.locations ?? []).length === 0 ? (
              <span className="text-xs text-muted-foreground">—</span>
            ) : (
              <div className="flex flex-col gap-1">
                {resourceReq.locations!.map((loc, idx) => (
                  <span key={idx} className="font-mono text-[10px] break-all">
                    {loc}
                  </span>
                ))}
              </div>
            )}
          </DetailItem>
          <DetailItem label="Datatypes">
            {(resourceReq.datatypes ?? []).length === 0 ? (
              <span className="text-xs text-muted-foreground">—</span>
            ) : (
              <div className="flex flex-wrap gap-1">
                {resourceReq.datatypes!.map((d, idx) => (
                  <Badge key={idx} variant="info">
                    {d}
                  </Badge>
                ))}
              </div>
            )}
          </DetailItem>
          <DetailItem label="Privileges">
            {(resourceReq.privileges ?? []).length === 0 ? (
              <span className="text-xs text-muted-foreground">—</span>
            ) : (
              <div className="flex flex-wrap gap-1">
                {resourceReq.privileges!.map((p, idx) => (
                  <Badge key={idx} variant="info">
                    {p}
                  </Badge>
                ))}
              </div>
            )}
          </DetailItem>
          <DetailItem label="Flags">
            {(resourceReq.flags ?? []).length === 0 ? (
              <span className="text-xs text-muted-foreground">—</span>
            ) : (
              <div className="flex flex-wrap gap-1">
                {resourceReq.flags!.map((f, idx) => (
                  <Badge key={idx} variant="info" className="font-mono">
                    {enumLabel(f)}
                  </Badge>
                ))}
              </div>
            )}
          </DetailItem>
        </div>
      </CardContent>
    </Card>
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
