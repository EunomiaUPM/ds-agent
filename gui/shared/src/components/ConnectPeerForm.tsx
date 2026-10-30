import { Loader2, ShieldCheck } from 'lucide-react';
import { useState } from 'react';
import { Button } from "@eunomia/ui";
import { FormatDate } from "@eunomia/ui";
import {
    Select, SelectContent, SelectItem, SelectTrigger, SelectValue
} from "@eunomia/ui";
import { Visibility } from 'shared/src/data/orval/model/visibility';
import { connectToPeer } from 'shared/src/data/orval/onboard/onboard';
import { discoverPeer } from 'shared/src/data/peerConnection';

import { useQueryClient } from '@tanstack/react-query';

import type { FederatedParticipant } from "shared/src/data/useFederatedCatalog";

/** What each visibility means for a connection. */
export const VISIBILITY_HELP: Record<Visibility, string> = {
  Private: "Only you and the roles above you.",
  Anonymous: "Everyone, without your name.",
  Public: "Everyone.",
};

interface ConnectPeerFormProps {
  participant: FederatedParticipant;
  /** Called once the request is sent; the grant may still be on its way. */
  onSent?: () => void;
  onCancel?: () => void;
}

/** Asks `participant` for a grant with the chosen visibility, presenting the dataspace VC. */
export const ConnectPeerForm = ({ participant, onSent, onCancel }: ConnectPeerFormProps) => {
  const queryClient = useQueryClient();
  const [visibility, setVisibility] = useState<Visibility>("Private");
  const [sending, setSending] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const nick = participant.participant_nick || "this participant";

  const connect = async () => {
    setSending(true);
    setError(null);
    try {
      const peer = await discoverPeer(participant.base_url);
      await connectToPeer({
        id: peer.id,
        nick,
        url: peer.accessUrl,
        actions: ["talk"],
        visibility,
        auto: true,
        requested: true,
      });
      await queryClient.invalidateQueries({
        predicate: (q) =>
          typeof q.queryKey[0] === "string" && q.queryKey[0].startsWith("/peer-connection"),
      });
      onSent?.();
    } catch (err: any) {
      setError(err?.message || "Could not send the request.");
    } finally {
      setSending(false);
    }
  };

  return (
    <div className="space-y-4 text-sm">
      <p>
        You are going to talk to <strong>{nick}</strong>.
      </p>
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
      <p className="flex items-start gap-2 text-xs text-muted-foreground">
        <ShieldCheck className="h-4 w-4 shrink-0 text-brand-sky" />
        You will authenticate by presenting your DataSpace Participant credential.
      </p>
      <div className="space-y-1.5">
        <span className="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
          Visibility
        </span>
        <Select
          value={visibility}
          onValueChange={(v) => setVisibility(v as Visibility)}
          disabled={sending}
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
      {error && <p className="text-xs text-destructive">{error}</p>}
      <div className="flex justify-end gap-2">
        {onCancel && (
          <Button size="sm" variant="ghost" onClick={onCancel} disabled={sending}>
            Cancel
          </Button>
        )}
        <Button size="sm" onClick={connect} disabled={sending}>
          {sending ? (
            <>
              <Loader2 className="mr-2 h-4 w-4 animate-spin" />
              Connecting…
            </>
          ) : (
            "Connect"
          )}
        </Button>
      </div>
    </div>
  );
};

export default ConnectPeerForm;
