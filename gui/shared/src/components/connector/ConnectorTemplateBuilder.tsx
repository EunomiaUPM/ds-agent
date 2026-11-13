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

// Form over a connector template body: interaction, authentication and parameters.

import React, { useId, useRef } from "react";
import { Copy, Plus, Trash2 } from "lucide-react";
import { toast } from "sonner";
import {
  Button,
  Checkbox,
  cn,
  Input,
  SegmentedControl,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  TagInput,
  Textarea,
} from "@eunomia/ui";
import type { ConnectorParameterDefinition } from "shared/src/data/orval/model";
import {
  ALLOWED_KAFKA_CLIENT_PROPERTIES,
  ConnectorTemplateModel as Model,
  type LooseAuth,
  type Mode,
  type ParamType,
  type Protocol,
  type StepSpec,
  type TemplateBody,
  validateParameterDefaultValue,
} from "./connectorTemplateModel";

const PARAM_TYPES: { value: ParamType; label: string }[] = [
  { value: "STRING", label: "Text" },
  { value: "INT", label: "Integer" },
  { value: "BOOLEAN", label: "Yes / no" },
  { value: "VEC<STRING>", label: "List of texts" },
  { value: "MAP<STRING,STRING>", label: "Key-value map" },
];

const AUTH_TYPES = [
  { value: "NO_AUTH", label: "None" },
  { value: "BASIC_AUTH", label: "Basic (user and password)" },
  { value: "BEARER_TOKEN", label: "Bearer token" },
  { value: "API_KEY", label: "API key" },
  { value: "OAUTH2", label: "OAuth 2.0" },
];

const MODES: { value: Mode; label: string; hint: string }[] = [
  { value: "PULL", label: "Pull", hint: "The consumer asks; the dataplane proxies the back-end." },
  {
    value: "PUSH",
    label: "Push",
    hint: "The back-end sends; the dataplane forwards to the consumer.",
  },
];

const MODE_HELP: Record<Mode, string> = {
  PULL: "The consumer fetches the data when it wants: the dataplane proxies each request to your back-end.",
  PUSH: "Your back-end delivers the data: the dataplane subscribes when a transfer starts and forwards what arrives.",
};

const isTemplate = (value: unknown): value is string => typeof value === "string";

function Section({
  title,
  hint,
  action,
  children,
}: {
  title: string;
  hint?: React.ReactNode;
  action?: React.ReactNode;
  children: React.ReactNode;
}) {
  return (
    <section className="rounded-lg border border-ink/10">
      <header className="flex items-center justify-between gap-3 border-b border-ink/[0.07] px-3 py-2">
        <div className="min-w-0">
          <h2 className="text-sm font-semibold text-ink/90">{title}</h2>
          {hint && <p className="text-xs text-ink/55">{hint}</p>}
        </div>
        {action}
      </header>
      <div className="space-y-3 p-3">{children}</div>
    </section>
  );
}

function Field({
  label,
  hint,
  className,
  action,
  children,
}: {
  label: string;
  hint?: React.ReactNode;
  className?: string;
  action?: React.ReactNode;
  children: React.ReactNode;
}) {
  return (
    <div className={cn("flex min-w-0 flex-col gap-1", className)}>
      <div className="flex items-center justify-between gap-2">
        <span className="text-xs font-medium text-ink/80">{label}</span>
        {action}
      </div>
      {children}
      {hint && <span className="text-xs text-ink/50">{hint}</span>}
    </div>
  );
}

function Choice({
  value,
  onChange,
  options,
  ariaLabel,
}: {
  value: string;
  onChange: (value: string) => void;
  options: { value: string; label: string; disabled?: boolean }[];
  ariaLabel: string;
}) {
  return (
    <Select value={value} onValueChange={onChange}>
      <SelectTrigger className="h-8 text-xs" aria-label={ariaLabel}>
        <SelectValue />
      </SelectTrigger>
      <SelectContent>
        {options.map((o) => (
          <SelectItem key={o.value} value={o.value} disabled={o.disabled}>
            {o.label}
          </SelectItem>
        ))}
      </SelectContent>
    </Select>
  );
}

/** Switches a list or map field between values set here and a parameter asked later. */
function AskLater({ asking, onToggle }: { asking: boolean; onToggle: () => void }) {
  return (
    <button type="button" onClick={onToggle} className="text-xs text-brand-sky hover:underline">
      {asking ? "Set it here" : "Ask on instantiation"}
    </button>
  );
}

function KeyValueRows({
  value,
  onChange,
  keyPlaceholder,
  suggestions,
  defaultExtraValue = "{{__EXTRA_MAP__}}",
}: {
  value: Record<string, string>;
  onChange: (value: Record<string, string>) => void;
  keyPlaceholder: string;
  suggestions?: readonly string[];
  defaultExtraValue?: string;
}) {
  const listId = useId();
  const entries = Object.entries(value);
  const replace = (index: number, entry: [string, string]) =>
    onChange(Object.fromEntries(entries.map((e, i) => (i === index ? entry : e))));
  const unusedSuggestions = (suggestions ?? []).filter((s) => !(s in value));

  return (
    <div className="space-y-1.5">
      {entries.map(([k, v], index) => (
        <div key={index} className="flex items-center gap-1.5">
          <Input
            list={suggestions?.length ? listId : undefined}
            className="h-8 font-mono text-xs"
            placeholder={keyPlaceholder}
            value={k}
            onChange={(e) => replace(index, [e.target.value, v])}
          />
          <Input
            className="h-8 font-mono text-xs"
            placeholder={k === "__EXTRA__" ? defaultExtraValue : "Value"}
            value={v}
            onChange={(e) => replace(index, [k, e.target.value])}
          />
          <Button
            type="button"
            size="icon_sm"
            variant="ghost"
            title="Remove"
            onClick={() => onChange(Object.fromEntries(entries.filter((_, i) => i !== index)))}
          >
            <Trash2 />
          </Button>
        </div>
      ))}
      {suggestions?.length ? (
        <datalist id={listId}>
          {unusedSuggestions.map((s) => (
            <option key={s} value={s} />
          ))}
        </datalist>
      ) : null}
      <div className="flex flex-wrap items-center gap-1.5">
        <Button
          type="button"
          size="xs"
          variant="outline"
          disabled={"" in value}
          onClick={() => onChange({ ...value, "": "" })}
        >
          <Plus /> Add
        </Button>
        {unusedSuggestions.map((s) => (
          <button
            key={s}
            type="button"
            onClick={() =>
              onChange({
                ...value,
                [s]: s === "__EXTRA__" ? defaultExtraValue : "",
              })
            }
            className="rounded border border-ink/10 bg-ink/[0.02] px-1.5 py-0.5 font-mono text-[11px] text-ink/60 transition-colors hover:border-ink/25 hover:text-ink"
            title={s === "__EXTRA__" ? "Add dynamic MAP<STRING,STRING> merge slot" : `Add ${s}`}
          >
            + {s}
          </button>
        ))}
      </div>
    </div>
  );
}

/** A map field: rows set here, or one `{{__NAME__}}` asked on instantiation. */
function MapField({
  label,
  hint,
  value,
  onChange,
  suggested,
  keyPlaceholder,
  suggestions,
}: {
  label: string;
  hint?: string;
  value: unknown;
  onChange: (value: unknown) => void;
  suggested: string;
  keyPlaceholder: string;
  suggestions?: readonly string[];
}) {
  const asking = isTemplate(value);
  const entries = !asking && value && typeof value === "object" ? Object.entries(value) : [];
  const hasUnsupportedPlaceholder = entries.some(([k, v]) => {
    if (Model.namesIn(String(k)).length > 0) return true;
    if (k === "__EXTRA__") return false;
    return Model.namesIn(String(v)).some((name) => !name.startsWith("RUNTIME_"));
  });

  return (
    <Field
      label={label}
      hint={
        hasUnsupportedPlaceholder ? (
          <span className="text-danger-600 dark:text-danger-400">
            Fixed entries ignore template parameters. Switch to “Ask on instantiation”, use key{" "}
            <code className="font-mono">__EXTRA__</code>, or use{" "}
            <code className="font-mono">{"{{__RUNTIME_SECRET_{...}__}}"}</code>.
          </span>
        ) : (
          hint
        )
      }
      action={
        <AskLater
          asking={asking}
          onToggle={() => onChange(asking ? undefined : `{{__${suggested}__}}`)}
        />
      }
    >
      {asking ? (
        <Input
          className="h-8 font-mono text-xs"
          value={value}
          onChange={(e) => onChange(e.target.value)}
        />
      ) : (
        <KeyValueRows
          value={(value ?? {}) as Record<string, string>}
          keyPlaceholder={keyPlaceholder}
          suggestions={suggestions}
          defaultExtraValue={`{{__EXTRA_${suggested}__}}`}
          onChange={(next) => onChange(Object.keys(next).length ? next : undefined)}
        />
      )}
    </Field>
  );
}

function HttpStep({
  spec,
  onChange,
  withBody,
  stepRole = "access",
}: {
  spec: StepSpec;
  onChange: (spec: StepSpec) => void;
  withBody: boolean;
  stepRole?: "access" | "subscribe" | "unsubscribe";
}) {
  const bodyRef = useRef<HTMLTextAreaElement>(null);
  const set = (patch: Record<string, unknown>) => onChange({ ...spec, ...patch });
  const methods = isTemplate(spec.method)
    ? spec.method.includes("{{")
      ? undefined
      : [spec.method]
    : ((spec.method as string[] | undefined) ?? []);

  const insertBodyToken = (token: string) => {
    const current = String(spec.bodyTemplate ?? "");
    const el = bodyRef.current;
    if (!el || typeof el.selectionStart !== "number") {
      set({ bodyTemplate: current ? `${current}${token}` : token });
      return;
    }
    const start = el.selectionStart;
    const end = el.selectionEnd ?? start;
    const next = `${current.slice(0, start)}${token}${current.slice(end)}`;
    set({ bodyTemplate: next });
    requestAnimationFrame(() => {
      el.focus();
      el.setSelectionRange(start + token.length, start + token.length);
    });
  };

  const bodyTokens =
    stepRole === "unsubscribe"
      ? ["{{__RUNTIME_JSON_{.id}__}}", "{{__RUNTIME_INGRESS__}}"]
      : ["{{__RUNTIME_INGRESS__}}", "{{__RUNTIME_PARAMETER_{/path}__}}"];

  return (
    <div className="space-y-3">
      <div className="grid grid-cols-1 gap-3 md:grid-cols-[1fr_auto]">
        <Field
          label="URL"
          hint={
            <>
              Write <code className="font-mono">{"{{__NAME__}}"}</code> wherever each distribution
              sets its own value.
            </>
          }
        >
          <Input
            className="h-8 font-mono text-xs"
            placeholder={
              stepRole === "unsubscribe"
                ? "https://api.example.org/webhooks/{{__RUNTIME_JSON_{.id}__}}"
                : "https://api.example.org/items/{{__ITEM__}}"
            }
            value={String(spec.urlTemplate ?? "")}
            onChange={(e) => set({ urlTemplate: e.target.value })}
          />
        </Field>
        <Field
          label="Methods"
          action={
            <AskLater
              asking={!methods}
              onToggle={() => set({ method: methods ? "{{__METHODS__}}" : ["GET"] })}
            />
          }
        >
          {methods ? (
            <div className="flex h-8 items-center gap-1">
              {Model.HTTP_METHODS.map((m) => {
                const on = methods.includes(m);
                return (
                  <button
                    key={m}
                    type="button"
                    aria-pressed={on}
                    onClick={() =>
                      set({ method: on ? methods.filter((x) => x !== m) : [...methods, m] })
                    }
                    className={cn(
                      "rounded border px-1.5 py-0.5 font-mono text-xs transition-colors",
                      on
                        ? "border-brand-sky/40 bg-brand-sky/10 text-brand-sky"
                        : "border-ink/10 text-ink/50 hover:text-ink",
                    )}
                  >
                    {m}
                  </button>
                );
              })}
            </div>
          ) : (
            <Input
              className="h-8 font-mono text-xs"
              value={String(spec.method ?? "")}
              onChange={(e) => set({ method: e.target.value })}
            />
          )}
        </Field>
      </div>
      <MapField
        label="Headers"
        value={spec.headers}
        suggested="HEADERS"
        keyPlaceholder="Header"
        suggestions={["Accept", "Content-Type", "__EXTRA__"]}
        onChange={(headers) => set({ headers })}
      />
      {withBody && (
        <Field
          label="Body"
          action={
            <div className="flex flex-wrap items-center gap-1">
              <span className="text-[11px] text-ink/45">Insert:</span>
              {bodyTokens.map((t) => (
                <button
                  key={t}
                  type="button"
                  onClick={() => insertBodyToken(t)}
                  className="rounded border border-ink/10 px-1.5 py-0.5 font-mono text-[11px] text-brand-sky hover:border-brand-sky/40"
                >
                  {t}
                </button>
              ))}
            </div>
          }
          hint={
            stepRole === "unsubscribe"
              ? "Sent verbatim when tearing down the subscription; use {{__RUNTIME_JSON_{.id}__}} to echo a field returned by subscribe."
              : "Sent verbatim when starting the subscription; {{__RUNTIME_INGRESS__}} is the dataplane webhook URL."
          }
        >
          <Textarea
            ref={bodyRef}
            rows={3}
            className="font-mono text-xs"
            spellCheck={false}
            value={String(spec.bodyTemplate ?? "")}
            onChange={(e) => set({ bodyTemplate: e.target.value || undefined })}
          />
        </Field>
      )}
    </div>
  );
}

function KafkaStep({ spec, onChange }: { spec: StepSpec; onChange: (spec: StepSpec) => void }) {
  const set = (patch: Record<string, unknown>) => onChange({ ...spec, ...patch });
  const security = String(spec.securityProtocol ?? "PLAINTEXT");
  const sasl = security.startsWith("SASL");
  const askingBrokers = isTemplate(spec.brokers);
  const staticBrokers =
    !askingBrokers && Array.isArray(spec.brokers) ? (spec.brokers as string[]) : [];
  const invalidStaticBrokers = staticBrokers.filter(
    (b) => !Model.namesIn(b).length && !/^[^:\s]+:\d+$/.test(b.trim()),
  );

  return (
    <div className="space-y-3">
      <div className="grid grid-cols-1 gap-3 md:grid-cols-2">
        <Field
          label="Bootstrap brokers"
          hint={
            invalidStaticBrokers.length > 0 ? (
              <span className="text-warn-700 dark:text-warn-400">
                Use host:port format (e.g. redpanda:9092): {invalidStaticBrokers.join(", ")}
              </span>
            ) : askingBrokers ? undefined : (
              "host:port, press Enter after each one."
            )
          }
          action={
            <AskLater
              asking={askingBrokers}
              onToggle={() => set({ brokers: askingBrokers ? [] : "{{__KAFKA_BROKERS__}}" })}
            />
          }
        >
          {askingBrokers ? (
            <Input
              className="h-8 font-mono text-xs"
              value={String(spec.brokers)}
              onChange={(e) => set({ brokers: e.target.value })}
            />
          ) : (
            <TagInput
              value={(spec.brokers as string[] | undefined) ?? []}
              onChange={(brokers) => set({ brokers })}
              placeholder="redpanda:9092"
            />
          )}
        </Field>
        <Field label="Topic">
          <Input
            className="h-8 font-mono text-xs"
            placeholder="telemetry.sensors.v1 or {{__KAFKA_TOPIC__}}"
            value={String(spec.topic ?? "")}
            onChange={(e) => set({ topic: e.target.value })}
          />
        </Field>
      </div>
      <div className="grid grid-cols-1 gap-3 md:grid-cols-4">
        <Field label="Security">
          <Choice
            ariaLabel="Security protocol"
            value={security}
            onChange={(securityProtocol) =>
              set({
                securityProtocol,
                saslMechanism: securityProtocol.startsWith("SASL")
                  ? (spec.saslMechanism ?? "SCRAM_SHA_256")
                  : undefined,
              })
            }
            options={[
              { value: "PLAINTEXT", label: "Plaintext" },
              { value: "SSL", label: "TLS" },
              { value: "SASL_PLAINTEXT", label: "SASL, plaintext" },
              { value: "SASL_SSL", label: "SASL over TLS" },
            ]}
          />
        </Field>
        <Field label="SASL mechanism">
          {sasl ? (
            <Choice
              ariaLabel="SASL mechanism"
              value={String(spec.saslMechanism ?? "")}
              onChange={(saslMechanism) => set({ saslMechanism })}
              options={["PLAIN", "SCRAM_SHA_256", "SCRAM_SHA_512", "OAUTHBEARER"].map((m) => ({
                value: m,
                label: m.replace(/_/g, "-"),
              }))}
            />
          ) : (
            <span className="flex h-8 items-center text-xs text-ink/40">Not used</span>
          )}
        </Field>
        <Field label="Start reading from">
          <Choice
            ariaLabel="Auto offset reset"
            value={String(spec.autoOffsetReset ?? "LATEST")}
            onChange={(autoOffsetReset) => set({ autoOffsetReset })}
            options={[
              { value: "LATEST", label: "New records only" },
              { value: "EARLIEST", label: "The beginning" },
            ]}
          />
        </Field>
        <Field label="Consumer group" hint="Empty: one per transfer.">
          <Input
            className="h-8 font-mono text-xs"
            value={String(spec.groupId ?? "")}
            onChange={(e) => set({ groupId: e.target.value || undefined })}
          />
        </Field>
      </div>
      <MapField
        label="Client properties"
        hint="Allowlisted librdkafka properties, or __EXTRA__ to merge at instantiation."
        value={spec.clientProperties}
        suggested="KAFKA_PROPERTIES"
        keyPlaceholder="property.name"
        suggestions={[...ALLOWED_KAFKA_CLIENT_PROPERTIES, "__EXTRA__"]}
        onChange={(clientProperties) => set({ clientProperties })}
      />
    </div>
  );
}

/** A credential: where it is stored and its content, usually a keystore reference. */
function SecretField({
  label,
  value,
  onChange,
  suggestedParam = "SECRET",
}: {
  label: string;
  value: unknown;
  onChange: (value: unknown) => void;
  suggestedParam?: string;
}) {
  const current = (value ?? { type: "PLAIN", content: "" }) as { type: string; content: string };
  return (
    <Field
      label={label}
      className="md:col-span-2"
      action={
        <div className="flex flex-wrap items-center gap-2 text-xs">
          <button
            type="button"
            className="font-mono text-brand-sky hover:underline"
            onClick={() => onChange({ ...current, content: "{{__RUNTIME_SECRET_{/my/secret}__}}" })}
          >
            Keystore secret
          </button>
          <button
            type="button"
            className="font-mono text-ink/60 hover:text-ink hover:underline"
            onClick={() => onChange({ ...current, content: `{{__${suggestedParam}__}}` })}
          >
            Ask on instantiation
          </button>
        </div>
      }
      hint={
        current.type === "ENV_VAR"
          ? "Name of the environment variable on the dataplane host."
          : current.type === "BASE64"
            ? "Base64-encoded secret value or keystore / template placeholder."
            : "Prefer {{__RUNTIME_SECRET_{/path}__}} to read from the keystore on each transfer."
      }
    >
      <div className="grid grid-cols-[9rem_1fr] gap-1.5">
        <Choice
          ariaLabel={`${label} storage`}
          value={current.type}
          onChange={(type) => onChange({ ...current, type })}
          options={[
            { value: "PLAIN", label: "As written" },
            { value: "BASE64", label: "Base64" },
            { value: "ENV_VAR", label: "Env variable" },
          ]}
        />
        <Input
          className="h-8 font-mono text-xs"
          placeholder={
            current.type === "ENV_VAR" ? "BACKEND_API_TOKEN" : "{{__RUNTIME_SECRET_{/my/secret}__}}"
          }
          value={current.content}
          onChange={(e) => onChange({ ...current, content: e.target.value })}
        />
      </div>
    </Field>
  );
}

function AuthFields({ auth, onChange }: { auth: LooseAuth; onChange: (auth: LooseAuth) => void }) {
  const set = (patch: Record<string, unknown>) => onChange({ ...auth, ...patch });
  const text = (key: string, label: string, placeholder?: string) => (
    <Field label={label}>
      <Input
        className="h-8 font-mono text-xs"
        placeholder={placeholder}
        value={String(auth[key] ?? "")}
        onChange={(e) => set({ [key]: e.target.value })}
      />
    </Field>
  );

  if (auth.type === "BASIC_AUTH") {
    return (
      <>
        {text("username", "Username", "service-account or {{__USERNAME__}}")}
        <SecretField
          label="Password"
          value={auth.password}
          suggestedParam="PASSWORD"
          onChange={(password) => set({ password })}
        />
      </>
    );
  }
  if (auth.type === "BEARER_TOKEN") {
    return (
      <SecretField
        label="Token"
        value={auth.token}
        suggestedParam="BEARER_TOKEN"
        onChange={(token) => set({ token })}
      />
    );
  }
  if (auth.type === "API_KEY") {
    return (
      <>
        {text("key", "Key name", "X-API-Key")}
        <Field label="Sent in">
          <Choice
            ariaLabel="API key location"
            value={String(auth.location ?? "HEADER")}
            onChange={(location) => set({ location })}
            options={[
              { value: "HEADER", label: "A header" },
              { value: "QUERY", label: "The query string" },
            ]}
          />
        </Field>
        <SecretField
          label="Key value"
          value={auth.value}
          suggestedParam="API_KEY"
          onChange={(value) => set({ value })}
        />
      </>
    );
  }
  if (auth.type === "OAUTH2") {
    const grant = (auth.grantType ?? { type: "CLIENT_CREDENTIALS" }) as Record<string, unknown>;
    return (
      <>
        {text("tokenUrl", "Token URL", "https://idp.example.org/token")}
        {text("clientId", "Client id", "client-id or {{__CLIENT_ID__}}")}
        <SecretField
          label="Client secret"
          value={auth.clientSecret}
          suggestedParam="CLIENT_SECRET"
          onChange={(clientSecret) => set({ clientSecret })}
        />
        <Field label="Grant">
          <Choice
            ariaLabel="Grant type"
            value={String(grant.type)}
            onChange={(type) => set({ grantType: { type } })}
            options={[
              { value: "CLIENT_CREDENTIALS", label: "Client credentials" },
              { value: "PASSWORD", label: "Password" },
              { value: "AUTHORIZATION_CODE", label: "Authorization code" },
            ]}
          />
        </Field>
        <Field label="When the token expires">
          <Choice
            ariaLabel="On token expire"
            value={String(auth.onTokenExpire ?? "REFETCH")}
            onChange={(onTokenExpire) => set({ onTokenExpire })}
            options={[
              { value: "REFETCH", label: "Ask for a new one" },
              { value: "REFRESH_OR_REFETCH", label: "Refresh, else ask again" },
            ]}
          />
        </Field>
        {grant.type === "PASSWORD" && (
          <>
            <Field label="Username">
              <Input
                className="h-8 font-mono text-xs"
                value={String(grant.username ?? "")}
                onChange={(e) => set({ grantType: { ...grant, username: e.target.value } })}
              />
            </Field>
            <SecretField
              label="Password"
              value={grant.password}
              onChange={(password) => set({ grantType: { ...grant, password } })}
            />
          </>
        )}
        <Field label="Scopes" className="md:col-span-2">
          {isTemplate(auth.scopes) ? (
            <Input
              className="h-8 font-mono text-xs"
              value={auth.scopes}
              onChange={(e) => set({ scopes: e.target.value })}
            />
          ) : (
            <TagInput
              value={(auth.scopes as string[] | undefined) ?? []}
              onChange={(scopes) => set({ scopes })}
              placeholder="Enter after each scope"
            />
          )}
        </Field>
      </>
    );
  }
  return null;
}

function defaultPlaceholder(type: ParamType): string {
  switch (type) {
    case "INT":
      return "Default integer, e.g. 30";
    case "BOOLEAN":
      return "true or false (optional)";
    case "VEC<STRING>":
      return 'JSON array, e.g. ["a", "b"]';
    case "MAP<STRING,STRING>":
      return 'JSON object, e.g. {"Accept": "application/json"}';
    default:
      return "Default (optional)";
  }
}

function ParameterRows({
  body,
  onChange,
}: {
  body: TemplateBody;
  onChange: (body: TemplateBody) => void;
}) {
  const used = new Map(Model.usedPlaceholders(body).map((u) => [u.name, u]));
  const patch = (index: number, change: Partial<ConnectorParameterDefinition>) =>
    onChange({
      ...body,
      parameters: body.parameters.map((p, i) => (i === index ? { ...p, ...change } : p)),
    });

  if (body.parameters.length === 0) {
    return (
      <p className="text-xs text-ink/55">
        Nothing is asked on instantiation yet. Write{" "}
        <code className="font-mono">{"{{__NAME__}}"}</code> in any field above and declare it from
        the checks.
      </p>
    );
  }

  return (
    <div className="space-y-2">
      {body.parameters.map((p, index) => {
        const usage = used.get(p.name);
        const defaultErr = validateParameterDefaultValue(p.paramType, p.defaultValue);
        const token = p.name ? `{{__${p.name}__}}` : null;
        return (
          <div key={index} className="rounded-md border border-ink/10 p-2">
            <div className="grid grid-cols-1 gap-1.5 md:grid-cols-[11rem_1fr_9rem_auto_auto]">
              <Input
                aria-label="Placeholder name"
                className="h-8 font-mono text-xs"
                placeholder="NAME"
                value={p.name}
                onChange={(e) =>
                  onChange(
                    Model.renameParameter(
                      body,
                      index,
                      e.target.value.toUpperCase().replace(/[^A-Z0-9_]/g, "_"),
                    ),
                  )
                }
              />
              <Input
                aria-label="Title"
                className="h-8 text-xs"
                placeholder="Label shown when instantiating"
                value={p.title ?? ""}
                onChange={(e) => patch(index, { title: e.target.value })}
              />
              <Choice
                ariaLabel="Type"
                value={p.paramType}
                onChange={(paramType) => patch(index, { paramType: paramType as ParamType })}
                options={PARAM_TYPES}
              />
              <label className="flex h-8 items-center gap-1.5 text-xs text-ink/70">
                <Checkbox
                  checked={p.required}
                  onCheckedChange={(c) => patch(index, { required: c === true })}
                />
                Required
              </label>
              <Button
                type="button"
                size="icon_sm"
                variant="ghost"
                title="Remove parameter"
                onClick={() => onChange(Model.removeParameter(body, p.name))}
              >
                <Trash2 />
              </Button>
            </div>
            <div className="mt-1.5 grid grid-cols-1 gap-1.5 md:grid-cols-[14rem_1fr]">
              <div>
                <Input
                  aria-label="Default value"
                  className={cn("h-8 font-mono text-xs", defaultErr && "border-danger-500/60")}
                  placeholder={defaultPlaceholder(p.paramType)}
                  value={p.defaultValue ?? ""}
                  onChange={(e) => patch(index, { defaultValue: e.target.value || undefined })}
                />
                {defaultErr && (
                  <p className="mt-0.5 text-[11px] text-danger-600 dark:text-danger-400">
                    {defaultErr}
                  </p>
                )}
                {p.paramType === "BOOLEAN" && (
                  <div className="mt-1 flex items-center gap-1 text-[11px]">
                    {(["true", "false"] as const).map((boolVal) => (
                      <button
                        key={boolVal}
                        type="button"
                        onClick={() =>
                          patch(index, {
                            defaultValue: p.defaultValue === boolVal ? undefined : boolVal,
                          })
                        }
                        className={cn(
                          "rounded border px-1.5 py-0.5 font-mono transition-colors",
                          p.defaultValue === boolVal
                            ? "border-brand-sky/40 bg-brand-sky/10 text-brand-sky"
                            : "border-ink/10 text-ink/55 hover:text-ink",
                        )}
                      >
                        {boolVal}
                      </button>
                    ))}
                  </div>
                )}
              </div>
              <Input
                aria-label="Description"
                className="h-8 text-xs"
                placeholder="Help text shown when instantiating (optional)"
                value={p.description ?? ""}
                onChange={(e) => patch(index, { description: e.target.value || undefined })}
              />
            </div>
            <div className="mt-1.5 flex flex-wrap items-center justify-between gap-2 text-xs">
              <span
                className={
                  usage
                    ? "text-ink/50"
                    : p.name
                      ? "text-warn-700 dark:text-warn-400"
                      : "text-ink/40"
                }
              >
                {usage
                  ? `Used in: ${usage.where.join(", ")}`
                  : p.name
                    ? "Not referenced in any template field yet"
                    : "Enter a parameter name"}
              </span>
              {token && (
                <button
                  type="button"
                  onClick={() => {
                    navigator.clipboard.writeText(token);
                    toast.success(`Copied ${token}`);
                  }}
                  className="inline-flex items-center gap-1 rounded border border-ink/10 px-1.5 py-0.5 font-mono text-[11px] text-ink/70 hover:border-ink/25 hover:text-ink"
                  title="Copy placeholder token"
                >
                  {token}
                  <Copy className="h-2.5 w-2.5" />
                </button>
              )}
            </div>
          </div>
        );
      })}
    </div>
  );
}

export function ConnectorTemplateBuilder({
  value,
  onChange,
}: {
  value: TemplateBody;
  onChange: (value: TemplateBody) => void;
}) {
  const interaction = Model.interaction(value);
  const auth = Model.auth(value);
  const mode = Model.mode(value);
  const protocol = Model.protocol(value);
  const main = Model.mainStep(value) ?? Model.httpStep();
  const setMain = (spec: StepSpec) =>
    onChange(
      Model.withInteraction(value, mode === "PULL" ? { dataAccess: spec } : { subscribe: spec }),
    );

  return (
    <div className="space-y-3">
      <Section title="How the data flows" hint={MODE_HELP[mode]}>
        <div className="flex flex-wrap items-end gap-x-5 gap-y-3">
          <Field label="Mode">
            <SegmentedControl
              ariaLabel="Mode"
              value={mode}
              onChange={(m) => onChange(Model.withMode(value, m))}
              options={MODES}
            />
          </Field>
          <Field label="Back-end">
            <SegmentedControl<Protocol>
              ariaLabel="Protocol"
              value={protocol}
              onChange={(p) => onChange(Model.withProtocol(value, p))}
              options={
                mode === "PULL"
                  ? [{ value: "HTTP", label: "HTTP" }]
                  : [
                      { value: "HTTP", label: "HTTP webhook" },
                      { value: "KAFKA", label: "Kafka topic" },
                    ]
              }
            />
          </Field>
          <Field label="Transfer ends">
            <SegmentedControl
              ariaLabel="Lifetime"
              value={interaction.lifetime ?? "INFINITE"}
              onChange={(lifetime) => onChange(Model.withInteraction(value, { lifetime }))}
              options={[
                { value: "INFINITE", label: "When someone stops it" },
                { value: "FINITE", label: "After one delivery" },
              ]}
            />
          </Field>
          <Field label="Retries">
            <Input
              type="number"
              min={0}
              max={20}
              className="h-8 w-20 text-xs"
              value={interaction.maxRetries ?? ""}
              placeholder="3"
              onChange={(e) =>
                onChange(
                  Model.withInteraction(value, {
                    maxRetries: e.target.value === "" ? undefined : Number(e.target.value),
                  }),
                )
              }
            />
          </Field>
        </div>
      </Section>

      <Section
        title={mode === "PULL" ? "Data access" : "Subscribe"}
        hint={
          mode === "PULL"
            ? "The request the dataplane makes to your back-end for each consumer request."
            : protocol === "KAFKA"
              ? "The topic the dataplane consumes while the transfer is running."
              : "The request that asks your back-end to start sending, made when a transfer starts."
        }
      >
        {protocol === "KAFKA" ? (
          <KafkaStep spec={main} onChange={setMain} />
        ) : (
          <HttpStep
            spec={main}
            stepRole={mode === "PULL" ? "access" : "subscribe"}
            onChange={setMain}
            withBody={mode === "PUSH"}
          />
        )}
      </Section>

      {mode === "PUSH" && protocol === "HTTP" && (
        <Section
          title="Unsubscribe"
          hint="The request that asks your back-end to stop, made when the transfer ends."
          action={
            <label className="flex shrink-0 items-center gap-1.5 text-xs text-ink/70">
              <Checkbox
                checked={!!interaction.unsubscribe}
                onCheckedChange={(c) =>
                  onChange(
                    Model.withInteraction(value, {
                      unsubscribe: c === true ? Model.httpStep("DELETE") : undefined,
                    }),
                  )
                }
              />
              Needed
            </label>
          }
        >
          {interaction.unsubscribe ? (
            <HttpStep
              spec={interaction.unsubscribe}
              stepRole="unsubscribe"
              onChange={(unsubscribe) => onChange(Model.withInteraction(value, { unsubscribe }))}
              withBody
            />
          ) : (
            <p className="text-xs text-ink/55">
              Not needed: the back-end stops by itself or the subscription expires.
            </p>
          )}
        </Section>
      )}

      <Section title="Authentication" hint="How the dataplane identifies itself to your back-end.">
        <div className="grid grid-cols-1 gap-3 md:grid-cols-2">
          <Field
            label="Method"
            hint={
              protocol === "KAFKA"
                ? "Kafka SASL pairs Basic with PLAIN/SCRAM, and Bearer or OAuth 2.0 with OAUTHBEARER."
                : undefined
            }
          >
            <Choice
              ariaLabel="Authentication method"
              value={auth.type ?? "NO_AUTH"}
              onChange={(type) => onChange({ ...value, authentication: Model.authOfType(type) })}
              options={AUTH_TYPES.map((o) => ({
                ...o,
                disabled: protocol === "KAFKA" && o.value === "API_KEY" && auth.type !== "API_KEY",
              }))}
            />
          </Field>
          <span className="hidden md:block" />
          <AuthFields
            auth={auth}
            onChange={(next) =>
              onChange({
                ...value,
                authentication: next as unknown as TemplateBody["authentication"],
              })
            }
          />
        </div>
      </Section>

      <Section
        title="Parameters"
        hint="What each distribution answers when it instantiates this template."
        action={
          <Button
            type="button"
            size="xs"
            variant="outline"
            onClick={() =>
              onChange({
                ...value,
                parameters: [
                  ...value.parameters,
                  { name: "", title: "", paramType: "STRING", required: true },
                ],
              })
            }
          >
            <Plus /> Add parameter
          </Button>
        }
      >
        <ParameterRows body={value} onChange={onChange} />
      </Section>
    </div>
  );
}
