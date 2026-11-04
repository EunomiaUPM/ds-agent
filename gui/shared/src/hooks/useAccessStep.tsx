// The step before an action at a peer: asks for what is missing and, if the peer wants credentials,
// shows who asks for what in the same dialog, and goes on with the action once they are presented.

import { useRef, useState } from "react";
import {
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  encodePathId,
} from "@eunomia/ui";
import { ConnectPeerForm } from "shared/src/components/ConnectPeerForm";
import { useGetParticipantById } from "shared/src/data/orval/participants/participants";
import { holdsAccess, prepareAccess, type AccessNeed } from "shared/src/data/peerConnection";
import type { FederatedParticipant } from "shared/src/data/useFederatedCatalog";

const WAIT_POLLS = 20;
const POLL_MS = 1_000;

interface Pending {
  peerId: string;
  grantId: string;
  need: AccessNeed;
  action: () => Promise<void>;
}

export const useAccessStep = () => {
  const [pending, setPending] = useState<Pending | null>(null);
  const finishing = useRef(false);
  const { data: peer } = useGetParticipantById(encodePathId(pending?.peerId ?? ""), {
    query: { enabled: !!pending },
  });

  /** Runs `action` once `need` is covered, asking the user first if it takes credentials. */
  const withAccess = async (
    peerId: string,
    need: AccessNeed,
    requested: boolean,
    action: () => Promise<void>,
  ) => {
    const step = await prepareAccess(peerId, need, requested);
    if (step.pending) setPending({ peerId, grantId: step.grantId, need, action });
    else await action();
  };

  const finish = async () => {
    if (!pending || finishing.current) return;
    finishing.current = true;
    for (let polls = 0; polls < WAIT_POLLS; polls++) {
      if (await holdsAccess(pending.peerId, pending.need)) break;
      await new Promise((resolve) => setTimeout(resolve, POLL_MS));
    }
    finishing.current = false;
    setPending(null);
    await pending.action();
  };

  const participant =
    peer?.status === 200 ? (peer.data as unknown as FederatedParticipant) : undefined;
  const form =
    pending && participant ? (
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>Credentials needed</DialogTitle>
          <DialogDescription>Present them to go on with what you asked for.</DialogDescription>
        </DialogHeader>
        <ConnectPeerForm
          participant={participant}
          pending={pending.grantId}
          onSent={finish}
          onCancel={() => setPending(null)}
        />
      </DialogContent>
    ) : null;

  return { withAccess, form };
};
