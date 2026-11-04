// Connections of the session's user with the dataspace peers.
//
// What this user (not someone else of the organization) can read of each peer, from what its
// latest approved grant covers: a grant for offers or replies gives no access to the catalog. The
// federated catalog lists the peers; the grant is asked for from the catalog browser, and once it
// completes the peer is stored as a mate with the chosen visibility.

import { useMemo } from "react";
import { encodePathId } from "@eunomia/ui";
import type { AccessItem } from "./orval/model/accessItem";
import { getDidFromUrl } from "./orval/did/did";
import { useGetSessionUser } from "./orval/gateway/gateway";
import { useGetAllPeerConnectionRequests } from "./orval/onboard/onboard";
import type { DidService } from "./orval/model/didService";
import type { SentGrant } from "./orval/model/sentGrant";
import { customInstance } from "./orval-mutator";

/** What this user can read of a peer: its catalog, only datasets by id, nothing yet because a
 * grant is on its way, or nothing (a denied grant is finalized, RFC 9635 §1.5). */
export type PeerConnection = "catalog" | "datasets" | "pending" | "none";

/** A peer as its DID document describes it. */
export interface DiscoveredPeer {
  id: string;
  services: DidService[];
  /** Where grant requests go: `<gate>/access`. */
  accessUrl: string;
}

/** The peer's DID and gate, from the DID document at `baseUrl` (the gateway fetches it). */
export const discoverPeer = async (baseUrl: string): Promise<DiscoveredPeer> => {
  const clean = baseUrl.replace(/\/$/, "");
  const res = await getDidFromUrl(encodeURIComponent(clean));
  if (res.status !== 200) throw new Error("Could not read the participant's DID document");
  const services = res.data.service ?? [];
  const gate =
    services.find((s) => s.type === "AuthorizationServer")?.serviceEndpoint ??
    `${clean}/api/v1/gate`;
  return { id: res.data.id, services, accessUrl: `${gate.replace(/\/$/, "")}/access` };
};

/** Whether `granted` covers the peer's catalog: named, or through the datasets of a location. */
export const coversCatalog = (granted: AccessItem[]) =>
  granted.some(
    (access) =>
      access.type.endsWith("/catalog") ||
      (access.type.endsWith("/dataset") && (access.locations ?? []).length > 0),
  );

/** The datasets `granted` names by id. */
export const grantedDatasets = (granted: AccessItem[]) => [
  ...new Set(
    granted.flatMap((access) =>
      access.type.endsWith("/dataset") && access.identifier ? [access.identifier] : [],
    ),
  ),
];

/** What `granted` lets this user read of the peer. */
const reachOf = (granted: AccessItem[]): PeerConnection => {
  if (coversCatalog(granted)) return "catalog";
  return grantedDatasets(granted).length > 0 ? "datasets" : "none";
};

/** What `granted` lets this user see, in the words of the catalog cards. */
export const accessLabel = (granted: AccessItem[]) => {
  const count = grantedDatasets(granted).length;
  const datasets = `${count} dataset${count === 1 ? "" : "s"}`;
  if (coversCatalog(granted)) return count > 0 ? `Catalog + ${datasets}` : "Catalog only";
  return count > 0 ? datasets : "No access";
};

/** What an action at a peer needs from its AS. */
export type AccessNeed =
  | { kind: "dataset"; base: string; id: string; action: "view" | "use" }
  | { kind: "offer"; base: string };

/** Gets ready `need` at a peer; a pending grant has a presentation for the user to answer. */
export const prepareAccess = async (peerId: string, need: AccessNeed, requested: boolean) => {
  const answer = await customInstance<{
    status: number;
    data: { grantId: string; pending: boolean };
  }>(`/peer-connection/access/${encodePathId(peerId)}`, {
    method: "POST",
    data: { need, requested },
  });
  if (answer.status !== 200) throw new Error("Could not get access ready with this participant.");
  return answer.data;
};

/** Whether this user already holds a token towards the peer that covers `need`. */
export const holdsAccess = async (peerId: string, need: AccessNeed) => {
  const answer = await customInstance<{ status: number; data: string | null }>(
    `/peer-connection/token/${encodePathId(peerId)}/held`,
    { method: "POST", data: { need } },
  );
  return answer.status === 200 && !!answer.data;
};

const OPEN_STATUSES = ["Pending", "Processing"];
/** While a grant of this user is on its way, how often to look again (in case `/live` is down). */
const PENDING_POLL_MS = 2_000;

/** This user's connection with each peer, from the grants they sent. */
export const usePeerConnections = () => {
  const { data: session, isLoading: sessionLoading } = useGetSessionUser();
  const userId = session?.status === 200 ? session.data.userId : undefined;
  // A role above sees its team's grants too: only the user's own count.
  const ownGrants = (data: unknown): SentGrant[] => {
    const page = data as { status: number; data: { items: SentGrant[] } } | undefined;
    return page?.status === 200 ? page.data.items.filter((g) => g.user_id === userId) : [];
  };
  const { data, isLoading, isFetchedAfterMount } = useGetAllPeerConnectionRequests(
    { limit: 100, sort: "created_at_desc" },
    {
      query: {
        refetchInterval: (query) =>
          ownGrants(query.state.data).some((g) => OPEN_STATUSES.includes(g.status))
            ? PENDING_POLL_MS
            : false,
      },
    },
  );

  const byPeer = useMemo(() => {
    const map = new Map<string, PeerConnection>();
    const decided = new Set<string>();
    // Newest first: the first grant seen for a peer is its latest, unless an older one gives access.
    for (const grant of ownGrants(data)) {
      const known = map.get(grant.participant_id);
      if (grant.status === "Approved" && !decided.has(grant.participant_id)) {
        decided.add(grant.participant_id);
        const reach = reachOf(grant.granted ?? []);
        if (reach !== "none" || !known) map.set(grant.participant_id, reach);
      } else if (known) continue;
      else if (OPEN_STATUSES.includes(grant.status)) map.set(grant.participant_id, "pending");
      else map.set(grant.participant_id, "none");
    }
    return map;
  }, [data, userId]);

  return {
    // Cached grants may be stale (a disconnect elsewhere): asking a peer with them would get a
    // grant on its own, so callers wait for the list fetched on mount.
    isLoading: sessionLoading || isLoading || !isFetchedAfterMount,
    statusOf: (participantId: string | undefined): PeerConnection =>
      (participantId && byPeer.get(participantId)) || "none",
    /** This user's latest approved grant with the peer. */
    approvedGrantOf: (participantId: string | undefined): SentGrant | undefined =>
      ownGrants(data).find((g) => g.participant_id === participantId && g.status === "Approved"),
    /** This user's latest grant with the peer still on its way. */
    pendingGrantOf: (participantId: string | undefined): SentGrant | undefined =>
      ownGrants(data).find(
        (g) => g.participant_id === participantId && OPEN_STATUSES.includes(g.status),
      ),
  };
};

/** Whether `granted` covers dataset `id`: by its id or by the datasets of a catalog. */
export const coversDataset = (granted: AccessItem[], id: string) =>
  granted.some(
    (access) =>
      access.type.endsWith("/dataset") &&
      (access.identifier === id || (access.locations ?? []).length > 0),
  );
