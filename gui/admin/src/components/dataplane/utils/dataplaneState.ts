import type { DataplaneTransferLogDto } from "shared/src/data/orval/model";

const SECRET_KEY_RE =
  /pass|secret|token|authorization|credential|api_?key|access_token|refresh_token/i;

const FAILURE_REASON_INDICATORS = [
  "[code=",
  "failed",
  "error",
  "denied",
  "blocked",
  "rejected",
  "exceeded",
  "forbidden",
  "unreachable",
  "timeout",
  "mismatch",
  "missing",
  "not found",
  "invalid",
  "exhausted",
  "terminated locally",
  "forced local termination",
];

const REASON_CODE_RE = /\[code=([^\]\s]+)\]/;

export interface ParsedReason {
  code?: string;
  message: string;
}

export interface ParsedAdapterConfig {
  kind: string;
  endpoint?: string;
  tokenType?: string;
  token?: string;
  extra: Record<string, unknown>;
}

export interface FlowControlSummary {
  lifetime?: string;
  maxRetries?: number;
  authType?: string;
  authDetails?: Record<string, unknown>;
  hasSubscription: boolean;
  hasUnsubscription: boolean;
  isVaultedAuth: boolean;
}

export interface ParsedEventData {
  method?: string;
  status?: number;
  reason?: string;
  target?: string;
  originPath?: string;
  role?: string;
  mode?: string;
  ingress?: string;
  egress?: string;
  kafka?: { topic?: string; partition?: unknown; offset?: unknown };
  rest: Array<[string, unknown]>;
}

/** Maps dataplane state values (both PascalCase and UPPERCASE) to @eunomia/ui badge states. */
export function dataplaneStateVariant(state?: string | null): string {
  switch (state?.toLowerCase()) {
    case "init":
      return "default";
    case "configuring":
    case "auth":
    case "subscribing":
      return "ACTIVE";
    case "ready":
      return "AGREED";
    case "started":
      return "STARTED";
    case "unsubscribing":
      return "SUSPENDED";
    case "stopped":
      return "STOPPED";
    case "terminated":
    case "error":
      return "TERMINATED";
    default:
      return "default";
  }
}

/** Normalizes a dataplane role string into a UI label compatible with Badge role variants. */
export function roleLabel(role?: string): "Provider" | "Consumer" {
  return role?.toUpperCase() === "CONSUMER" ? "Consumer" : "Provider";
}

/** Formats a millisecond delta concisely for transition logs and pipeline timings. */
export function formatDurationMs(ms: number): string {
  if (!Number.isFinite(ms) || ms <= 0) return "0 ms";
  if (ms < 1_000) return `${Math.round(ms)} ms`;
  if (ms < 60_000) return `${(ms / 1_000).toFixed(ms < 10_000 ? 2 : 1)} s`;
  if (ms < 3_600_000) {
    const mins = Math.floor(ms / 60_000);
    const secs = Math.round((ms % 60_000) / 1_000);
    return secs > 0 ? `${mins}m ${secs}s` : `${mins}m`;
  }
  const hours = Math.floor(ms / 3_600_000);
  const mins = Math.floor((ms % 3_600_000) / 60_000);
  return mins > 0 ? `${hours}h ${mins}m` : `${hours}h`;
}

/** Extracts an embedded `[code=XYZ]` tag from a dataplane log reason string. */
export function parseReason(reason?: string | null): ParsedReason {
  if (!reason) return { message: "" };
  const match = REASON_CODE_RE.exec(reason);
  if (!match) return { message: reason };
  const code = match[1];
  const cleaned = reason
    .replace(/\s*\[code=[^\]\s]+\]\s*(:\s*)?/, (_, colon?: string) => (colon ? ": " : " "))
    .replace(/\s{2,}/g, " ")
    .trim();
  return { code, message: cleaned || reason };
}

/** Checks whether a reason or trigger string describes a failure/error condition. */
export function isFailureReason(text?: string | null): boolean {
  if (!text) return false;
  const lower = text.toLowerCase();
  return FAILURE_REASON_INDICATORS.some((indicator) => lower.includes(indicator));
}

/** Determines whether a dataplane transition log represents an abnormal termination or error. */
export function isFailureLog(
  log: Pick<DataplaneTransferLogDto, "new_state" | "trigger" | "reason">,
): boolean {
  const nextState = log.new_state?.toLowerCase();
  if (nextState === "error") return true;
  if (nextState === "terminated") {
    return isFailureReason(log.reason) || isFailureReason(log.trigger);
  }
  return isFailureReason(log.reason);
}

function asRecord(value: unknown): Record<string, unknown> | null {
  return value && typeof value === "object" && !Array.isArray(value)
    ? (value as Record<string, unknown>)
    : null;
}

/** Parses Rust serde-serialized ingress/egress configs into a structured adapter summary. */
export function parseAdapterConfig(raw: unknown): ParsedAdapterConfig {
  if (raw == null) return { kind: "None", extra: {} };
  if (typeof raw === "string") return { kind: raw, extra: {} };

  const obj = asRecord(raw);
  if (!obj) return { kind: String(raw), extra: {} };

  const keys = Object.keys(obj);
  let kind = "Custom";
  let payload: Record<string, unknown> = obj;

  if (keys.length === 1) {
    const inner = obj[keys[0]];
    if (inner === null) {
      return { kind: keys[0], extra: {} };
    }
    const innerRec = asRecord(inner);
    if (innerRec) {
      kind = keys[0];
      payload = innerRec;
    }
  } else if (typeof obj.type === "string") {
    kind = obj.type;
  } else if (typeof obj.protocol === "string") {
    kind = obj.protocol;
  }

  const endpoint =
    typeof payload.path === "string"
      ? payload.path
      : typeof payload.endpoint === "string"
        ? payload.endpoint
        : typeof payload.url === "string"
          ? payload.url
          : undefined;
  const tokenType =
    typeof payload.token_type === "string"
      ? payload.token_type
      : typeof payload.authorization_type === "string"
        ? payload.authorization_type
        : undefined;
  const token =
    typeof payload.token === "string"
      ? payload.token
      : typeof payload.authorization === "string"
        ? payload.authorization
        : undefined;

  const extra: Record<string, unknown> = {};
  for (const [k, v] of Object.entries(payload)) {
    if (
      k === "path" ||
      k === "endpoint" ||
      k === "url" ||
      k === "token_type" ||
      k === "authorization_type" ||
      k === "token" ||
      k === "authorization"
    ) {
      continue;
    }
    extra[k] = v;
  }

  return { kind, endpoint, tokenType, token, extra };
}

function isVaultPlaceholder(value: string): boolean {
  return value.includes("{{__") || value === "[REDACTED]";
}

/** Recursively masks secret fields while preserving vault placeholders and non-secret metadata. */
export function maskSensitiveValues(
  value: unknown,
  reveal: boolean,
  parentKey = "",
): { masked: unknown; hasSecrets: boolean } {
  if (value == null) return { masked: value, hasSecrets: false };
  if (typeof value === "string") {
    if (!isVaultPlaceholder(value) && SECRET_KEY_RE.test(parentKey) && parentKey !== "token_type") {
      return {
        masked: reveal ? value : "••••••••",
        hasSecrets: true,
      };
    }
    return { masked: value, hasSecrets: false };
  }
  if (Array.isArray(value)) {
    let anySecret = false;
    const mapped = value.map((item) => {
      const res = maskSensitiveValues(item, reveal, parentKey);
      if (res.hasSecrets) anySecret = true;
      return res.masked;
    });
    return { masked: mapped, hasSecrets: anySecret };
  }
  const rec = asRecord(value);
  if (!rec) return { masked: value, hasSecrets: false };

  let anySecret = false;
  const out: Record<string, unknown> = {};
  for (const [k, v] of Object.entries(rec)) {
    const res = maskSensitiveValues(v, reveal, k);
    if (res.hasSecrets) anySecret = true;
    out[k] = res.masked;
  }
  return { masked: out, hasSecrets: anySecret };
}

function isNonEmptyValue(v: unknown): boolean {
  if (v == null) return false;
  if (typeof v === "string") return v.trim().length > 0;
  if (Array.isArray(v)) return v.length > 0;
  const rec = asRecord(v);
  return rec ? Object.keys(rec).length > 0 : true;
}

/** Extracts high-level runtime delivery policy and auth facts from `dp.flowControl`. */
export function extractFlowControlSummary(flowControl: unknown): FlowControlSummary {
  const rec = asRecord(flowControl);
  if (!rec) {
    return { hasSubscription: false, hasUnsubscription: false, isVaultedAuth: false };
  }

  const policy = asRecord(rec.delivery_policy ?? rec.deliveryPolicy);
  const lifetime = typeof policy?.lifetime === "string" ? policy.lifetime : undefined;
  const rawRetries = policy?.max_retries ?? policy?.maxRetries;
  const maxRetries = typeof rawRetries === "number" ? rawRetries : undefined;

  const authRec = asRecord(rec.auth);
  const authType = typeof authRec?.type === "string" ? authRec.type : undefined;
  const authJson = authRec ? JSON.stringify(authRec) : "";
  const isVaultedAuth = authJson.includes("{{__RUNTIME_SECRET_");

  return {
    lifetime,
    maxRetries,
    authType,
    authDetails: authRec ?? undefined,
    hasSubscription: isNonEmptyValue(rec.subscription),
    hasUnsubscription: isNonEmptyValue(rec.unsubscription),
    isVaultedAuth,
  };
}

/** Formats arbitrary event field values safely without crashing on nested objects or nulls. */
export function formatEventFieldValue(val: unknown): string {
  if (val == null) return "—";
  if (typeof val === "object") return JSON.stringify(val);
  return String(val);
}

/** Splits known diagnostic keys from `TransferEventDto.data` for scannable rendering. */
export function parseTransferEventData(raw: unknown): ParsedEventData {
  const rec = asRecord(raw);
  if (!rec) return { rest: [] };

  const consumed = new Set<string>();
  const takeStr = (key: string): string | undefined => {
    const val = rec[key];
    if (typeof val === "string" && val.length > 0) {
      consumed.add(key);
      return val;
    }
    if (val === null) consumed.add(key);
    return undefined;
  };

  let status: number | undefined;
  if (typeof rec.status === "number") {
    status = rec.status;
    consumed.add("status");
  } else if (typeof rec.status === "string" && rec.status.trim() !== "") {
    const n = Number(rec.status);
    if (!Number.isNaN(n)) {
      status = n;
      consumed.add("status");
    }
  }

  const method = takeStr("method");
  const reason = takeStr("reason");
  const target = takeStr("target");
  const originPath = takeStr("origin_path");
  const role = takeStr("role");
  const mode = takeStr("mode");
  const ingress = takeStr("ingress");
  const egress = takeStr("egress");

  let kafka: ParsedEventData["kafka"];
  if ("topic" in rec || "partition" in rec || "offset" in rec) {
    kafka = {
      topic: typeof rec.topic === "string" ? rec.topic : undefined,
      partition: rec.partition,
      offset: rec.offset,
    };
    consumed.add("topic");
    consumed.add("partition");
    consumed.add("offset");
  }

  const rest = Object.entries(rec).filter(([k, v]) => !consumed.has(k) && v !== null);
  return { method, status, reason, target, originPath, role, mode, ingress, egress, kafka, rest };
}
