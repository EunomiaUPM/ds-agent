import {
  AlertTriangle,
  Check,
  CircleHelp,
  Copy,
  IdCard,
  Loader2,
  ShieldAlert,
  ShieldCheck,
  ShieldQuestion,
  ShieldX,
  type LucideIcon,
} from "lucide-react";
import { type ReactNode, useEffect, useState } from "react";
import QRCode from "react-qr-code";
import { Button } from "shared/src/components/ui/button";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "shared/src/components/ui/tooltip";
import { getFriendlyVCType } from "shared/src/lib/utils";
import { FormatDate } from "shared/src/components/ui/format-date";
import { FormatDid } from "shared/src/components/ui/format-did";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "shared/src/components/ui/select";
import { Visibility } from "shared/src/data/orval/model/visibility";
import { connectToPeer } from "shared/src/data/orval/onboard/onboard";
import { customInstance } from "shared/src/data/orval-mutator";
import { discoverPeer } from "shared/src/data/peerConnection";

import { useQueryClient } from "@tanstack/react-query";

import type { FederatedParticipant } from "shared/src/data/useFederatedCatalog";

/** What each visibility means for a connection. */
export const VISIBILITY_HELP: Record<Visibility, string> = {
  Private: "Only you and the roles above you.",
  Anonymous: "Everyone, without your name.",
  Public: "Everyone.",
};

/** Who asks for the presentation, as the agent checked it against the peer. */
type VerifierCheck =
  | { state: "verified"; did: string }
  | { state: "mismatch"; did: string }
  | { state: "same_origin" }
  | { state: "unsigned" }
  | { state: "untrusted"; reason: string };

interface ResolvedPresentation {
  uri: string;
  request: {
    requested?: {
      dcql?: { credentials?: CredentialQuery[] };
      definition?: { input_descriptors?: { id: string }[] };
    };
  } | null;
  verifier: VerifierCheck;
}

interface CredentialQuery {
  id: string;
  meta?: { type_values?: string[][] };
}

/** The credentials a request asks for, by their most specific type. */
const requestedCredentials = (resolved: ResolvedPresentation): string[] => {
  const requested = resolved.request?.requested;
  const dcql = requested?.dcql?.credentials?.map((query) => {
    const types = query.meta?.type_values?.[0] ?? [];
    return types[types.length - 1] ?? query.id;
  });
  return dcql ?? requested?.definition?.input_descriptors?.map((d) => d.id) ?? [];
};

const blocks = (verifier: VerifierCheck) =>
  verifier.state === "mismatch" || verifier.state === "untrusted";

/** How long what appears waits for what leaves (both take 350 ms). */
const ENTER_DELAY_MS = 150;

const enterDelay = (show: boolean) => ({ transitionDelay: show ? `${ENTER_DELAY_MS}ms` : "0ms" });

const fade = (show: boolean) =>
  `transition-opacity duration-[350ms] ease-out ${show ? "opacity-100" : "opacity-0"}`;

/** Unfolds `children` when `show` and folds them away otherwise, height and opacity alike. */
const Reveal = ({
  show,
  className = "",
  children,
}: {
  show: boolean;
  className?: string;
  children: ReactNode;
}) => (
  <div
    aria-hidden={!show}
    style={enterDelay(show)}
    className={`grid transition-[grid-template-rows,opacity] duration-[350ms] ease-out ${
      show ? "grid-rows-[1fr] opacity-100" : "grid-rows-[0fr] opacity-0"
    }`}
  >
    <div className="min-h-0 overflow-hidden">
      <div className={className}>{children}</div>
    </div>
  </div>
);

/** A `?` that explains `children` on hover. */
const Hint = ({ children }: { children: ReactNode }) => (
  <TooltipProvider delayDuration={150}>
    <Tooltip>
      <TooltipTrigger asChild>
        <CircleHelp className="h-3.5 w-3.5 shrink-0 cursor-help text-muted-foreground" />
      </TooltipTrigger>
      {/* Above the popovers (z-[100]) the form may be shown in. */}
      <TooltipContent className="z-[110] max-w-xs text-xs">{children}</TooltipContent>
    </Tooltip>
  </TooltipProvider>
);

/** `uri` on one line, with a button to copy it whole. */
const CopyUri = ({ uri }: { uri: string }) => {
  const [copied, setCopied] = useState(false);
  const copy = () => {
    navigator.clipboard.writeText(uri);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };
  return (
    <div className="flex items-center gap-2 rounded-md border border-ink/10 bg-background-800/40 p-2">
      <span title={uri} className="flex-1 truncate font-mono text-xs">
        {uri}
      </span>
      <Button variant="ghost" size="icon" className="h-7 w-7" onClick={copy} title="Copy">
        {copied ? <Check className="h-3 w-3 text-success-600" /> : <Copy className="h-3 w-3" />}
      </Button>
    </div>
  );
};

/** How each check of who asks reads, and what it means. */
const VERIFIER_VIEW: Record<
  VerifierCheck["state"],
  { label: string; help: string; tone: string; Icon: LucideIcon }
> = {
  verified: {
    label: "Verified request",
    help: "The request is signed with the DID of the participant you are connecting to, so it comes from them.",
    tone: "text-success-700 dark:text-success-400",
    Icon: ShieldCheck,
  },
  same_origin: {
    label: "Unsigned, same host",
    help: "The request is not signed, but the answer goes to the same host you sent your request to.",
    tone: "text-amber-600",
    Icon: ShieldQuestion,
  },
  unsigned: {
    label: "Unsigned request",
    help: "The request is not signed: who asks for your credentials cannot be checked.",
    tone: "text-amber-600",
    Icon: AlertTriangle,
  },
  mismatch: {
    label: "Not the participant",
    help: "The request is signed by another DID, not by the participant you are connecting to. Presenting is blocked.",
    tone: "text-destructive",
    Icon: ShieldAlert,
  },
  untrusted: {
    label: "Invalid signature",
    help: "The signature of the request could not be checked. Presenting is blocked.",
    tone: "text-destructive",
    Icon: ShieldX,
  },
};

const VerifierStatus = ({ verifier, nick }: { verifier: VerifierCheck; nick: string }) => {
  const { label, help, tone, Icon } = VERIFIER_VIEW[verifier.state];
  return (
    <div className="space-y-1">
      <p className={`flex items-center gap-2 text-sm font-medium ${tone}`}>
        <Icon className="h-5 w-5 shrink-0" />
        {label}
        <Hint>{help}</Hint>
      </p>
      <p className="pl-7 text-xs text-muted-foreground">
        {verifier.state === "verified" && `Signed by ${nick}`}
        {verifier.state === "mismatch" && (
          <>
            Signed by <FormatDid did={verifier.did} />
          </>
        )}
        {verifier.state === "untrusted" && verifier.reason}
      </p>
    </div>
  );
};


/** An approved grant to widen to a dataset it does not cover yet. */
export interface WidenAccess {
  grantId: string;
  datasetId: string;
  title?: string;
}

interface ConnectPeerFormProps {
  participant: FederatedParticipant;
  /** Widens this grant instead of asking for a new one. */
  widen?: WidenAccess;
  /** Resumes this pending grant: shows what it asks for instead of asking again. */
  pending?: string;
  /** Called once the request is sent; the grant may still be on its way. */
  onSent?: () => void;
  onCancel?: () => void;
}

/** Asks `participant` for a grant, then shows who asks for which credentials before presenting. */
export const ConnectPeerForm = ({
  participant,
  widen,
  pending,
  onSent,
  onCancel,
}: ConnectPeerFormProps) => {
  const queryClient = useQueryClient();
  const [visibility, setVisibility] = useState<Visibility>("Private");
  const [sending, setSending] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [grantId, setGrantId] = useState<string | null>(null);
  const [resolved, setResolved] = useState<ResolvedPresentation | null>(null);
  const [awaiting, setAwaiting] = useState(false);
  const [qrOpen, setQrOpen] = useState(false);
  const nick = participant.participant_nick || "this participant";

  const refresh = () =>
    queryClient.invalidateQueries({
      predicate: (q) =>
        typeof q.queryKey[0] === "string" && q.queryKey[0].startsWith("/peer-connection"),
    });

  const run = async (step: () => Promise<void>) => {
    setSending(true);
    setError(null);
    try {
      await step();
    } catch (err: any) {
      setError(err?.message || "Something went wrong.");
    } finally {
      setSending(false);
    }
  };

  const resolve = async (id: string) => {
    setGrantId(id);
    const presentation = await customInstance<{ status: number; data: ResolvedPresentation }>(
      `/peer-connection/oid4vp/${encodeURIComponent(id)}/resolve`,
      { method: "POST" },
    );
    if (presentation.status === 200) {
      setResolved(presentation.data);
    } else {
      setAwaiting(true);
      onSent?.();
    }
  };

  useEffect(() => {
    if (pending) run(() => resolve(pending));
  }, [pending]);

  const widenAccess = (target: WidenAccess) =>
    run(async () => {
      const answer = await customInstance<{ status: number; data: { pending?: boolean } }>(
        `/peer-connection/request/${encodeURIComponent(target.grantId)}/access`,
        { method: "POST", data: { dataset: target.datasetId } },
      );
      await refresh();
      if (answer.status === 200 && answer.data.pending) {
        await resolve(target.grantId);
      } else {
        onSent?.();
      }
    });

  const connect = () =>
    run(async () => {
      const peer = await discoverPeer(participant.base_url);
      const answer = await connectToPeer({
        id: peer.id,
        nick,
        url: peer.accessUrl,
        visibility,
        auto: false,
        requested: true,
      });
      const id = (answer.data as { id?: string } | undefined)?.id;
      await refresh();
      if (!id) {
        onSent?.();
        return;
      }
      await resolve(id);
    });

  const answer = (action: "present" | "refuse") =>
    run(async () => {
      await customInstance(`/peer-connection/oid4vp/${encodeURIComponent(grantId!)}/${action}`, {
        method: "POST",
      });
      await refresh();
      if (action === "present") {
        setResolved(null);
        setAwaiting(true);
        onSent?.();
      } else {
        onCancel?.();
      }
    });

  const credentials = resolved ? requestedCredentials(resolved) : [];
  const visibilityRow = !!resolved && !widen && !pending;

  return (
    <div className="text-sm">
      {/* Both lines share one cell, so one fades into the other. */}
      <div className="grid">
        <p aria-hidden={!!resolved} className={`[grid-area:1/1] ${fade(!resolved)}`}>
          {widen ? (
            <>
              <strong>{nick}</strong> needs more access for{" "}
              <strong>{widen.title || widen.datasetId}</strong>. Do you want to continue?
            </>
          ) : (
            <>
              You're asking <strong>{nick}</strong> for access to its catalog.
            </>
          )}
        </p>
        <p
          aria-hidden={!resolved}
          style={enterDelay(!!resolved)}
          className={`[grid-area:1/1] ${fade(!!resolved)}`}
        >
          <strong>{nick}</strong> asks for these credentials before granting access.
        </p>
      </div>
      {/* Rows spaced by their own padding, not a gap: a folded row then takes no room. */}
      <dl className="mt-4 grid grid-cols-[auto_minmax(0,1fr)] gap-x-3 rounded-lg border border-ink/10 bg-background-800/40 px-3 py-[9px] text-xs [&>dd]:py-[3px] [&>dt]:py-[3px]">
        <dt className="text-muted-foreground">DID</dt>
        <dd>
          <FormatDid did={participant.participant_id} />
        </dd>
        <dt className="text-muted-foreground">URL</dt>
        <dd className="font-mono break-all">{participant.base_url}</dd>
        <dt className="text-muted-foreground">Type</dt>
        <dd>{participant.participant_type}</dd>
        {participant.saved_at && (
          <>
            <dt className="text-muted-foreground">Member since</dt>
            <dd>
              <FormatDate date={participant.saved_at} />
            </dd>
          </>
        )}
        {participant.is_vc_issued != null && (
          <>
            <dt className="text-muted-foreground">Credential</dt>
            <dd>{participant.is_vc_issued ? "Issued by the authority" : "Not issued yet"}</dd>
          </>
        )}
        {/* Once sent, the visibility chosen can no longer change: it becomes one more row. */}
        <div
          aria-hidden={!visibilityRow}
          style={enterDelay(visibilityRow)}
          className={`col-span-2 grid grid-cols-subgrid transition-[grid-template-rows,opacity] duration-[350ms] ease-out ${
            visibilityRow ? "grid-rows-[1fr] opacity-100" : "grid-rows-[0fr] opacity-0"
          }`}
        >
          <div className="col-span-2 grid min-h-0 grid-cols-subgrid overflow-hidden">
            <dt className="py-[3px] text-muted-foreground">Visibility</dt>
            <dd className="py-[3px]">{visibility}</dd>
          </div>
        </div>
      </dl>
      <Reveal show={!widen && !pending && !resolved} className="space-y-1.5 pt-4">
        <span className="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
          Visibility
        </span>
        <Select
          value={visibility}
          onValueChange={(v) => setVisibility(v as Visibility)}
          disabled={sending || !!grantId}
        >
          <SelectTrigger>
            <SelectValue />
          </SelectTrigger>
          <SelectContent className="z-[110]">
            {Object.values(Visibility).map((v) => (
              <SelectItem key={v} value={v}>
                {v}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
        <p className="text-xs text-muted-foreground">
          Who else sees this connection: {VISIBILITY_HELP[visibility].toLowerCase()}
        </p>
      </Reveal>

      <Reveal show={!!resolved} className="pt-4">
        {resolved && (
          <section className="flex items-center gap-3 rounded-lg border border-ink/10 p-3">
            <div className="min-w-0 flex-1 space-y-3">
              <VerifierStatus verifier={resolved.verifier} nick={nick} />
              <div className="space-y-1.5">
                <span className="flex items-center gap-1.5 text-xs text-muted-foreground">
                  Credentials to present
                  <Hint>
                    Credentials from your wallet that {nick} will see. Your other credentials are not
                    presented.
                  </Hint>
                </span>
                {credentials.length > 0 ? (
                  <div className="flex flex-wrap gap-2">
                    {credentials.map((credential, idx) => (
                      <span
                        key={idx}
                        className="inline-flex items-center gap-1.5 rounded-md border border-brand-sky/40 bg-brand-sky/10 px-2 py-1 text-xs font-medium"
                      >
                        <IdCard className="h-4 w-4 shrink-0 text-brand-sky" />
                        {getFriendlyVCType(credential)}
                      </span>
                    ))}
                  </div>
                ) : (
                  <p className="text-xs text-muted-foreground">The request could not be read.</p>
                )}
              </div>
            </div>
            {/* About as tall as the status and credentials beside it; too dense to scan here. */}
            <button
              type="button"
              title="Show it larger"
              onClick={() => setQrOpen((open) => !open)}
              className="shrink-0 cursor-zoom-in rounded-md bg-white p-1.5"
            >
              <QRCode value={resolved.uri} size={88} />
            </button>
          </section>
        )}
        {resolved && (
          <Reveal show={qrOpen} className="space-y-2 pt-3">
            <button
              type="button"
              title="Hide"
              onClick={() => setQrOpen(false)}
              className="mx-auto block cursor-zoom-out rounded-md bg-white p-3"
            >
              <QRCode value={resolved.uri} size={300} />
            </button>
            <CopyUri uri={resolved.uri} />
          </Reveal>
        )}
      </Reveal>

      {error && <p className="mt-4 text-xs text-destructive">{error}</p>}
      <div className="mt-4 flex justify-end gap-2">
        {resolved ? (
          <>
            <Button size="sm" variant="ghost" onClick={() => answer("refuse")} disabled={sending}>
              Refuse
            </Button>
            <Button
              size="sm"
              onClick={() => answer("present")}
              disabled={sending || blocks(resolved.verifier)}
            >
              {sending ? <Loader2 className="mr-2 h-4 w-4 animate-spin" /> : null}
              Present credentials
            </Button>
          </>
        ) : awaiting ? (
          <p className="flex items-center gap-2 text-xs text-muted-foreground">
            <Loader2 className="h-4 w-4 animate-spin" />
            Waiting for {nick} to accept your request.
          </p>
        ) : pending ? (
          sending && <Loader2 className="h-4 w-4 animate-spin" />
        ) : (
          <>
            {onCancel && (
              <Button size="sm" variant="ghost" onClick={onCancel} disabled={sending}>
                Cancel
              </Button>
            )}
            <Button
              size="sm"
              onClick={() => (widen ? widenAccess(widen) : connect())}
              disabled={sending}
            >
              {sending ? (
                <>
                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                  {widen ? "Requesting…" : "Connecting…"}
                </>
              ) : widen ? (
                "Continue"
              ) : (
                "Connect"
              )}
            </Button>
          </>
        )}
      </div>
    </div>
  );
};

export default ConnectPeerForm;
