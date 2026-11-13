/*
 * Copyright (C) 2026 - Universidad Politécnica de Madrid - UPM
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program. If not, see <https://www.gnu.org/licenses/>.
 */

import React, { useState } from "react";
import { Eye, EyeOff } from "lucide-react";
import {
  Badge,
  Button,
  FieldGrid,
  JsonTable,
  PageSection,
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@eunomia/ui";
import type {
  AuthenticationConfig,
  ConnectorParameterDefinition,
  ConnectorTemplateDto,
  InteractionConfig,
} from "shared/src/data/orval/model";

export interface ConnectorTemplateGroup {
  name: string;
  versions: ConnectorTemplateDto[];
  latest: ConnectorTemplateDto;
}

/** Groups flat template rows by `name`, sorting versions newest-first. */
export function groupConnectorTemplates(items: ConnectorTemplateDto[]): ConnectorTemplateGroup[] {
  const byName = new Map<string, ConnectorTemplateDto[]>();
  for (const item of items) {
    const name = item.name ?? "unnamed";
    const list = byName.get(name) ?? [];
    list.push(item);
    byName.set(name, list);
  }
  return Array.from(byName.entries())
    .map(([name, versions]) => {
      const sorted = [...versions].sort((a, b) =>
        String(b.version ?? "").localeCompare(String(a.version ?? ""), undefined, {
          numeric: true,
        }),
      );
      return { name, versions: sorted, latest: sorted[0] };
    })
    .sort((a, b) => a.name.localeCompare(b.name));
}

export function extractInteractionMode(interaction?: InteractionConfig | unknown): string {
  const raw = interaction as { mode?: string } | undefined;
  return raw?.mode ?? "UNKNOWN";
}

export function extractPrimaryProtocol(interaction?: InteractionConfig | unknown): string {
  const raw = interaction as
    | {
        dataAccess?: { protocol?: string };
        subscribe?: { protocol?: string };
      }
    | undefined;
  return raw?.dataAccess?.protocol ?? raw?.subscribe?.protocol ?? "UNKNOWN";
}

const AUTH_LABELS: Record<string, string> = {
  NO_AUTH: "None (NO_AUTH)",
  BASIC_AUTH: "Basic Auth (HTTP Basic / Kafka SASL)",
  BASIC: "Basic Auth",
  BEARER_TOKEN: "Bearer Token (HTTP Bearer / Kafka OAUTHBEARER)",
  BEARER: "Bearer Token",
  API_KEY: "API Key",
  OAUTH2: "OAuth 2.0",
};

const SECRET_KEY_RE = /pass|secret|token|value/i;

function maskValue(val: unknown, reveal: boolean, parentKey = ""): unknown {
  if (reveal || val == null) return val;
  if (typeof val === "object" && !Array.isArray(val)) {
    const rec = val as Record<string, unknown>;
    // `SecretString` wire representation: `{ type: "PLAIN" | "BASE64" | "ENV_VAR", content: "..." }`
    if (typeof rec.type === "string" && typeof rec.content === "string") {
      const isRuntimeRef = rec.content.includes("{{__");
      return {
        type: rec.type,
        content: isRuntimeRef ? rec.content : "••••••••",
      };
    }
    return Object.fromEntries(
      Object.entries(rec).map(([k, v]) => [k, maskValue(v, reveal, k)]),
    );
  }
  if (typeof val === "string" && SECRET_KEY_RE.test(parentKey) && !val.includes("{{__")) {
    return "••••••••";
  }
  return val;
}

function containsMaskableSecret(auth: Record<string, unknown>): boolean {
  for (const [k, v] of Object.entries(auth)) {
    if (k === "type") continue;
    if (v && typeof v === "object" && "content" in (v as Record<string, unknown>)) {
      return true;
    }
    if (typeof v === "string" && SECRET_KEY_RE.test(k)) {
      return true;
    }
  }
  return false;
}

type AnyProtocolSpec = {
  protocol?: string;
  // HTTP
  urlTemplate?: string;
  method?: string | string[];
  headers?: Record<string, string> | string | null;
  bodyTemplate?: unknown;
  // KAFKA
  brokers?: string | string[];
  topic?: string;
  groupId?: string | null;
  securityProtocol?: string;
  saslMechanism?: string | null;
  autoOffsetReset?: string;
  clientProperties?: Record<string, string> | string | null;
};

function ProtocolStepSection({ title, spec }: { title: string; spec: AnyProtocolSpec }) {
  if (spec.protocol === "KAFKA") {
    const brokers = Array.isArray(spec.brokers)
      ? spec.brokers
      : spec.brokers
        ? [spec.brokers]
        : [];
    return (
      <PageSection title={title}>
        <FieldGrid
          columns={3}
          fields={[
            {
              label: "Protocol",
              value: <Badge variant="info">KAFKA</Badge>,
            },
            {
              label: "Topic",
              value: <span className="font-mono text-xs">{spec.topic || "—"}</span>,
            },
            {
              label: "Consumer group",
              value: spec.groupId ? (
                <span className="font-mono text-xs">{spec.groupId}</span>
              ) : (
                <span className="text-xs text-ink/55">ds-&lt;dataplaneId&gt; (per-transfer)</span>
              ),
            },
            {
              label: "Security protocol",
              value: (
                <span className="font-mono text-xs">{spec.securityProtocol ?? "PLAINTEXT"}</span>
              ),
            },
            {
              label: "SASL mechanism",
              value: spec.saslMechanism ? (
                <span className="font-mono text-xs">{spec.saslMechanism}</span>
              ) : (
                <span className="text-ink/40">—</span>
              ),
            },
            {
              label: "Auto offset reset",
              value: <span className="font-mono text-xs">{spec.autoOffsetReset ?? "LATEST"}</span>,
            },
            {
              label: "Bootstrap brokers",
              span: 3,
              value: (
                <span className="flex flex-wrap gap-1.5">
                  {brokers.map((b) => (
                    <span
                      key={b}
                      className="rounded bg-ink/[0.07] px-2 py-0.5 font-mono text-xs"
                    >
                      {b}
                    </span>
                  ))}
                </span>
              ),
            },
          ]}
        />
        {!!spec.clientProperties && (
          <div className="mt-3">
            <p className="mb-1.5 text-xs font-medium text-ink/50">Kafka client properties</p>
            <JsonTable value={spec.clientProperties} rawToggle={false} />
          </div>
        )}
      </PageSection>
    );
  }

  const transport = spec.urlTemplate?.startsWith("https")
    ? "TLS (HTTPS)"
    : spec.urlTemplate?.startsWith("http://")
      ? "Plain HTTP"
      : "Parameterized URL";

  return (
    <PageSection title={title}>
      <FieldGrid
        columns={3}
        fields={[
          {
            label: "Protocol",
            value: <Badge variant="outline">{spec.protocol ?? "HTTP"}</Badge>,
          },
          {
            label: "Methods",
            value: (
              <span className="flex flex-wrap gap-1">
                {[spec.method ?? []].flat().map((m) => (
                  <span
                    key={m}
                    className="rounded bg-ink/[0.07] px-1.5 py-0.5 font-mono text-xs"
                  >
                    {m}
                  </span>
                ))}
              </span>
            ),
          },
          {
            label: "Transport",
            value: transport,
          },
          {
            label: "Endpoint",
            span: 3,
            value: <span className="break-all font-mono text-xs">{spec.urlTemplate}</span>,
          },
        ]}
      />
      {(!!spec.headers || !!spec.bodyTemplate) && (
        <div className="mt-3 grid grid-cols-1 gap-4 xl:grid-cols-2">
          {!!spec.headers && (
            <div>
              <p className="mb-1.5 text-xs font-medium text-ink/50">Request headers</p>
              <JsonTable value={spec.headers} rawToggle={false} />
            </div>
          )}
          {!!spec.bodyTemplate && (
            <div>
              <p className="mb-1.5 text-xs font-medium text-ink/50">Body template</p>
              <JsonTable value={spec.bodyTemplate} />
            </div>
          )}
        </div>
      )}
    </PageSection>
  );
}

export function ConnectorSpecView({
  interaction,
  authentication,
}: {
  interaction: InteractionConfig | unknown;
  authentication: AuthenticationConfig | unknown;
}) {
  const [reveal, setReveal] = useState(false);

  const rawInteraction = (interaction ?? {}) as {
    mode?: string;
    lifetime?: string;
    maxRetries?: number | null;
    dataAccess?: AnyProtocolSpec;
    subscribe?: AnyProtocolSpec;
    unsubscribe?: AnyProtocolSpec | null;
  };
  const steps: [string, AnyProtocolSpec | null | undefined][] = rawInteraction.dataAccess
    ? [["Data access", rawInteraction.dataAccess]]
    : [
        ["Subscribe", rawInteraction.subscribe],
        ["Unsubscribe", rawInteraction.unsubscribe],
      ];

  const authObj = (authentication ?? {}) as Record<string, unknown>;
  const authType = String(authObj.type ?? "NO_AUTH");
  const hasSecrets = containsMaskableSecret(authObj);
  const maskedAuthFields = Object.fromEntries(
    Object.entries(authObj)
      .filter(([k]) => k !== "type")
      .map(([k, v]) => [k, maskValue(v, reveal, k)]),
  );

  return (
    <div className="space-y-5">
      <PageSection title="Interaction & delivery policy">
        <FieldGrid
          columns={3}
          fields={[
            {
              label: "Mode",
              value: <Badge variant="info">{rawInteraction.mode ?? "—"}</Badge>,
            },
            {
              label: "Transfer lifetime",
              value: (
                <Badge variant={rawInteraction.lifetime === "FINITE" ? "warning" : "outline"}>
                  {rawInteraction.lifetime ?? "INFINITE"}
                </Badge>
              ),
            },
            {
              label: "Max transient retries",
              value: (
                <span className="font-mono text-xs">
                  {rawInteraction.maxRetries != null
                    ? String(rawInteraction.maxRetries)
                    : "3 (default)"}
                </span>
              ),
            },
          ]}
        />
      </PageSection>

      {steps
        .filter((entry): entry is [string, AnyProtocolSpec] => !!entry[1])
        .map(([title, spec]) => (
          <ProtocolStepSection key={title} title={title} spec={spec} />
        ))}

      <PageSection
        title="Authentication"
        action={
          hasSecrets && (
            <Button variant="ghost" size="xs" onClick={() => setReveal((r) => !r)}>
              {reveal ? <EyeOff /> : <Eye />}
              {reveal ? "Hide secrets" : "Show secrets"}
            </Button>
          )
        }
      >
        <FieldGrid
          columns={3}
          fields={[{ label: "Strategy", value: AUTH_LABELS[authType] ?? authType }]}
        />
        {Object.keys(maskedAuthFields).length > 0 && (
          <JsonTable value={maskedAuthFields} rawToggle={false} className="mt-3" />
        )}
      </PageSection>
    </div>
  );
}

export function ConnectorParametersTable({
  parameters,
}: {
  parameters: ConnectorParameterDefinition[];
}) {
  if (!parameters || parameters.length === 0) {
    return (
      <p className="text-xs text-ink/50">
        This template declares no instance-time parameters (static or runtime-only placeholders).
      </p>
    );
  }

  return (
    <Table>
      <TableHeader>
        <TableRow>
          <TableHead>Placeholder</TableHead>
          <TableHead>Title</TableHead>
          <TableHead>Type</TableHead>
          <TableHead>Required</TableHead>
          <TableHead>Default</TableHead>
          <TableHead>Description</TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        {parameters.map((p) => (
          <TableRow key={p.name}>
            <TableCell className="font-mono text-xs">{`{{__${p.name}__}}`}</TableCell>
            <TableCell className="text-xs font-medium">{p.title}</TableCell>
            <TableCell>
              <Badge variant="outline" className="font-mono text-[11px]">
                {p.paramType}
              </Badge>
            </TableCell>
            <TableCell>
              {p.required ? (
                <Badge variant="info">Required</Badge>
              ) : (
                <span className="text-xs text-ink/45">Optional</span>
              )}
            </TableCell>
            <TableCell className="font-mono text-xs">
              {p.defaultValue != null ? (
                p.defaultValue
              ) : (
                <span className="text-ink/35">—</span>
              )}
            </TableCell>
            <TableCell className="text-xs text-ink/65">{p.description || "—"}</TableCell>
          </TableRow>
        ))}
      </TableBody>
    </Table>
  );
}
