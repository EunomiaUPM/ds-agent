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

import React, { useEffect, useMemo, useState } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { createFileRoute, useNavigate } from "@tanstack/react-router";
import { Wand2 } from "lucide-react";
import { toast } from "sonner";
import {
  Badge,
  Button,
  Input,
  JsonViewer,
  PageHeader,
  PageLayout,
  PageSection,
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
  Textarea,
} from "@eunomia/ui";
import {
  ConnectorParametersTable,
  ConnectorSpecView,
} from "shared/src/components/connector/ConnectorSpecView";
import {
  getGetConnectorTemplateQueryKey,
  getGetConnectorTemplatesQueryKey,
  useCreateConnectorTemplate,
  useGetConnectorTemplateByNameVersion,
} from "shared/src/data/orval/connector-templates/connector-templates";
import type {
  AuthenticationConfig,
  ConnectorParameterDefinition,
  ConnectorTemplateDto,
  InteractionConfig,
} from "shared/src/data/orval/model";

interface NewConnectorSearch {
  from?: string;
  version?: string;
}

export const Route = createFileRoute("/my-catalog/connectors/new")({
  component: NewConnectorTemplatePage,
  validateSearch: (search: Record<string, unknown>): NewConnectorSearch => ({
    from: typeof search.from === "string" ? search.from : undefined,
    version: typeof search.version === "string" ? search.version : undefined,
  }),
});

interface TemplateBodyJson {
  authentication: AuthenticationConfig;
  interaction: InteractionConfig;
  parameters: ConnectorParameterDefinition[];
}

interface Preset {
  id: string;
  label: string;
  badge: string;
  name: string;
  description: string;
  body: TemplateBodyJson;
}

const NO_AUTH_WIRE = { type: "NO_AUTH" } as unknown as AuthenticationConfig;

const PRESETS: Preset[] = [
  {
    id: "http-pull",
    label: "HTTP Pull (REST API)",
    badge: "PULL · HTTP",
    name: "http-rest-pull",
    description: "Consumer-driven HTTP GET pull proxy with parameterized base URL.",
    body: {
      authentication: NO_AUTH_WIRE,
      interaction: {
        mode: "PULL",
        lifetime: "INFINITE",
        maxRetries: 3,
        dataAccess: {
          protocol: "HTTP",
          urlTemplate: "{{__BASE_URL__}}",
          method: ["GET"],
        },
      },
      parameters: [
        {
          name: "BASE_URL",
          title: "Upstream HTTP URL",
          description: "Base URL proxied when the consumer requests data via EDR token.",
          paramType: "STRING",
          required: true,
          defaultValue: "https://api.example.org/v1/items",
        },
      ],
    },
  },
  {
    id: "http-push",
    label: "HTTP Push (Webhook)",
    badge: "PUSH · HTTP",
    name: "http-webhook-push",
    description: "Provider-initiated HTTP webhook subscription forwarding to consumer DataAddress.",
    body: {
      authentication: {
        type: "BEARER_TOKEN",
        token: {
          type: "PLAIN",
          content: "{{__UPSTREAM_TOKEN__}}",
        },
      },
      interaction: {
        mode: "PUSH",
        lifetime: "INFINITE",
        maxRetries: 3,
        subscribe: {
          protocol: "HTTP",
          urlTemplate: "{{__SUBSCRIBE_URL__}}",
          method: ["POST"],
          headers: {
            "Content-Type": "application/json",
          },
          bodyTemplate: '{"callbackEndpoint":"{{__RUNTIME_DSP_DATA_ADDRESS_ENDPOINT__}}"}',
        },
      },
      parameters: [
        {
          name: "SUBSCRIBE_URL",
          title: "Upstream subscribe endpoint",
          description: "Provider backend endpoint invoked when transfer starts.",
          paramType: "STRING",
          required: true,
        },
        {
          name: "UPSTREAM_TOKEN",
          title: "Upstream bearer token",
          description: "Bearer credential sent to the upstream webhook emitter.",
          paramType: "STRING",
          required: true,
        },
      ],
    },
  },
  {
    id: "kafka-push",
    label: "Kafka Push Stream",
    badge: "PUSH · KAFKA",
    name: "kafka-stream-push",
    description:
      "Supervised Kafka consumer relay forwarding topic records as HTTP POST events to the consumer DataAddress sink.",
    body: {
      authentication: NO_AUTH_WIRE,
      interaction: {
        mode: "PUSH",
        lifetime: "INFINITE",
        maxRetries: 3,
        subscribe: {
          protocol: "KAFKA",
          brokers: "{{__KAFKA_BROKERS__}}",
          topic: "{{__KAFKA_TOPIC__}}",
          securityProtocol: "PLAINTEXT",
          autoOffsetReset: "EARLIEST",
        },
      },
      parameters: [
        {
          name: "KAFKA_BROKERS",
          title: "Bootstrap brokers",
          description: "List of host:port bootstrap brokers (e.g. redpanda:9092).",
          paramType: "VEC<STRING>",
          required: true,
          defaultValue: '["redpanda:9092"]',
        },
        {
          name: "KAFKA_TOPIC",
          title: "Kafka topic",
          description: "Source Kafka topic to subscribe and relay.",
          paramType: "STRING",
          required: true,
          defaultValue: "telemetry.sensors.v1",
        },
      ],
    },
  },
  {
    id: "kafka-sasl",
    label: "Kafka Push (SASL/SCRAM)",
    badge: "PUSH · KAFKA · SASL",
    name: "kafka-sasl-scram-push",
    description:
      "Authenticated SASL_SSL SCRAM_SHA_256 Kafka topic relay with Vault-encrypted credentials.",
    body: {
      authentication: {
        type: "BASIC_AUTH",
        username: "{{__SASL_USERNAME__}}",
        password: {
          type: "PLAIN",
          content: "{{__SASL_PASSWORD__}}",
        },
      },
      interaction: {
        mode: "PUSH",
        lifetime: "INFINITE",
        maxRetries: 5,
        subscribe: {
          protocol: "KAFKA",
          brokers: "{{__KAFKA_BROKERS__}}",
          topic: "{{__KAFKA_TOPIC__}}",
          securityProtocol: "SASL_SSL",
          saslMechanism: "SCRAM_SHA_256",
          autoOffsetReset: "LATEST",
        },
      },
      parameters: [
        {
          name: "KAFKA_BROKERS",
          title: "Bootstrap brokers",
          description: "TLS Kafka bootstrap brokers (host:port).",
          paramType: "VEC<STRING>",
          required: true,
        },
        {
          name: "KAFKA_TOPIC",
          title: "Kafka topic",
          description: "Source Kafka topic.",
          paramType: "STRING",
          required: true,
        },
        {
          name: "SASL_USERNAME",
          title: "SASL username",
          description: "SCRAM_SHA_256 principal username.",
          paramType: "STRING",
          required: true,
        },
        {
          name: "SASL_PASSWORD",
          title: "SASL password",
          description: "SCRAM_SHA_256 password (encrypted in Vault upon instantiation).",
          paramType: "STRING",
          required: true,
        },
      ],
    },
  },
];

function bumpVersion(v?: string | null): string {
  if (!v) return "1.0.0";
  const m = v.match(/^(.*?)(\d+)$/);
  return m ? `${m[1]}${Number(m[2]) + 1}` : `${v}.1`;
}

function NewConnectorTemplatePage() {
  const { from, version: fromVersion } = Route.useSearch();
  const navigate = useNavigate();
  const queryClient = useQueryClient();

  const { data: baseRes } = useGetConnectorTemplateByNameVersion(from ?? "", fromVersion ?? "", {
    query: { enabled: !!(from && fromVersion) },
  });
  const base = baseRes?.status === 200 ? baseRes.data : undefined;

  const [name, setName] = useState(PRESETS[0].name);
  const [version, setVersion] = useState("1.0.0");
  const [author, setAuthor] = useState("");
  const [description, setDescription] = useState(PRESETS[0].description);
  const [jsonText, setJsonText] = useState(() => JSON.stringify(PRESETS[0].body, null, 2));
  const [loadedFromBase, setLoadedFromBase] = useState(false);
  const [serverError, setServerError] = useState<string | null>(null);

  useEffect(() => {
    if (!base || loadedFromBase) return;
    setName(base.name ?? from ?? "");
    setVersion(bumpVersion(base.version));
    setAuthor(base.author ?? "");
    setDescription(base.description ?? "");
    setJsonText(
      JSON.stringify(
        {
          authentication: base.authentication,
          interaction: base.interaction,
          parameters: base.parameters ?? [],
        },
        null,
        2,
      ),
    );
    setLoadedFromBase(true);
  }, [base, from, loadedFromBase]);

  const parsed = useMemo<{ value?: TemplateBodyJson; error?: string }>(() => {
    try {
      const obj = JSON.parse(jsonText) as Partial<TemplateBodyJson>;
      if (!obj || typeof obj !== "object") {
        return { error: "Template specification must be a JSON object." };
      }
      if (!obj.authentication || !obj.interaction) {
        return {
          error: "JSON must contain top-level `authentication`, `interaction`, and `parameters`.",
        };
      }
      return {
        value: {
          authentication: obj.authentication,
          interaction: obj.interaction,
          parameters: Array.isArray(obj.parameters) ? obj.parameters : [],
        },
      };
    } catch (e) {
      return { error: e instanceof Error ? e.message : "Invalid JSON syntax." };
    }
  }, [jsonText]);

  const applyPreset = (preset: Preset) => {
    if (!base) {
      setName(preset.name);
      setDescription(preset.description);
    }
    setJsonText(JSON.stringify(preset.body, null, 2));
    setServerError(null);
  };

  const formatJson = () => {
    if (parsed.value) {
      setJsonText(JSON.stringify(parsed.value, null, 2));
    }
  };

  const createMutation = useCreateConnectorTemplate();

  const problem = !name.trim()
    ? "Provide a template name."
    : !version.trim()
      ? "Provide a template version."
      : parsed.error;

  const fullDto: ConnectorTemplateDto | undefined = parsed.value
    ? {
        name: name.trim(),
        version: version.trim(),
        author: author.trim() || undefined,
        description: description.trim() || undefined,
        authentication: parsed.value.authentication,
        interaction: parsed.value.interaction,
        parameters: parsed.value.parameters,
      }
    : undefined;

  const handleSave = async () => {
    if (!fullDto) return;
    setServerError(null);
    try {
      const res = await createMutation.mutateAsync({ data: fullDto });
      if (res.status !== 200) {
        const msg =
          (res.data as { details?: string; message?: string } | undefined)?.details ||
          (res.data as { message?: string } | undefined)?.message ||
          "Template validation failed.";
        setServerError(msg);
        return;
      }
      await queryClient.invalidateQueries({ queryKey: getGetConnectorTemplatesQueryKey() });
      await queryClient.invalidateQueries({
        queryKey: getGetConnectorTemplateQueryKey(fullDto.name ?? ""),
      });
      toast.success(`Saved connector template ${fullDto.name}@${fullDto.version}.`);
      navigate({
        to: "/my-catalog/connectors/$templateName",
        params: { templateName: res.data.name ?? fullDto.name! },
        search: { version: res.data.version ?? fullDto.version ?? undefined },
      });
    } catch (err) {
      const msg = err instanceof Error ? err.message : "Failed to save connector template.";
      setServerError(msg);
    }
  };

  return (
    <PageLayout>
      <PageHeader
        eyebrow="Connector templates"
        title={base ? `New version of ${base.name}` : "New connector template"}
        description="Define an HTTP or Kafka connector blueprint with {{__PARAM__}} placeholders. The backend validates all placeholders, protocols, and SASL/Auth combinations on save."
        actions={
          <>
            <Button
              variant="outline"
              onClick={() =>
                navigate(
                  from
                    ? {
                        to: "/my-catalog/connectors/$templateName",
                        params: { templateName: from },
                      }
                    : { to: "/my-catalog/connectors" },
                )
              }
            >
              Cancel
            </Button>
            <Button
              onClick={handleSave}
              disabled={!!problem}
              isLoading={createMutation.isPending}
              title={problem}
            >
              Save template
            </Button>
          </>
        }
      />

      {/* Starter presets */}
      <div className="mb-5 flex flex-wrap items-center gap-2">
        <span className="mr-1 text-xs font-medium text-ink/55">Starter presets:</span>
        {PRESETS.map((p) => (
          <Button key={p.id} type="button" size="xs" variant="outline" onClick={() => applyPreset(p)}>
            <span>{p.label}</span>
            <Badge variant="outline" className="ml-1 text-[10px]">
              {p.badge}
            </Badge>
          </Button>
        ))}
      </div>

      {/* Metadata fields */}
      <div className="mb-6 grid grid-cols-1 gap-3 md:grid-cols-6">
        <label className="flex flex-col gap-1 md:col-span-2">
          <span className="text-xs font-medium">Template name</span>
          <Input
            value={name}
            onChange={(e) => setName(e.target.value)}
            disabled={!!base}
            placeholder="kafka-stream-push"
          />
        </label>
        <label className="flex flex-col gap-1">
          <span className="text-xs font-medium">Version</span>
          <Input
            value={version}
            onChange={(e) => setVersion(e.target.value)}
            placeholder="1.0.0"
          />
        </label>
        <label className="flex flex-col gap-1">
          <span className="text-xs font-medium">Author</span>
          <Input
            value={author}
            onChange={(e) => setAuthor(e.target.value)}
            placeholder="optional"
          />
        </label>
        <label className="flex flex-col gap-1 md:col-span-2">
          <span className="text-xs font-medium">Description</span>
          <Input
            value={description}
            onChange={(e) => setDescription(e.target.value)}
            placeholder="What data source this template connects to…"
          />
        </label>
      </div>

      {serverError && (
        <div className="mb-5 rounded-md border border-danger-500/30 bg-danger-500/10 px-3.5 py-2.5 text-xs text-danger-700 dark:text-danger-300">
          {serverError}
        </div>
      )}

      <div className="grid grid-cols-1 gap-6 xl:grid-cols-2">
        <PageSection
          title="Specification JSON (authentication, interaction, parameters)"
          action={
            <Button type="button" size="xs" variant="ghost" onClick={formatJson} disabled={!!parsed.error}>
              <Wand2 /> Format JSON
            </Button>
          }
        >
          <Textarea
            rows={26}
            value={jsonText}
            onChange={(e) => {
              setJsonText(e.target.value);
              setServerError(null);
            }}
            className="font-mono text-xs leading-relaxed"
            spellCheck={false}
          />
          {parsed.error && (
            <p className="mt-2 text-xs text-danger-600 dark:text-danger-400">{parsed.error}</p>
          )}
        </PageSection>

        <div className="space-y-5">
          <PageSection title="Live blueprint preview">
            {!parsed.value ? (
              <p className="text-xs text-ink/55">Fix JSON syntax to preview the connector blueprint.</p>
            ) : (
              <Tabs defaultValue="blueprint">
                <TabsList>
                  <TabsTrigger value="blueprint">Blueprint</TabsTrigger>
                  <TabsTrigger value="parameters">
                    Parameters ({parsed.value.parameters.length})
                  </TabsTrigger>
                  <TabsTrigger value="dto">Full DTO</TabsTrigger>
                </TabsList>
                <TabsContent value="blueprint" className="pt-4">
                  <ConnectorSpecView
                    interaction={parsed.value.interaction}
                    authentication={parsed.value.authentication}
                  />
                </TabsContent>
                <TabsContent value="parameters" className="pt-4">
                  <ConnectorParametersTable parameters={parsed.value.parameters} />
                </TabsContent>
                <TabsContent value="dto" className="pt-4">
                  <JsonViewer value={fullDto} />
                </TabsContent>
              </Tabs>
            )}
          </PageSection>
        </div>
      </div>
    </PageLayout>
  );
}
