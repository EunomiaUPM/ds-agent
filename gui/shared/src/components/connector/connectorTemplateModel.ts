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

// Pure model of a connector template being edited: defaults, placeholders and checks.
// The checks mirror the connector crate's, so most refusals show before saving.

import type {
  AuthenticationConfig,
  ConnectorParameterDefinition,
  InteractionConfig,
} from "shared/src/data/orval/model";

export type ParamType = ConnectorParameterDefinition["paramType"];
export type Mode = "PULL" | "PUSH";
export type Protocol = "HTTP" | "KAFKA";

/** The sections of a template that carry placeholders. */
export interface TemplateBody {
  authentication: AuthenticationConfig;
  interaction: InteractionConfig;
  parameters: ConnectorParameterDefinition[];
}

/** A protocol step read loosely: the form edits HTTP and Kafka fields alike. */
export type StepSpec = { protocol?: string } & Record<string, unknown>;

export interface LooseInteraction {
  mode?: Mode;
  lifetime?: "INFINITE" | "FINITE";
  maxRetries?: number | null;
  dataAccess?: StepSpec;
  subscribe?: StepSpec;
  unsubscribe?: StepSpec | null;
}

export type LooseAuth = { type?: string } & Record<string, unknown>;

export interface UsedPlaceholder {
  name: string;
  /** The type its position asks for. */
  type: ParamType;
  where: string[];
}

export interface TemplateIssue {
  id: string;
  level: "error" | "warning";
  message: string;
  fix?: { label: string; apply: (body: TemplateBody) => TemplateBody };
}

export interface TemplatePreset {
  id: string;
  label: string;
  mode: Mode;
  protocol: Protocol;
  name: string;
  description: string;
  body: TemplateBody;
}

const PLACEHOLDER = /\{\{\s*__(.*?)__\s*\}\}/g;
const RUNTIME_KNOWN = /^RUNTIME_(INGRESS|JSON_\{.+\}|PARAMETER_\{.+\}|SECRET_\{.+\})$/;
const SYS_KNOWN = new Set([
  "SYS_URN",
  "SYS_TOKEN",
  "SYS_TIMESTAMP",
  "SYS_ISO8601",
  "SYS_OWN_URL",
  "SYS_OWN_URL_DOCKER",
]);
const SASL_BASIC = new Set(["PLAIN", "SCRAM_SHA_256", "SCRAM_SHA_512"]);

/** Backend Kafka client properties allowlisted by KafkaSpec validation. */
export const ALLOWED_KAFKA_CLIENT_PROPERTIES = [
  "session.timeout.ms",
  "max.poll.interval.ms",
  "fetch.min.bytes",
  "fetch.max.bytes",
  "ssl.ca.location",
] as const;

const ALLOWED_KAFKA_CLIENT_PROPERTIES_SET = new Set<string>(ALLOWED_KAFKA_CLIENT_PROPERTIES);

/** Validates that a default value string parses according to the backend ParamType rules. */
export function validateParameterDefaultValue(
  paramType: ParamType,
  raw?: string | null,
): string | null {
  if (raw === undefined || raw === null) return null;
  if (paramType !== "STRING" && raw.trim() === "") {
    return `Default value cannot be blank for ${paramType}; leave it unset instead.`;
  }
  if (paramType === "INT") {
    const trimmed = raw.trim();
    if (!/^-?\d+$/.test(trimmed)) return "Must be a valid integer (e.g. 30).";
    try {
      const n = BigInt(trimmed);
      if (n < -9223372036854775808n || n > 9223372036854775807n) {
        return "Must fit in a signed 64-bit integer.";
      }
    } catch {
      return "Must be a valid integer.";
    }
  } else if (paramType === "BOOLEAN") {
    const lower = raw.trim().toLowerCase();
    if (lower !== "true" && lower !== "false") return 'Must be "true" or "false".';
  } else if (paramType === "VEC<STRING>") {
    try {
      const parsed: unknown = JSON.parse(raw);
      if (!Array.isArray(parsed) || !parsed.every((item) => typeof item === "string")) {
        return 'Must be a JSON array of strings, e.g. ["a", "b"].';
      }
    } catch {
      return 'Must be valid JSON array syntax, e.g. ["a", "b"].';
    }
  } else if (paramType === "MAP<STRING,STRING>") {
    try {
      const parsed: unknown = JSON.parse(raw);
      if (
        !parsed ||
        typeof parsed !== "object" ||
        Array.isArray(parsed) ||
        !Object.values(parsed).every((v) => typeof v === "string")
      ) {
        return 'Must be a JSON object of string values, e.g. {"Accept": "application/json"}.';
      }
    } catch {
      return 'Must be valid JSON object syntax, e.g. {"key": "value"}.';
    }
  }
  return null;
}

const NO_AUTH = { type: "NO_AUTH" } as unknown as AuthenticationConfig;
const secret = (content: string) => ({ type: "PLAIN", content });

export const ConnectorTemplateModel = {
  HTTP_METHODS: ["GET", "POST", "PUT", "PATCH", "DELETE"],

  /** Placeholders the agent or the dataplane fill by themselves. */
  BUILT_IN: [
    { token: "{{__SYS_URN__}}", when: "On instantiation", what: "A new URN" },
    { token: "{{__SYS_TOKEN__}}", when: "On instantiation", what: "A random token" },
    { token: "{{__SYS_ISO8601__}}", when: "On instantiation", what: "The current date and time" },
    { token: "{{__SYS_OWN_URL__}}", when: "On instantiation", what: "This agent's base URL" },
    {
      token: "{{__RUNTIME_INGRESS__}}",
      when: "At transfer time",
      what: "The dataplane path the back-end pushes to",
    },
    {
      token: "{{__RUNTIME_SECRET_{/path}__}}",
      when: "At transfer time",
      what: "A keystore secret",
    },
    {
      token: "{{__RUNTIME_PARAMETER_{/path}__}}",
      when: "At transfer time",
      what: "A keystore parameter",
    },
    {
      token: "{{__RUNTIME_JSON_{.field}__}}",
      when: "At transfer time",
      what: "A jq path over the subscribe response",
    },
  ],

  interaction(body: TemplateBody): LooseInteraction {
    return (body.interaction ?? {}) as unknown as LooseInteraction;
  },

  auth(body: TemplateBody): LooseAuth {
    return (body.authentication ?? {}) as unknown as LooseAuth;
  },

  mode(body: TemplateBody): Mode {
    return ConnectorTemplateModel.interaction(body).mode === "PUSH" ? "PUSH" : "PULL";
  },

  /** The step that decides the protocol: data access in PULL, subscribe in PUSH. */
  mainStep(body: TemplateBody): StepSpec | undefined {
    const i = ConnectorTemplateModel.interaction(body);
    return i.dataAccess ?? i.subscribe;
  },

  protocol(body: TemplateBody): Protocol {
    return ConnectorTemplateModel.mainStep(body)?.protocol === "KAFKA" ? "KAFKA" : "HTTP";
  },

  httpStep(method = "GET"): StepSpec {
    return { protocol: "HTTP", urlTemplate: "", method: [method] };
  },

  kafkaStep(): StepSpec {
    return {
      protocol: "KAFKA",
      brokers: [],
      topic: "",
      securityProtocol: "PLAINTEXT",
      autoOffsetReset: "LATEST",
    };
  },

  authOfType(type: string): AuthenticationConfig {
    const auth: Record<string, LooseAuth> = {
      NO_AUTH: { type: "NO_AUTH" },
      BASIC_AUTH: { type: "BASIC_AUTH", username: "", password: secret("") },
      BEARER_TOKEN: { type: "BEARER_TOKEN", token: secret("") },
      API_KEY: { type: "API_KEY", key: "", value: secret(""), location: "HEADER" },
      OAUTH2: {
        type: "OAUTH2",
        grantType: { type: "CLIENT_CREDENTIALS" },
        tokenUrl: "",
        clientId: "",
        clientSecret: secret(""),
        scopes: [],
      },
    };
    return (auth[type] ?? auth.NO_AUTH) as unknown as AuthenticationConfig;
  },

  withInteraction(body: TemplateBody, patch: Partial<LooseInteraction>): TemplateBody {
    const next = { ...ConnectorTemplateModel.interaction(body), ...patch };
    return { ...body, interaction: next as unknown as InteractionConfig };
  },

  /** Switches mode keeping the HTTP step; Kafka only exists as a PUSH subscription. */
  withMode(body: TemplateBody, mode: Mode): TemplateBody {
    const {
      dataAccess,
      subscribe,
      unsubscribe: _unsubscribe,
      ...rest
    } = ConnectorTemplateModel.interaction(body);
    const current = dataAccess ?? subscribe;
    const http = current?.protocol === "KAFKA" ? undefined : current;
    const interaction =
      mode === "PULL"
        ? { ...rest, mode, dataAccess: http ?? ConnectorTemplateModel.httpStep("GET") }
        : { ...rest, mode, subscribe: http ?? ConnectorTemplateModel.httpStep("POST") };
    return { ...body, interaction: interaction as unknown as InteractionConfig };
  },

  withProtocol(body: TemplateBody, protocol: Protocol): TemplateBody {
    if (ConnectorTemplateModel.protocol(body) === protocol) return body;
    if (protocol === "KAFKA") {
      return ConnectorTemplateModel.withInteraction(
        { ...body, authentication: NO_AUTH },
        { subscribe: ConnectorTemplateModel.kafkaStep(), unsubscribe: undefined },
      );
    }
    return ConnectorTemplateModel.withInteraction(body, {
      subscribe: ConnectorTemplateModel.httpStep("POST"),
    });
  },

  /** Every `{{__NAME__}}` of `text`, built-in ones included. */
  namesIn(text: string): string[] {
    return [...text.matchAll(PLACEHOLDER)].map((m) => m[1]);
  },

  isBuiltIn(name: string): boolean {
    return name.startsWith("SYS_") || name.startsWith("RUNTIME_");
  },

  /** The placeholders an instance has to fill, with the type each position asks for. */
  usedPlaceholders(body: TemplateBody): UsedPlaceholder[] {
    const found = new Map<string, UsedPlaceholder>();
    const builtIn: string[] = [];
    ConnectorTemplateModel.scan(body, (name, type, where) => {
      if (ConnectorTemplateModel.isBuiltIn(name)) {
        builtIn.push(name);
        return;
      }
      const entry = found.get(name) ?? { name, type, where: [] };
      // A list or map position wins: it is the stricter one.
      if (type !== "STRING") entry.type = type;
      if (!entry.where.includes(where)) entry.where.push(where);
      found.set(name, entry);
    });
    return [...found.values()];
  },

  builtInPlaceholders(body: TemplateBody): string[] {
    const names: string[] = [];
    ConnectorTemplateModel.scan(body, (name) => {
      if (ConnectorTemplateModel.isBuiltIn(name) && !names.includes(name)) names.push(name);
    });
    return names;
  },

  scan(
    body: TemplateBody,
    visit: (name: string, type: ParamType, where: string) => void,
    onUnextractableMap?: (name: string, where: string, isKey: boolean) => void,
  ): void {
    const text = (where: string, value: unknown, type: ParamType = "STRING") => {
      if (typeof value !== "string") return;
      ConnectorTemplateModel.namesIn(value).forEach((name) => visit(name, type, where));
    };
    const list = (where: string, value: unknown) => {
      if (typeof value === "string") text(where, value, "VEC<STRING>");
      // Backend TemplateVecString extracts items inside Value(...) as VEC<STRING>.
      else if (Array.isArray(value)) value.forEach((item) => text(where, item, "VEC<STRING>"));
    };
    const map = (where: string, value: unknown) => {
      if (typeof value === "string") text(where, value, "MAP<STRING,STRING>");
      else if (value && typeof value === "object") {
        Object.entries(value).forEach(([k, v]) => {
          ConnectorTemplateModel.namesIn(k).forEach((name) =>
            onUnextractableMap?.(name, where, true),
          );
          if (k === "__EXTRA__") {
            text(`${where} (__EXTRA__)`, v, "MAP<STRING,STRING>");
          } else if (typeof v === "string") {
            // Only RUNTIME_* resolves inside fixed map entries; parameters and SYS_* do not.
            ConnectorTemplateModel.namesIn(v).forEach((name) => {
              if (name.startsWith("RUNTIME_")) visit(name, "STRING", where);
              else onUnextractableMap?.(name, where, false);
            });
          }
        });
      }
    };
    const step = (label: string, spec?: StepSpec | null) => {
      if (!spec) return;
      if (spec.protocol === "KAFKA") {
        list(`${label} brokers`, spec.brokers);
        text(`${label} topic`, spec.topic);
        text(`${label} consumer group`, spec.groupId);
        map(`${label} client properties`, spec.clientProperties);
      } else {
        text(`${label} URL`, spec.urlTemplate);
        list(`${label} methods`, spec.method);
        map(`${label} headers`, spec.headers);
        text(`${label} body`, spec.bodyTemplate);
      }
    };
    const authField = (value: unknown) => {
      if (typeof value === "string") text("Authentication", value);
      else if (value && typeof value === "object" && !Array.isArray(value)) {
        Object.entries(value).forEach(([k, v]) =>
          k === "scopes" ? list("Authentication scopes", v) : authField(v),
        );
      }
    };

    const i = ConnectorTemplateModel.interaction(body);
    step("Data access", i.dataAccess);
    step("Subscribe", i.subscribe);
    step("Unsubscribe", i.unsubscribe);
    const { type: _type, ...auth } = ConnectorTemplateModel.auth(body);
    authField(auth);
  },

  declare(body: TemplateBody, name: string, type: ParamType): TemplateBody {
    if (body.parameters.some((p) => p.name === name)) return body;
    const title = name.charAt(0) + name.slice(1).toLowerCase().replace(/_/g, " ");
    return {
      ...body,
      parameters: [...body.parameters, { name, title, paramType: type, required: true }],
    };
  },

  removeParameter(body: TemplateBody, name: string): TemplateBody {
    return { ...body, parameters: body.parameters.filter((p) => p.name !== name) };
  },

  /** Renames a declaration and every place that uses it. */
  renameParameter(body: TemplateBody, index: number, to: string): TemplateBody {
    const from = body.parameters[index]?.name ?? "";
    const parameters = body.parameters.map((p, i) => (i === index ? { ...p, name: to } : p));
    if (!from || !to || body.parameters.some((p, i) => i !== index && p.name === to)) {
      return { ...body, parameters };
    }
    const rewrite = <T>(section: T): T =>
      JSON.parse(
        JSON.stringify(section).replace(PLACEHOLDER, (whole: string, name: string) =>
          name === from ? `{{__${to}__}}` : whole,
        ),
      );
    return {
      authentication: rewrite(body.authentication),
      interaction: rewrite(body.interaction),
      parameters,
    };
  },

  /** What the agent would refuse, and what looks wrong, with a fix where there is one. */
  issues(name: string, version: string, body: TemplateBody): TemplateIssue[] {
    const issues: TemplateIssue[] = [];
    const error = (id: string, message: string, fix?: TemplateIssue["fix"]) =>
      issues.push({ id, level: "error", message, fix });
    const warning = (id: string, message: string) => issues.push({ id, level: "warning", message });

    if (!name.trim()) error("name", "Give the template a name.");
    if (!version.trim()) error("version", "Give the template a version.");

    const i = ConnectorTemplateModel.interaction(body);
    const auth = ConnectorTemplateModel.auth(body);
    const steps: [string, StepSpec | null | undefined][] = [
      ["Data access", i.dataAccess],
      ["Subscribe", i.subscribe],
      ["Unsubscribe", i.unsubscribe],
    ];

    if (i.mode === "PULL" && !i.dataAccess) error("pull-step", "PULL needs a data access step.");
    if (i.mode === "PUSH" && !i.subscribe) error("push-step", "PUSH needs a subscribe step.");
    if (i.mode !== "PULL" && i.mode !== "PUSH") error("mode", "Mode must be PULL or PUSH.");
    if (
      i.maxRetries != null &&
      (!Number.isInteger(i.maxRetries) || i.maxRetries < 0 || i.maxRetries > 20)
    ) {
      error("retries", "Max retries must be a whole number between 0 and 20.");
    }

    const secretContent = (v: unknown) =>
      v && typeof v === "object" ? String((v as { content?: unknown }).content ?? "").trim() : "";

    if (auth.type === "BASIC_AUTH") {
      if (!String(auth.username ?? "").trim() || !secretContent(auth.password)) {
        warning(
          "auth-basic",
          "Basic auth is missing username or password (use a literal, parameter, or {{__RUNTIME_SECRET_{/path}__}}).",
        );
      }
    } else if (auth.type === "BEARER_TOKEN") {
      if (!secretContent(auth.token)) {
        warning(
          "auth-bearer",
          "Bearer token is empty (use a parameter or {{__RUNTIME_SECRET_{/path}__}}).",
        );
      }
    } else if (auth.type === "API_KEY") {
      if (!String(auth.key ?? "").trim() || !secretContent(auth.value)) {
        warning("auth-apikey", "API key auth is missing header/query name or secret value.");
      }
    } else if (auth.type === "OAUTH2") {
      if (
        !String(auth.tokenUrl ?? "").trim() ||
        !String(auth.clientId ?? "").trim() ||
        !secretContent(auth.clientSecret)
      ) {
        warning("auth-oauth2", "OAuth 2.0 is missing token URL, client ID, or client secret.");
      }
    }

    for (const [label, spec] of steps) {
      if (!spec) continue;
      if (spec.protocol === "KAFKA") {
        if (label !== "Subscribe") {
          error(
            `${label}-kafka`,
            `Kafka is only allowed as the PUSH subscribe step, not in ${label}.`,
          );
          continue;
        }
        const brokers = spec.brokers;
        if (!brokers || (Array.isArray(brokers) && brokers.length === 0)) {
          error("brokers", "Add at least one Kafka broker, or ask for them with a parameter.");
        }
        if (!String(spec.topic ?? "").trim()) error("topic", "Set the Kafka topic.");
        const sasl = String(spec.securityProtocol ?? "PLAINTEXT").startsWith("SASL");
        const mechanism = spec.saslMechanism ? String(spec.saslMechanism) : undefined;
        if (sasl && !mechanism) error("sasl", "A SASL security protocol needs a SASL mechanism.");
        if (!sasl && mechanism) error("sasl", "Remove the SASL mechanism or pick a SASL protocol.");
        if (auth.type === "NO_AUTH" && sasl) {
          error(
            "kafka-auth",
            "SASL needs credentials: choose Basic auth, a bearer token or OAuth 2.0.",
          );
        } else if (auth.type === "BASIC_AUTH" && !(sasl && SASL_BASIC.has(mechanism ?? ""))) {
          error("kafka-auth", "Basic auth goes with SASL PLAIN or SCRAM.");
        } else if (
          (auth.type === "BEARER_TOKEN" || auth.type === "OAUTH2") &&
          !(sasl && mechanism === "OAUTHBEARER")
        ) {
          error("kafka-auth", "A bearer token or OAuth 2.0 goes with SASL OAUTHBEARER.");
        } else if (auth.type === "API_KEY") {
          error("kafka-auth", "Kafka does not take an API key.");
        }
        if (spec.clientProperties && typeof spec.clientProperties === "object") {
          for (const key of Object.keys(spec.clientProperties as Record<string, unknown>)) {
            if (key !== "__EXTRA__" && !ALLOWED_KAFKA_CLIENT_PROPERTIES_SET.has(key)) {
              error(
                `kafka-prop-${key}`,
                `${label} client property "${key || "(empty)"}" is not allowlisted (${ALLOWED_KAFKA_CLIENT_PROPERTIES.join(", ")}, or __EXTRA__).`,
              );
            }
          }
        }
      } else {
        if (!String(spec.urlTemplate ?? "").trim()) error(`${label}-url`, `${label}: set the URL.`);
        const method = spec.method;
        if (!method || (Array.isArray(method) && method.length === 0)) {
          error(`${label}-method`, `${label}: choose at least one HTTP method.`);
        }
      }
    }

    const badMapPlaceholders: Array<{ name: string; where: string; isKey: boolean }> = [];
    const seenBadMap = new Set<string>();
    ConnectorTemplateModel.scan(
      body,
      () => {},
      (pname, where, isKey) => {
        const key = `${where}\0${pname}\0${isKey}`;
        if (!seenBadMap.has(key)) {
          seenBadMap.add(key);
          badMapPlaceholders.push({ name: pname, where, isKey });
        }
      },
    );
    const badMapNames = new Set(badMapPlaceholders.map((b) => b.name));
    for (const b of badMapPlaceholders) {
      error(
        `map-entry-${b.where}-${b.name}-${b.isKey ? "k" : "v"}`,
        b.isKey
          ? `${b.where}: placeholder {{__${b.name}__}} in a map key is ignored by the backend. Switch to "Ask on instantiation" or use key "__EXTRA__".`
          : `${b.where}: backend ignores {{__${b.name}__}} in fixed entries; use "Ask on instantiation" (whole map), key "__EXTRA__", or {{__RUNTIME_SECRET_{...}__}}.`,
      );
    }

    const used = ConnectorTemplateModel.usedPlaceholders(body);
    const usedByName = new Map(used.map((u) => [u.name, u]));
    const declared = new Map(body.parameters.map((p) => [p.name, p]));

    for (const u of used) {
      if (declared.has(u.name)) continue;
      error(
        `undeclared-${u.name}`,
        `{{__${u.name}__}} is used in ${u.where[0]} but not declared.`,
        {
          label: "Declare",
          apply: (b) => ConnectorTemplateModel.declare(b, u.name, u.type),
        },
      );
    }

    const names = new Set<string>();
    const titles = new Set<string>();
    body.parameters.forEach((p, index) => {
      if (!p.name) return error(`param-${index}-name`, `Parameter ${index + 1} has no name.`);
      if (/\s/.test(p.name)) {
        error(`param-space-${p.name}`, `${p.name}: parameter names cannot contain spaces.`);
      }
      if (ConnectorTemplateModel.isBuiltIn(p.name)) {
        return error(
          `reserved-${p.name}`,
          `${p.name}: names starting with SYS_ or RUNTIME_ are reserved.`,
        );
      }
      if (names.has(p.name)) error(`dup-${p.name}`, `${p.name} is declared twice.`);
      names.add(p.name);
      if (!p.title?.trim()) error(`title-${p.name}`, `${p.name} needs a title.`);
      else if (titles.has(p.title))
        error(`dup-title-${p.name}`, `Two parameters share the title "${p.title}".`);
      titles.add(p.title);

      const defaultErr = validateParameterDefaultValue(p.paramType, p.defaultValue);
      if (defaultErr) error(`default-${p.name}`, `${p.name} default value: ${defaultErr}`);

      const usage = usedByName.get(p.name);
      if (!usage) {
        if (badMapNames.has(p.name)) return;
        return error(`unused-${p.name}`, `${p.name} is declared but never used.`, {
          label: "Remove",
          apply: (b) => ConnectorTemplateModel.removeParameter(b, p.name),
        });
      }
      const scalar = p.paramType === "STRING" || p.paramType === "INT" || p.paramType === "BOOLEAN";
      if (usage.type === "STRING" ? !scalar : usage.type !== p.paramType) {
        error(
          `type-${p.name}`,
          `${p.name} is ${p.paramType} but ${usage.where[0]} takes ${usage.type}.`,
          {
            label: `Make it ${usage.type}`,
            apply: (b) => ({
              ...b,
              parameters: b.parameters.map((q) =>
                q.name === p.name ? { ...q, paramType: usage.type } : q,
              ),
            }),
          },
        );
      }
    });

    for (const builtIn of ConnectorTemplateModel.builtInPlaceholders(body)) {
      const known = builtIn.startsWith("SYS_")
        ? SYS_KNOWN.has(builtIn)
        : RUNTIME_KNOWN.test(builtIn);
      if (!known)
        warning(`unknown-${builtIn}`, `{{__${builtIn}__}} is not a placeholder the agent fills.`);
    }

    return issues;
  },

  PRESETS: [
    {
      id: "http-pull",
      label: "REST API",
      mode: "PULL",
      protocol: "HTTP",
      name: "http-rest-pull",
      description:
        "The consumer fetches the data through the dataplane, which proxies an HTTP API.",
      body: {
        authentication: NO_AUTH,
        interaction: {
          mode: "PULL",
          lifetime: "INFINITE",
          maxRetries: 3,
          dataAccess: { protocol: "HTTP", urlTemplate: "{{__BASE_URL__}}", method: ["GET"] },
        },
        parameters: [
          {
            name: "BASE_URL",
            title: "Back-end URL",
            description: "URL the dataplane proxies when the consumer asks for data.",
            paramType: "STRING",
            required: true,
          },
        ],
      },
    },
    {
      id: "http-push",
      label: "Webhook",
      mode: "PUSH",
      protocol: "HTTP",
      name: "http-webhook-push",
      description: "The back-end is asked to send events to the dataplane, which forwards them.",
      body: {
        authentication: {
          type: "BEARER_TOKEN",
          token: { type: "PLAIN", content: "{{__RUNTIME_SECRET_{/webhook/token}__}}" },
        },
        interaction: {
          mode: "PUSH",
          lifetime: "INFINITE",
          maxRetries: 3,
          subscribe: {
            protocol: "HTTP",
            urlTemplate: "{{__SUBSCRIBE_URL__}}",
            method: ["POST"],
            headers: { "Content-Type": "application/json" },
            bodyTemplate: '{"callbackUrl":"{{__DATAPLANE_URL__}}{{__RUNTIME_INGRESS__}}"}',
          },
        },
        parameters: [
          {
            name: "SUBSCRIBE_URL",
            title: "Subscribe endpoint",
            description: "Back-end endpoint called when a transfer starts.",
            paramType: "STRING",
            required: true,
          },
          {
            name: "DATAPLANE_URL",
            title: "Dataplane base URL",
            description:
              "Base URL the back-end reaches this dataplane at, e.g. http://provider:1200.",
            paramType: "STRING",
            required: true,
          },
        ],
      },
    },
    {
      id: "kafka-push",
      label: "Kafka topic",
      mode: "PUSH",
      protocol: "KAFKA",
      name: "kafka-stream-push",
      description: "The dataplane consumes a topic and forwards each record to the consumer.",
      body: {
        authentication: NO_AUTH,
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
            description: "host:port of each bootstrap broker.",
            paramType: "VEC<STRING>",
            required: true,
          },
          {
            name: "KAFKA_TOPIC",
            title: "Topic",
            description: "Topic to consume and relay.",
            paramType: "STRING",
            required: true,
          },
        ],
      },
    },
    {
      id: "kafka-sasl",
      label: "Kafka topic, SASL",
      mode: "PUSH",
      protocol: "KAFKA",
      name: "kafka-sasl-push",
      description: "The same relay against a broker that asks for SASL/SCRAM credentials over TLS.",
      body: {
        authentication: {
          type: "BASIC_AUTH",
          username: "{{__SASL_USERNAME__}}",
          password: { type: "PLAIN", content: "{{__RUNTIME_SECRET_{/kafka/password}__}}" },
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
            description: "host:port of each TLS bootstrap broker.",
            paramType: "VEC<STRING>",
            required: true,
          },
          {
            name: "KAFKA_TOPIC",
            title: "Topic",
            description: "Topic to consume and relay.",
            paramType: "STRING",
            required: true,
          },
          {
            name: "SASL_USERNAME",
            title: "SASL username",
            description: "Principal the dataplane signs in as.",
            paramType: "STRING",
            required: true,
          },
        ],
      },
    },
  ] as TemplatePreset[],
};
