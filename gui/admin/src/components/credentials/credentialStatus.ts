// TODO: move to @eunomia/ui (ymir/ui). Duplicated in heimdall/react and ds-agent/gui/admin under
// src/components/credentials: keep both copies identical until then.
import { type UseQueryResult, useQueries } from "@tanstack/react-query";
import { decodeJwtPayload } from "./CredentialView";

/** How long a downloaded status list counts as fresh before it is checked again. */
const STALE_MS = 30_000;

/** One `credentialStatus` entry of a credential: which list, and where in it. */
export interface StatusEntry {
  purpose: string;
  url: string;
  index: number;
}

export type EntryState = "checking" | "active" | "revoked" | "suspended" | "unknown";

/** The status of a credential as a page knows it, ready to show. */
export interface CredentialStatus {
  entries: Array<StatusEntry & { state: EntryState }>;
  /** Where the states come from, such as the lists the issuer publishes. */
  source: string;
  /** A check is running, maybe with the previous result still on screen. */
  refreshing?: boolean;
  /** Checks again, past what is kept in memory. */
  onRecheck?: () => void;
}

export type OverallState =
  | "Checking"
  | "Active"
  | "Suspended"
  | "Revoked"
  | "Unknown"
  | "No status";

/** One word for the whole credential: revoked or suspended wins over the rest. */
export function overallState(status?: CredentialStatus): OverallState {
  const states = status?.entries.map((entry) => entry.state) ?? [];
  if (states.length === 0) return "No status";
  if (states.includes("revoked")) return "Revoked";
  if (states.includes("suspended")) return "Suspended";
  if (states.includes("checking")) return "Checking";
  if (states.includes("unknown")) return "Unknown";
  return "Active";
}

/** The revocation and suspension entries of a credential document or JWT payload. */
export function statusEntries(document: unknown): StatusEntry[] {
  const payload = asDoc(document);
  const vc = asDoc(payload.vc);
  const raw = (Object.keys(vc).length > 0 ? vc : payload).credentialStatus;
  return (Array.isArray(raw) ? raw : raw ? [raw] : []).map(asDoc).flatMap((entry) => {
    const purpose = entry.statusPurpose;
    const url = entry.statusListCredential;
    const index = Number(entry.statusListIndex);
    const known = purpose === "revocation" || purpose === "suspension";
    return known && typeof url === "string" && Number.isInteger(index) && index >= 0
      ? [{ purpose, url, index }]
      : [];
  });
}

/** A downloaded status list: its purpose and its bits. */
interface StatusList {
  purpose: string;
  bits: Uint8Array;
}

/**
 * Downloads each list at `urls` once, keeps it in memory, and tells the state of any entries
 * from it. Many credentials share a list, so this is one request per list, not per credential.
 */
export function useStatusLists(urls: string[]) {
  const unique = [...new Set(urls)];
  const queries = useQueries({
    queries: unique.map((url) => ({
      queryKey: ["status-list", url],
      queryFn: () => fetchStatusList(url),
      staleTime: STALE_MS,
      retry: false,
    })),
  });
  const byUrl = new Map(unique.map((url, i) => [url, queries[i]]));

  const statusOf = (entries: StatusEntry[], source: string): CredentialStatus => {
    const lists = [...new Set(entries.map((entry) => entry.url))];
    return {
      source,
      refreshing: lists.some((url) => byUrl.get(url)?.isFetching),
      onRecheck: () => lists.forEach((url) => void byUrl.get(url)?.refetch()),
      entries: entries.map((entry) => ({
        ...entry,
        state: entryState(entry, byUrl.get(entry.url)),
      })),
    };
  };
  return { statusOf };
}

function entryState(entry: StatusEntry, query?: UseQueryResult<StatusList>): EntryState {
  if (!query || query.isPending) return "checking";
  const list = query.data;
  const byte = entry.index >> 3;
  if (!list || list.purpose !== entry.purpose || byte >= list.bits.length) return "unknown";
  if ((list.bits[byte] & (0x80 >> (entry.index & 7))) === 0) return "active";
  return entry.purpose === "revocation" ? "revoked" : "suspended";
}

/**
 * Downloads and opens a status list. Its signature is not checked: the state is only shown
 * here, and verifiers check the list for real.
 */
async function fetchStatusList(url: string): Promise<StatusList> {
  const res = await fetch(url);
  if (!res.ok) throw new Error(`The status list answered HTTP ${res.status}`);
  const payload = asDoc(decodeJwtPayload((await res.text()).trim()));
  const vc = asDoc(payload.vc);
  const subject = asDoc((Object.keys(vc).length > 0 ? vc : payload).credentialSubject);
  const encoded = subject.encodedList;
  if (typeof encoded !== "string" || !encoded.startsWith("u")) {
    throw new Error("Not a Bitstring Status List");
  }
  return {
    purpose: String(subject.statusPurpose ?? ""),
    bits: await gunzip(base64UrlBytes(encoded.slice(1))),
  };
}

function base64UrlBytes(text: string): Uint8Array {
  const base64 = text.replace(/-/g, "+").replace(/_/g, "/");
  const padded = base64 + "=".repeat((4 - (base64.length % 4)) % 4);
  return Uint8Array.from(atob(padded), (c) => c.charCodeAt(0));
}

async function gunzip(bytes: Uint8Array): Promise<Uint8Array> {
  const stream = new Blob([bytes]).stream().pipeThrough(new DecompressionStream("gzip"));
  return new Uint8Array(await new Response(stream).arrayBuffer());
}

const asDoc = (value: unknown): Record<string, unknown> =>
  value && typeof value === "object" && !Array.isArray(value)
    ? (value as Record<string, unknown>)
    : {};
