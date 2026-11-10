// TODO: move to @eunomia/ui (ymir/ui). Duplicated in heimdall/react and ds-agent/gui/admin under
// src/components/credentials: keep both copies identical until then.
import React from "react";
import {
  Braces,
  Building2,
  CalendarClock,
  Fingerprint,
  ListChecks,
  Loader2,
  RefreshCw,
  UserRound,
} from "lucide-react";
import {
  Badge,
  Button,
  FormatDate,
  Identifier,
  JsonTable,
  cn,
  getFriendlyVCType,
} from "@eunomia/ui";
import { type CredentialStatus, type EntryState, overallState } from "./credentialStatus";

type Doc = Record<string, unknown>;

export interface CredentialViewProps {
  /** The signed credential, as a JWT. */
  jwt?: string | null;
  /** The credential document, or a JWT payload, when there is no JWT. */
  document?: unknown;
  /** The state of its status lists, from wherever the page knows it. */
  status?: CredentialStatus;
  className?: string;
}

/** What a verifiable credential says: its types, issuer, holder, validity, status and claims. */
export function CredentialView({ jwt, document, status, className }: CredentialViewProps) {
  const payload = jwt ? decodeJwtPayload(jwt) : document;
  const doc = credentialDocument(payload);
  if (!doc) {
    return <p className="text-sm text-muted-foreground">No credential to show.</p>;
  }

  const types = asStrings(doc.type).filter((t) => t !== "VerifiableCredential");
  const issuer = doc.issuer;
  const issuerId = typeof issuer === "string" ? issuer : asString(asDoc(issuer).id);
  const issuerName = asString(asDoc(issuer).name);
  const { id: holder, ...claims } = asDoc(doc.credentialSubject);
  const validFrom = asString(doc.validFrom) ?? asString(doc.issuanceDate);
  const validUntil = asString(doc.validUntil) ?? asString(doc.expirationDate);
  const statuses = asList(doc.credentialStatus).map(asDoc);

  return (
    <div className={cn("space-y-6", className)}>
      {types.length > 0 && (
        <div className="flex flex-wrap gap-1.5">
          {types.map((t) => (
            <Badge key={t} variant="info">
              {getFriendlyVCType(t)}
            </Badge>
          ))}
        </div>
      )}

      <dl className="grid grid-cols-1 gap-x-6 gap-y-4 sm:grid-cols-2">
        <Field icon={Building2} label="Issuer">
          {issuerName && <span className="mr-2 font-medium">{issuerName}</span>}
          <Identifier value={issuerId} />
        </Field>
        <Field icon={UserRound} label="Holder">
          <Identifier value={asString(holder)} />
        </Field>
        <Field icon={CalendarClock} label="Valid from">
          {validFrom ? <FormatDate date={validFrom} /> : "—"}
        </Field>
        <Field icon={CalendarClock} label="Valid until">
          {validUntil ? <FormatDate date={validUntil} /> : "Never"}
        </Field>
        <Field icon={Fingerprint} label="Credential ID">
          <Identifier value={asString(doc.id)} />
        </Field>
      </dl>

      {status ? (
        <section className="space-y-2">
          <div className="flex items-center justify-between gap-2">
            <SectionTitle icon={ListChecks}>Status</SectionTitle>
            {status.onRecheck && (
              <Button
                variant="ghost"
                size="sm"
                className="h-7 text-xs"
                disabled={status.refreshing}
                onClick={(e) => {
                  e.stopPropagation();
                  status.onRecheck?.();
                }}
              >
                <RefreshCw className={cn("mr-1 h-3 w-3", status.refreshing && "animate-spin")} />
                Check again
              </Button>
            )}
          </div>
          {status.entries.length === 0 ? (
            <p className="text-sm text-muted-foreground">This credential has no status list.</p>
          ) : (
            <ul className="space-y-1.5 text-sm">
              {status.entries.map((entry) => (
                <li
                  key={`${entry.purpose} ${entry.url}`}
                  className="flex flex-wrap items-center gap-2"
                >
                  <span className="w-24 capitalize text-muted-foreground">{entry.purpose}</span>
                  <EntryBadge state={entry.state} />
                  <span className="text-muted-foreground">index {entry.index} in</span>
                  <Identifier value={entry.url} tail={24} />
                </li>
              ))}
            </ul>
          )}
          <p className="text-xs text-muted-foreground">{status.source}</p>
        </section>
      ) : (
        statuses.length > 0 && (
          <section className="space-y-2">
            <SectionTitle icon={ListChecks}>Status lists</SectionTitle>
            <ul className="space-y-1.5 text-sm">
              {statuses.map((entry, i) => (
                <li key={i} className="flex flex-wrap items-center gap-2">
                  <Badge variant="info">{asString(entry.statusPurpose) ?? "status"}</Badge>
                  <span className="text-muted-foreground">
                    index {asString(entry.statusListIndex) ?? "—"} in
                  </span>
                  <Identifier value={asString(entry.statusListCredential)} tail={24} />
                </li>
              ))}
            </ul>
          </section>
        )
      )}

      <section className="space-y-2">
        <SectionTitle icon={ListChecks}>Claims</SectionTitle>
        <JsonTable value={claims} rawToggle={false} expandDepth={2} emptyText="No claims" />
      </section>

      <section className="space-y-2">
        <SectionTitle icon={Braces}>Signed payload</SectionTitle>
        <JsonTable value={payload} />
      </section>
    </div>
  );
}

/** One word for the state of a whole credential, with a spinner while it is checked. */
export function CredentialStatusBadge({ status }: { status?: CredentialStatus }) {
  const state = overallState(status);
  if (state === "Checking") {
    return (
      <Badge variant="outline" className="gap-1">
        <Loader2 className="h-3 w-3 animate-spin" />
        Checking…
      </Badge>
    );
  }
  const variant =
    state === "Revoked"
      ? "destructive"
      : state === "Suspended"
        ? "warning"
        : state === "Active"
          ? "success"
          : "outline";
  return (
    <Badge variant={variant} className="gap-1">
      {state}
      {status?.refreshing && <Loader2 className="h-3 w-3 animate-spin" />}
    </Badge>
  );
}

function EntryBadge({ state }: { state: EntryState }) {
  if (state === "checking") {
    return (
      <Badge variant="outline" className="gap-1">
        <Loader2 className="h-3 w-3 animate-spin" />
        Checking…
      </Badge>
    );
  }
  const [variant, label] = {
    active: ["success", "Active"],
    revoked: ["destructive", "Revoked"],
    suspended: ["warning", "Suspended"],
    unknown: ["outline", "Unknown"],
  }[state] as ["success" | "destructive" | "warning" | "outline", string];
  return <Badge variant={variant}>{label}</Badge>;
}

function Field({
  icon: Icon,
  label,
  children,
}: {
  icon: React.ElementType;
  label: string;
  children: React.ReactNode;
}) {
  return (
    <div className="space-y-1">
      <dt className="flex items-center gap-1.5 text-xs font-semibold uppercase tracking-wider text-muted-foreground/70">
        <Icon className="h-3.5 w-3.5" />
        {label}
      </dt>
      <dd className="flex flex-wrap items-center text-sm">{children}</dd>
    </div>
  );
}

function SectionTitle({
  icon: Icon,
  children,
}: {
  icon: React.ElementType;
  children: React.ReactNode;
}) {
  return (
    <h4 className="flex items-center gap-2 text-xs font-bold uppercase tracking-widest text-primary/60">
      <Icon className="h-3.5 w-3.5" />
      {children}
    </h4>
  );
}

/** The payload of a JWT, or `null` if it does not decode. */
export function decodeJwtPayload(jwt: string): unknown {
  try {
    const base64 = jwt.split(".")[1].replace(/-/g, "+").replace(/_/g, "/");
    const padded = base64 + "=".repeat((4 - (base64.length % 4)) % 4);
    const bytes = Uint8Array.from(atob(padded), (c) => c.charCodeAt(0));
    return JSON.parse(new TextDecoder().decode(bytes));
  } catch {
    return null;
  }
}

/** The credential inside a payload: under `vc` in a VCDM 1.1 JWT, the payload itself in 2.0. */
function credentialDocument(payload: unknown): Doc | null {
  if (!payload || typeof payload !== "object") return null;
  const vc = (payload as Doc).vc;
  return vc && typeof vc === "object" ? (vc as Doc) : (payload as Doc);
}

const asDoc = (value: unknown): Doc =>
  value && typeof value === "object" && !Array.isArray(value) ? (value as Doc) : {};

const asString = (value: unknown): string | undefined =>
  typeof value === "string" && value ? value : undefined;

const asStrings = (value: unknown): string[] =>
  asList(value).filter((v): v is string => typeof v === "string");

const asList = (value: unknown): unknown[] =>
  Array.isArray(value) ? value : value == null ? [] : [value];
