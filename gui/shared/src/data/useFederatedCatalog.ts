import { useQuery } from "@tanstack/react-query";
import { useGetAllParticipants } from "shared/src/data/orval/participants/participants";
import { customInstance } from "shared/src/data/orval-mutator";

export interface FederatedParticipant {
  participant_id: string;
  participant_slug: string;
  participant_type: string;
  base_url: string;
  is_me?: boolean;
  vc_uri?: string | null;
  is_vc_issued?: boolean;
  saved_at?: string;
  last_interaction?: string;
}

type FederatedCatalogResult =
  | { state: "loading" }
  | { state: "no-authority" }
  | { state: "error"; error: unknown }
  | { state: "ok"; agents: FederatedParticipant[]; myParticipantId: string | undefined };

const fetchFederatedCatalog = async (
  authorityBaseUrl: string,
): Promise<FederatedParticipant[]> => {
  const encoded = encodeURIComponent(authorityBaseUrl);
  const res = await customInstance<{
    status: number;
    data: FederatedParticipant[];
  }>(`/federated-catalog/${encoded}`, { method: "GET" });
  if (res.status !== 200 || !Array.isArray(res.data)) {
    throw new Error(`Federated catalog fetch failed (status ${res.status})`);
  }
  return res.data;
};

export const useFederatedCatalog = (): FederatedCatalogResult => {
  const { data: participantsResponse, isLoading: participantsLoading } =
    useGetAllParticipants();

  const localParticipants =
    participantsResponse?.status === 200 ? participantsResponse.data : [];

  const authority = localParticipants.find((p) => p.participant_type === "Authority");
  const myParticipantId = localParticipants.find((p) => p.is_me)?.participant_id;

  const {
    data: federated,
    isLoading: federatedLoading,
    error: federatedError,
  } = useQuery({
    queryKey: ["federated-catalog", authority?.base_url],
    queryFn: () => fetchFederatedCatalog(authority!.base_url!),
    enabled: !!authority?.base_url,
  });

  if (participantsLoading) return { state: "loading" };
  if (!authority?.base_url) return { state: "no-authority" };
  if (federatedLoading) return { state: "loading" };
  if (federatedError) return { state: "error", error: federatedError };
  if (!federated) return { state: "loading" };

  const agents = federated.filter(
    (p) => p.participant_type === "Agent" && p.participant_id !== myParticipantId,
  );

  return { state: "ok", agents, myParticipantId };
};
