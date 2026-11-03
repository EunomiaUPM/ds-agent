// Connections of the session's user with the dataspace peers.
//
// A peer counts as connected when this user (not someone else of the organization) holds an
// approved grant with it. The federated catalog lists the peers; the grant is asked for from the
// catalog browser, and once it completes the peer is stored as a mate with the chosen visibility.

import { useMemo } from "react";
import { useQuery } from "@tanstack/react-query";
import { customInstance } from "./orval-mutator";
import type { AccessItem } from "./orval/model/accessItem";
import { getDidFromUrl } from "./orval/did/did";
import { useGetSessionUser } from "./orval/gateway/gateway";
import { useGetAllPeerConnectionRequests } from "./orval/onboard/onboard";
import type { DidService } from "./orval/model/didService";
import type { SentGrant } from "./orval/model/sentGrant";

/** Where this user stands with a peer: an approved grant, one on its way, or none (a denied
 * grant is finalized, RFC 9635 §1.5). */
export type PeerConnection = "approved" | "pending" | "none";

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
    // Newest first: the first grant seen for a peer is its latest, unless an older one is approved.
    for (const grant of ownGrants(data)) {
      const known = map.get(grant.participant_id);
      if (grant.status === "Approved") map.set(grant.participant_id, "approved");
      else if (known) continue;
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

/** What this user was granted by the peer of approved grant `grantId`. */
export const useGrantedAccess = (grantId: string | undefined) => {
  const { data } = useQuery({
    queryKey: ["/peer-connection/request/details", grantId],
    queryFn: () =>
      customInstance<{
        status: number;
        data: { resource_req?: { granted?: AccessItem[] | null } };
      }>(`/peer-connection/request/${encodeURIComponent(grantId!)}/details`, { method: "GET" }),
    enabled: !!grantId,
  });
  return data?.status === 200 ? (data.data.resource_req?.granted ?? []) : undefined;
};

/** Whether `granted` covers dataset `id`: by its id or by the datasets of a catalog. */
export const coversDataset = (granted: AccessItem[], id: string) =>
  granted.some(
    (access) =>
      access.type.endsWith("/dataset") &&
      (access.identifier === id || (access.locations ?? []).length > 0),
  );
