import { useContext, useEffect } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { useGetWellKnownDSpaceVersion } from "../data/orval/well-known/well-known";
import { useFetchDataspaceVersionFromParticipant } from "../data/orval/well-known-rp-c/well-known-rp-c";
import { useGetParticipantById, getGetParticipantByIdQueryOptions } from "../data/orval/participants/participants";
import { GlobalInfoContext, GlobalInfoContextType } from "shared/src/context/GlobalInfoContext";

/**
 * Hook to get the full URL for a specific dataspace protocol version path.
 *
 * @param version - The protocol version to look for (default: "2025-1")
 * @returns The full URL (api_gateway_base + path) or null if not found.
 */
export const useMyWellKnownDSPPath = (version = "2025-1"): string | null => {
  const { data: myWellKnown } = useGetWellKnownDSpaceVersion();
  const context = useContext<GlobalInfoContextType | null>(GlobalInfoContext);

  if (!context) {
    console.warn("GlobalInfoContext is null in useWellKnownUrl");
    return null;
  }

  const { api_gateway_base } = context;

  // Check if data is available and is of type VersionResponse (has protocolVersions)
  if (!myWellKnown?.data || !("protocolVersions" in myWellKnown.data)) {
    return null;
  }

  const protocolVersion = myWellKnown.data.protocolVersions.find((p) => p.version === version);

  if (!protocolVersion) {
    return null;
  }

  return `${api_gateway_base}${protocolVersion.path}`;
};

/**
 * Hook to automatically fetch and return the dataspace version path for a specific participant.
 *
 * @param participantId - The ID of the participant (e.g., "urn:...")
 * @param version - The protocol version to look for (default: "2025-1")
 * @returns The full URL or null if still loading / not found.
 */
export const useParticipantDSPPath = (participantId: string | undefined, version = "2025-1") => {
  const queryClient = useQueryClient();
  const { data: participantData } = useGetParticipantById(participantId as string, {
    query: {
        enabled: !!participantId
    }
  });

  const { mutateAsync: fetchVersionAsync, mutate: fetchVersion, data: participantVersion } = useFetchDataspaceVersionFromParticipant();

  useEffect(() => {
    if (participantId) {
      fetchVersion({ data: { participant_id: participantId } });
    }
  }, [participantId, fetchVersion]);

  const resolve = async (id: string, ver = version): Promise<string | null> => {
      // Fetch participant
      const pData = await queryClient.fetchQuery(getGetParticipantByIdQueryOptions(id));
       if (pData.status !== 200 || !pData.data.base_url) return null;

      // Fetch version
      const vData = await fetchVersionAsync({ data: { participant_id: id } });
      if (!vData || !vData.data || !("protocolVersions" in vData.data)) return null;

      const protocolVersion = vData.data.protocolVersions.find((p: any) => p.version === ver);
      return protocolVersion ? `${pData.data.base_url}${protocolVersion.path}` : null;
  }

  // Ensure we have both participant data (for base_url) and version data (for path)
  let path: string | null = null;
  if (participantData && participantData.status === 200 && participantData.data.base_url && participantVersion?.data && ("protocolVersions" in participantVersion.data)) {
      const { base_url } = participantData.data;
      const protocolVersion = participantVersion.data.protocolVersions.find((p) => p.version === version);
      if (protocolVersion) {
          path = `${base_url}${protocolVersion.path}`;
      }
  }

  return { path, resolve };
};
