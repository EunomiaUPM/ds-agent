import React, { createContext, ReactNode, useState } from "react";
import { useQueryClient } from "@tanstack/react-query";
import {
  useGetWalletDid,
  useLinkWallet,
  getGetWalletDidQueryKey,
} from "../data/orval/wallet/wallet";
import {
  useBegVc,
  useGetAllVCRequests,
  useProcessVcRequestOid4vci,
} from "../data/orval/vc-request/vc-request";
import { VCRequestDto } from "../data/orval/model";
import {
  getAllPeerConnectionRequests,
  getPeerConnectionRequestDetails,
  useConnectToPeer,
  useProcessPeerConnectionOid4vp,
} from "../data/orval/onboard/onboard";
import { getDidFromUrl } from "../data/orval/did/did";

// Credential type requested from the authority in the guided flow.
const DEMO_VC_TYPE = "DataSpaceParticipant_jwt_vc_json";
import { useEffect } from "react";

export interface SSIAuthContextType {
  // Is Wallet onboarded
  ownWalletOnboarded: boolean;
  // Own DID
  ownDid: string | null;
  // Temp Peer
  tempPeer: {
    url: string | null;
    did: string | null;
    didDocument: Object | null;
  };
  // Auth DID
  authDid: {
    url: string | null;
    did: string | null;
    didDocument: Object | null;
  };
  // Auth Requests
  authRequests: VCRequestDto[];
  authRequestsPollInterval: number;
  authRequestsLastPoll: Date;
  currentAuthRequestId: string | null;
  // OIDC4VP Request
  oidc4vpRequestUri: string | null;
  oidc4vciRequestUri: string | null;

  // Success
  oidc4vpSuccess: boolean;

  // Loading states
  isLoading: {
    onboard: boolean;
    fetchAuthDid: boolean;
    fetchPeerDid: boolean;
    requestVC: boolean;
    fetchAuthRequests: boolean;
    oidc4vp: boolean;
    oidc4vci: boolean;
  };

  // Actions
  onboardInWallet: () => Promise<void>;
  fetchAuthDid: (url: string) => Promise<void>;
  fetchPeerDid: (url: string) => Promise<void>;
  requestVCtoAuthority: () => Promise<void>;
  fetchAuthRequests: () => Promise<void>;
  pollAuthRequests: () => Promise<void>;
  setOidc4VciRequestUri: () => void;
  saveOidc4VciRequestUri: () => void;
  setOidc4VpRequestUri: () => void;
  presentVPtoPeer: () => void;
}

export const SSIAuthContext = createContext<SSIAuthContextType>({
  ownWalletOnboarded: false,
  ownDid: null,
  tempPeer: {
    url: null,
    did: null,
    didDocument: null,
  },
  authDid: {
    url: null,
    did: null,
    didDocument: null,
  },
  authRequests: [],
  authRequestsPollInterval: 0,
  authRequestsLastPoll: new Date(),
  currentAuthRequestId: null,
  oidc4vciRequestUri: null,
  oidc4vpRequestUri: null,
  oidc4vpSuccess: false,
  isLoading: {
    onboard: false,
    fetchAuthDid: false,
    fetchPeerDid: false,
    requestVC: false,
    fetchAuthRequests: false,
    oidc4vp: false,
    oidc4vci: false,
  },
  onboardInWallet: async () => {},
  fetchAuthDid: async (url: string) => {},
  fetchPeerDid: async (_url: string) => {},
  requestVCtoAuthority: async () => {},
  fetchAuthRequests: async () => {},
  pollAuthRequests: async () => {},
  setOidc4VciRequestUri: async () => {},
  saveOidc4VciRequestUri: () => {},
  setOidc4VpRequestUri: () => {},
  presentVPtoPeer: () => {},
});

export const SSIAuthContextProvider = ({ children }: { children: ReactNode }) => {
  const queryClient = useQueryClient();
  // State
  const [isContextWorking, setIsContextWorking] = useState<boolean>(false);
  const [tempPeer, setTempPeer] = useState<SSIAuthContextType["tempPeer"]>({
    url: null,
    did: null,
    didDocument: null,
  });
  const [authDid, setAuthDid] = useState<SSIAuthContextType["authDid"]>({
    url: null,
    did: null,
    didDocument: null,
  });
  const [authRequests, setAuthRequests] = useState<VCRequestDto[]>([]);
  const [authRequestsPollInterval, setAuthRequestsPollInterval] = useState<number>(0);
  const [authRequestsLastPoll, setAuthRequestsLastPoll] = useState<Date>(new Date());
  const [currentAuthRequestId, setCurrentAuthRequestId] = useState<string | null>(null);
  const [oidc4vpRequestUri, setOidc4vpRequestUriState] = useState<string | null>(null);
  const [oidc4vciRequestUri, setOidc4vciRequestUriState] = useState<string | null>(null);
  const [oidc4vpSuccess, setOidc4vpSuccess] = useState<boolean>(false);
  const [peerRequestId, setPeerRequestId] = useState<string | null>(null);

  // Loading states for manual actions
  const [isFetchingAuthDid, setIsFetchingAuthDid] = useState(false);
  const [isFetchingPeerDid, setIsFetchingPeerDid] = useState(false);
  const [isRequestingVC, setIsRequestingVC] = useState(false);
  const [isFetchingAuthRequests, setIsFetchingAuthRequests] = useState(false);
  const [isRequestingVP, setIsRequestingVP] = useState(false);

  // Data access
  const { mutateAsync: linkWallet, isPending: isOnboardingWallet } = useLinkWallet();
  const { data: didResponse } = useGetWalletDid();
  const ownDid = didResponse?.status === 200 ? didResponse.data.id : null;
  const ownWalletOnboarded = didResponse?.status === 200;
  const { mutateAsync: processVcRequestOid4vci } = useProcessVcRequestOid4vci();
  const { mutateAsync: processPeerConnectionOid4vp } = useProcessPeerConnectionOid4vp();
  const { mutateAsync: begVc } = useBegVc();
  const { data: vcRequestsResponse, refetch: refetchAuthRequestsQuery } = useGetAllVCRequests({
    sort: "created_at_desc",
  });
  const vcRequests = vcRequestsResponse?.status === 200 ? vcRequestsResponse.data.items : [];
  const { mutateAsync: connectToPeer } = useConnectToPeer();

  // Actions
  const onboardInWallet = async () => {
    try {
      await linkWallet();
      await queryClient.invalidateQueries({ queryKey: getGetWalletDidQueryKey() });
    } catch (error) {
      console.error(error);
    }
  };

  const fetchAuthDid = async (url: string) => {
    setIsFetchingAuthDid(true);
    try {
      const cleanUrl = url.replace(/\/$/, "");
      const response = await getDidFromUrl(encodeURIComponent(cleanUrl));

      if (response.status === 200) {
        setAuthDid({
          url: cleanUrl,
          did: response.data.id,
          didDocument: response.data,
        });
      }
    } catch (error) {
      console.error(error);
    } finally {
      setIsFetchingAuthDid(false);
    }
  };

  const fetchPeerDid = async (url: string) => {
    setIsFetchingPeerDid(true);
    try {
      const cleanUrl = url.replace(/\/$/, "");
      const response = await getDidFromUrl(encodeURIComponent(cleanUrl));
      if (response.status === 200) {
        setTempPeer({
          url: cleanUrl,
          did: response.data.id,
          didDocument: response.data,
        });
      }
    } catch (error) {
      console.error(error);
    } finally {
      setIsFetchingPeerDid(false);
    }
  };

  useEffect(() => {
    if (authRequestsPollInterval === 0) return;
    const interval = setInterval(async () => {
      try {
        const { data } = await refetchAuthRequestsQuery();
        const requests = data?.status === 200 ? data.data.items : [];
        if (requests.length === 0) return;

        // Sort safely by creating a copy
        const requestsSorted = [...requests].sort(
          (a, b) => new Date(b.created_at || 0).getTime() - new Date(a.created_at || 0).getTime(),
        );
        const latest = requestsSorted[0];

        setAuthRequestsLastPoll(new Date());

        if (latest && latest.status === "Approved") {
          setAuthRequestsPollInterval(0);
          if (latest.vc_uri) {
            setOidc4vciRequestUriState(latest.vc_uri);
          }
        }
      } catch (error) {
        console.error("Error polling requests:", error);
      }
    }, authRequestsPollInterval);
    return () => clearInterval(interval);
  }, [authRequestsPollInterval, refetchAuthRequestsQuery]);

  const requestVCtoAuthority = async () => {
    setIsRequestingVC(true);
    try {
      if (!authDid.url || !ownDid || !authDid.did) {
        throw new Error("Missing Authority URL or Own DID");
      }
      await begVc({
        data: {
          id: authDid.did,
          nick: "authority",
          url: authDid.url,
          vc_type: DEMO_VC_TYPE,
          method: "oid4vp",
          auto: false,
        },
      });
      setAuthRequestsPollInterval(500);
    } catch (error) {
      console.error(error);
    } finally {
      setIsRequestingVC(false);
    }
  };

  const fetchAuthRequests = async () => {
    setIsFetchingAuthRequests(true);
    try {
      await refetchAuthRequestsQuery();
    } catch (error) {
      console.error(error);
    } finally {
      setIsFetchingAuthRequests(false);
    }
  };

  const pollAuthRequests = async () => {
    await fetchAuthRequests();
    setAuthRequestsPollInterval(500);
  };

  const setOidc4VciRequestUri = async () => {
    try {
      if (!authDid.url || !ownDid) {
        throw new Error("Missing Authority URL or Own DID");
      }
      const { data } = await refetchAuthRequestsQuery();
      const requests = data?.status === 200 ? data.data.items : [];
      const requestsSorted = [...requests].sort(
        (a, b) => new Date(b.created_at || 0).getTime() - new Date(a.created_at || 0).getTime(),
      );
      const latest = requestsSorted.at(0);
      if (!latest) {
        throw new Error("No Auth Request found");
      }
      setOidc4vciRequestUriState(latest.vc_uri ?? null);
    } catch (error) {
      console.error(error);
    }
  };

  const saveOidc4VciRequestUri = async () => {
    setIsRequestingVC(true);
    try {
      const { data } = await refetchAuthRequestsQuery();
      const requests = data?.status === 200 ? data.data.items : [];
      if (requests.length === 0) throw new Error("No requests found");

      // Sort safely by creating a copy
      const requestsSorted = [...requests].sort(
        (a, b) => new Date(b.created_at || 0).getTime() - new Date(a.created_at || 0).getTime(),
      );
      const latest = requestsSorted[0];

      if (!latest) {
        throw new Error("No Auth Request found");
      }

      if (latest.status !== "Approved") {
        throw new Error("Latest request is not approved yet");
      }

      if (!latest.vc_uri) {
        throw new Error("No VC URI found in approved request");
      }

      setOidc4vciRequestUriState(latest.vc_uri);

      await processVcRequestOid4vci({
        id: latest.id,
        data: {
          uri: latest.vc_uri,
        },
      });
    } catch (error) {
      console.error(error);
    } finally {
      setIsRequestingVC(false);
    }
  };

  const setOidc4VpRequestUri = async () => {
    setIsFetchingAuthRequests(true);
    try {
      if (!tempPeer.url || !tempPeer.did) {
        throw new Error("No Temp Peer URL or DID found");
      }
      await connectToPeer({
        data: {
          id: tempPeer.did,
          nick: "peer",
          url: `${tempPeer.url}/api/v1/gate/access`,
          actions: ["talk"],
          auto: false,
        },
      });
      // The connect call answers no id: take the newest request sent to this peer.
      const requests = await getAllPeerConnectionRequests({
        participantId: tempPeer.did,
        sort: "created_at_desc",
        limit: 1,
      });
      const latest = requests.status === 200 ? requests.data.items[0] : undefined;
      if (!latest) {
        throw new Error("No connection request found for the peer");
      }
      setPeerRequestId(latest.id);
      const details = await getPeerConnectionRequestDetails(latest.id);
      const uri =
        details.status === 200
          ? ((details.data as { verification?: { uri?: string } }).verification?.uri ?? null)
          : null;
      if (uri) {
        setOidc4vpRequestUriState(uri);
      } else {
        console.warn("Connection request has no OID4VP URI yet:", latest.id);
      }
    } catch (error) {
      console.error(error);
    } finally {
      setIsFetchingAuthRequests(false);
    }
  };

  const presentVPtoPeer = async () => {
    setIsRequestingVP(true);
    try {
      if (!oidc4vpRequestUri || !peerRequestId) {
        throw new Error("No OIDC4VP Request URI found");
      }
      await processPeerConnectionOid4vp({
        id: peerRequestId,
        data: {
          uri: oidc4vpRequestUri,
        },
      });
      setOidc4vpSuccess(true);
    } catch (error) {
      console.error(error);
    } finally {
      setIsRequestingVP(false);
    }
  };

  return (
    <SSIAuthContext.Provider
      value={{
        ownWalletOnboarded,
        ownDid,
        tempPeer,
        authDid,
        authRequests: vcRequests,
        authRequestsPollInterval,
        authRequestsLastPoll,
        currentAuthRequestId,
        oidc4vpRequestUri,
        oidc4vciRequestUri,
        oidc4vpSuccess,
        isLoading: {
          onboard: isOnboardingWallet,
          fetchAuthDid: isFetchingAuthDid,
          fetchPeerDid: isFetchingPeerDid,
          requestVC: isRequestingVC,
          fetchAuthRequests: isFetchingAuthRequests,
          oidc4vp: isRequestingVP,
          oidc4vci: isRequestingVC,
        },
        onboardInWallet,
        fetchAuthDid,
        fetchPeerDid,
        requestVCtoAuthority,
        fetchAuthRequests,
        pollAuthRequests,
        setOidc4VciRequestUri,
        saveOidc4VciRequestUri,
        setOidc4VpRequestUri,
        presentVPtoPeer,
      }}
    >
      {children}
    </SSIAuthContext.Provider>
  );
};
