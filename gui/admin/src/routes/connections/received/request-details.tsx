import { createFileRoute, Link } from "@tanstack/react-router";
import { useQuery } from "@tanstack/react-query";
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
import { useState } from "react";
import { FormatDate } from "shared/src/components/ui/format-date";
import {
  AlertCircle,
  ArrowLeft,
  CheckCircle2,
  Clock,
  Eye,
  EyeOff,
  FileJson,
  Inbox,
  Key,
  Layers,
} from "lucide-react";
import { getFriendlyVCType } from "shared/src/lib/utils";

interface RecvGrant {
  id: string;
  participant_nick: string;
  kind: string;
  token?: string | null;
  vc_type_config?: string[] | null;
  status: string;
  created_at: string;
  ended_at?: string | null;
}

interface RecvInteraction {
  id: string;
  start: any[];
  method: string;
  callback_uri: string;
  key_source?: any;
  client_nonce: string;
  hash_method: any;
  hints?: string | null;
  continue_endpoint: string;
  continue_id: string;
  continue_token: string;
  continue_wait?: number | null;
  as_nonce: string;
  interact_ref: string;
  hash: string;
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

interface RecvVerification {
  id: string;
  state: string;
  nonce: string;
  vc_type: any[];
  audience: string;
  holder?: string | null;
  vpt?: string | null;
  vcs: string[];
  status: string;
  created_at: string;
  ended_at?: string | null;
}

interface DetailsResponse {
  status: number;
  data: {
    grant: RecvGrant;
    resource_req: ResourceReq | null;
    interaction: RecvInteraction | null;
    verification: RecvVerification | null;
  };
}

const searchSchema = z.object({
  requestId: z.string(),
});

// @ts-ignore
export const Route = createFileRoute("/connections/received/request-details")({
  validateSearch: (search) => searchSchema.parse(search),
  component: ReceivedRequestDetails,
});

function ReceivedRequestDetails() {
  const { requestId } = Route.useSearch();

  const { data: response, isLoading } = useQuery({
    queryKey: ["gate-received-details", requestId],
    queryFn: () =>
      customInstance<DetailsResponse>(
        `/gate/request/${encodeURIComponent(requestId)}/details`,
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
          <p>We couldn't find an incoming request with the ID: {requestId}</p>
          <Link to="/connections/received" className="mt-4">
            <Button variant="outline">
              <ArrowLeft className="mr-2 h-4 w-4" />
              Back to Received
            </Button>
          </Link>
        </div>
      </PageSection>
    );
  }

  return (
    <>
      <div className="flex items-center justify-between mb-4">
        <Link to="/connections/received">
          <Button variant="ghost" size="sm">
            <ArrowLeft className="h-4 w-4 mr-2" />
            Back to Received
          </Button>
        </Link>
        <div className="text-xs text-muted-foreground font-mono">ID: {grant.id}</div>
      </div>

      <PageSection title={`Incoming: ${grant.participant_nick || "Peer"}`}>
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
          {/* ===== Grant ============================================================= */}
          <Card className="md:col-span-2">
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Inbox className="h-5 w-5 text-primary" />
                Grant
              </CardTitle>
              <CardDescription>The access-token request a peer sent to us.</CardDescription>
            </CardHeader>
            <CardContent>
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
                <DetailItem label="Peer Nick">{grant.participant_nick || "-"}</DetailItem>
                <DetailItem label="Issued Token">
                  <SecretField value={grant.token} />
                </DetailItem>
                <DetailItem label="VC Types Requested">
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
                <DetailItem label="Received At">
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
              <CardDescription>GNAP interaction handshake initiated by the peer.</CardDescription>
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
                    {interaction.continue_endpoint}
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
                <DetailItem label="AS Nonce">
                  <SecretField value={interaction.as_nonce} />
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
                Not required — this grant was approved directly without a GNAP handshake.
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
                We requested a presentation from the peer before issuing the token.
              </CardDescription>
            </CardHeader>
            <CardContent>
              <div className="grid grid-cols-1 sm:grid-cols-2 gap-y-6 gap-x-4">
                <DetailItem label="Status">
                  <Badge variant={"status"} state={verification.status}>
                    {verification.status}
                  </Badge>
                </DetailItem>
                <DetailItem label="State">
                  <SecretField value={verification.state} />
                </DetailItem>
                <DetailItem label="Nonce">
                  <SecretField value={verification.nonce} />
                </DetailItem>
                <DetailItem label="Audience">
                  <span className="font-mono text-[10px] break-all">{verification.audience}</span>
                </DetailItem>
                <DetailItem label="Holder">
                  {verification.holder ? (
                    <span className="font-mono text-[10px] break-all">{verification.holder}</span>
                  ) : (
                    <span className="text-xs text-muted-foreground">—</span>
                  )}
                </DetailItem>
                <DetailItem label="VC Types">
                  {(verification.vc_type ?? []).length === 0 ? (
                    <span className="text-xs text-muted-foreground">—</span>
                  ) : (
                    <div className="flex flex-wrap gap-1">
                      {verification.vc_type.map((t, idx) => (
                        <Badge key={idx} variant="role">
                          {typeof t === "string" ? t : Object.keys(t ?? {})[0] ?? "?"}
                        </Badge>
                      ))}
                    </div>
                  )}
                </DetailItem>
                <DetailItem label="Presented VCs count">
                  <span className="font-mono text-xs">{verification.vcs?.length ?? 0}</span>
                </DetailItem>
                <DetailItem label="Created At">
                  <FormatDate date={verification.created_at} />
                </DetailItem>
                <DetailItem label="Ended At">
                  {verification.ended_at ? <FormatDate date={verification.ended_at} /> : "—"}
                </DetailItem>
              </div>
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
                Not required — we didn't ask the peer for a presentation.
              </CardDescription>
            </CardHeader>
          </Card>
        ) : null}

        <RawDetails details={details} />
      </PageSection>
    </>
  );
}

function getTimelineData(req: RecvGrant) {
  const status = req.status?.toLowerCase() || "";
  const pastEvents: { id: string; title: string; date?: string | null }[] = [
    { id: "received", title: "Request Received", date: req.created_at },
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
      instruction = "We have received the request and are evaluating it.";
      break;
    case "pending":
      instruction = "Waiting for the peer to present credentials before issuing the token.";
      break;
    case "approved":
      instruction = "We approved the request. The peer has the access token.";
      break;
    case "finalized":
      instruction = "The connection lifecycle has ended (token expired or session closed).";
      break;
    case "rejected":
      instruction = "We rejected the request. No further action will be taken.";
      break;
    default:
      instruction = "Unknown state.";
      break;
  }

  return { pastEvents, instruction };
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
        <CardDescription>The resource access the peer is asking for.</CardDescription>
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

