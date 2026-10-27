import { AlertTriangle, BadgeCheck, Loader2, ShieldAlert, ShieldCheck } from "lucide-react";
import { useState } from "react";
import { Button } from "shared/src/components/ui/button";
import { FormatDate } from "shared/src/components/ui/format-date";
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

const VerifierLine = ({ verifier }: { verifier: VerifierCheck }) => {
  switch (verifier.state) {
    case "verified":
      return (
        <p className="flex items-start gap-2 text-xs text-success-700 dark:text-success-400">
          <BadgeCheck className="h-4 w-4 shrink-0" />
          <span>
            Verified: the request is signed by{" "}
            <span className="font-mono break-all">{verifier.did}</span>, the participant you are
            connecting to.
          </span>
        </p>
      );
    case "same_origin":
      return (
        <p className="flex items-start gap-2 text-xs text-amber-600">
          <ShieldCheck className="h-4 w-4 shrink-0" />
          Unsigned request, answered to the same host you sent the request to.
        </p>
      );
    case "unsigned":
      return (
        <p className="flex items-start gap-2 text-xs text-amber-600">
          <AlertTriangle className="h-4 w-4 shrink-0" />
          Unsigned request: who asks for it cannot be checked.
        </p>
      );
    case "mismatch":
      return (
        <p className="flex items-start gap-2 text-xs text-destructive">
          <ShieldAlert className="h-4 w-4 shrink-0" />
          <span>
            Signed by <span className="font-mono break-all">{verifier.did}</span>, not by the
            participant you are connecting to.
          </span>
        </p>
      );
    case "untrusted":
      return (
        <p className="flex items-start gap-2 text-xs text-destructive">
          <ShieldAlert className="h-4 w-4 shrink-0" />
          Invalid signature: {verifier.reason}
        </p>
      );
  }
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
  /** Called once the request is sent; the grant may still be on its way. */
  onSent?: () => void;
  onCancel?: () => void;
}

/** Asks `participant` for a grant, then shows who asks for which credentials before presenting. */
export const ConnectPeerForm = ({ participant, widen, onSent, onCancel }: ConnectPeerFormProps) => {
  const queryClient = useQueryClient();
  const [visibility, setVisibility] = useState<Visibility>("Private");
  const [sending, setSending] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [grantId, setGrantId] = useState<string | null>(null);
  const [resolved, setResolved] = useState<ResolvedPresentation | null>(null);
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
      onSent?.();
    }
  };

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
        onSent?.();
      } else {
        onCancel?.();
      }
    });

  const credentials = resolved ? requestedCredentials(resolved) : [];

  return (
    <div className="space-y-4 text-sm">
      {widen ? (
        <p>
          <strong>{nick}</strong> needs more access for{" "}
          <strong>{widen.title || widen.datasetId}</strong>. Do you want to continue?
        </p>
      ) : (
        <p>
          You are going to talk to <strong>{nick}</strong>.
        </p>
      )}
      <dl className="grid grid-cols-[auto_1fr] gap-x-3 gap-y-1.5 rounded-lg border border-ink/10 bg-background-800/40 p-3 text-xs">
        <dt className="text-muted-foreground">DID</dt>
        <dd className="font-mono break-all">{participant.participant_id}</dd>
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
      </dl>
      <div className={widen ? "hidden" : "space-y-1.5"}>
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
      </div>

      {resolved && (
        <div className="space-y-2 rounded-lg border border-ink/10 bg-background-800/40 p-3">
          <span className="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
            Credentials requested
          </span>
          {credentials.length > 0 ? (
            <ul className="list-disc pl-5 text-xs">
              {credentials.map((credential, idx) => (
                <li key={idx} className="font-mono">
                  {credential}
                </li>
              ))}
            </ul>
          ) : (
            <p className="text-xs text-muted-foreground">The request could not be read.</p>
          )}
          <VerifierLine verifier={resolved.verifier} />
        </div>
      )}

      {error && <p className="text-xs text-destructive">{error}</p>}
      <div className="flex justify-end gap-2">
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
              Present
            </Button>
          </>
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
