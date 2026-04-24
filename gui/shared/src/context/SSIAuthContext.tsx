import React, { createContext, ReactNode, useState } from "react";
import { useQueryClient } from "@tanstack/react-query";
import {
  useGetWalletDid,
  useOidc4vciRequest,
  useOidc4vpRequest,
  useOnboardWallet,
  getGetWalletDidQueryKey,
} from "../data/orval/wallet/wallet";
import {
  useGetAllVCRequests,
  useRequestVCtoAuthorityCrossUser,
} from "../data/orval/vc-request/vc-request";
import { VCRequestDto } from "../data/orval/model";
import { useOnboardProvider } from "../data/orval/onboard/onboard";
import { getDidFromUrl } from "../data/orval/did/did";
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
  fetchPeerDid: async (url: string) => {},
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

  // Loading states for manual actions
  const [isFetchingAuthDid, setIsFetchingAuthDid] = useState(false);
  const [isFetchingPeerDid, setIsFetchingPeerDid] = useState(false);
  const [isRequestingVC, setIsRequestingVC] = useState(false);
  const [isFetchingAuthRequests, setIsFetchingAuthRequests] = useState(false);
  const [isRequestingVP, setIsRequestingVP] = useState(false);

  // Data access
  const { mutateAsync: onboardWallet, isPending: isOnboardingWallet } = useOnboardWallet();
  const { data: didResponse } = useGetWalletDid();
  const ownDid = didResponse?.status === 200 ? didResponse.data.id : null;
  const ownWalletOnboarded = didResponse?.status === 200;
  const { mutateAsync: oidc4vciRequest, isPending: isOidc4vciRequesting } = useOidc4vciRequest();
  const { mutateAsync: oidc4vpRequest, isPending: isOidc4vpRequesting } = useOidc4vpRequest();
  const { mutateAsync: requestVCtoAuthorityCrossUser } = useRequestVCtoAuthorityCrossUser();
  const { data: vcRequestsResponse, refetch: refetchAuthRequestsQuery } = useGetAllVCRequests();
  const vcRequests = vcRequestsResponse?.status === 200 ? vcRequestsResponse.data : [];
  const { mutateAsync: onboardProvider } = useOnboardProvider();

  // Actions
  const onboardInWallet = async () => {
    try {
      await onboardWallet();
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
        const requests = data?.status === 200 ? data.data : [];
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
      await requestVCtoAuthorityCrossUser({
        data: {
          url: authDid.url + "/api/v1/gate/access",
          id: authDid.did,
          slug: "authority",
          vc_type: "DataspaceParticipant",
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
      const requests = data?.status === 200 ? data.data : [];
      const requestsSorted = requests.sort(
        (a, b) => new Date(b.created_at || 0).getTime() - new Date(a.created_at || 0).getTime(),
      );
      const latest = requestsSorted.at(0);
      if (!latest) {
        throw new Error("No Auth Request found");
      }
      // @ts-ignore vc_uri is not defined in the type but exists
      setOidc4vciRequestUriState(latest.vc_uri);
    } catch (error) {
      console.error(error);
    }
  };

  const saveOidc4VciRequestUri = async () => {
    setIsRequestingVC(true);
    try {
      const { data } = await refetchAuthRequestsQuery();
      const requests = data?.status === 200 ? data.data : [];
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

      // @ts-ignore vc_uri is not defined in the type but exists
      setOidc4vciRequestUriState(latest.vc_uri);

      await oidc4vciRequest({
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
      if (!tempPeer.did) {
        throw new Error("No Temp Peer DID found");
      }
      const response = await onboardProvider({
        data: {
          url: tempPeer.url + "/api/v1/gate/access",
          id: tempPeer.did,
          slug: "bro",
          actions: "talk",
        },
      });
      if (response.status === 200) {
        // response.data is now guaranteed to be the string content (URI) or a JSON object
        const responseData = response.data as any;
        const uri =
          typeof responseData === "string" ? responseData : responseData?.uri || responseData?.url;

        if (uri) {
          setOidc4vpRequestUriState(uri);
        } else {
          console.warn("Unexpected Onboard Provider response format:", responseData);
        }
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
      if (!oidc4vpRequestUri) {
        throw new Error("No OIDC4VP Request URI found");
      }
      await oidc4vpRequest({
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
