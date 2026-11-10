import { toast } from "sonner";

/** What an agent endpoint answered, as `customInstance` returns it. */
export interface AgentResponse {
  status: number;
  data: unknown;
}

/** Why an agent action failed, if it did: an HTTP error, or a presentation the verifier rejected. */
export function outcomeFailure(res: AgentResponse): string | null {
  const data = (res.data ?? {}) as {
    status?: string;
    reason?: string | null;
    details?: string | null;
    message?: string | null;
  };
  if (res.status < 200 || res.status >= 300) {
    return data.details || data.message || `The request failed (HTTP ${res.status})`;
  }
  if (data.status === "Failed") {
    return `Presentation failed: ${data.reason || "the verifier gave no reason"}`;
  }
  return null;
}

/** Toasts how an agent action went; whether it succeeded. */
export function reportOutcome(res: AgentResponse): boolean {
  const failure = outcomeFailure(res);
  if (failure) {
    toast.error(failure);
    return false;
  }
  if ((res.data as { status?: string } | null)?.status === "Verified") {
    toast.success("Presentation verified");
  }
  return true;
}
