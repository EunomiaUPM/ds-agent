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
import { AlertTriangle, CheckCircle2, Copy, Wand2, XCircle } from "lucide-react";
import { toast } from "sonner";
import {
  Button,
  cn,
  Input,
  PageHeader,
  PageLayout,
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
  Textarea,
} from "@eunomia/ui";
import { ConnectorTemplateBuilder } from "shared/src/components/connector/ConnectorTemplateBuilder";
import {
  ConnectorTemplateModel,
  type TemplateBody,
  type TemplateIssue,
  type TemplatePreset,
} from "shared/src/components/connector/connectorTemplateModel";
import {
  getGetConnectorTemplateQueryKey,
  getGetConnectorTemplatesQueryKey,
  useCreateConnectorTemplate,
  useGetConnectorTemplateByNameVersion,
} from "shared/src/data/orval/connector-templates/connector-templates";
import type { ConnectorTemplateDto } from "shared/src/data/orval/model";

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

const PRESETS = ConnectorTemplateModel.PRESETS;
const serialize = (body: TemplateBody) => JSON.stringify(body, null, 2);

function bumpVersion(v?: string | null): string {
  if (!v) return "1.0.0";
  const m = v.match(/^(.*?)(\d+)$/);
  return m ? `${m[1]}${Number(m[2]) + 1}` : `${v}.1`;
}

/** What is left to fix before saving, each with its one-click fix when there is one. */
function Checks({
  issues,
  serverError,
  onFix,
}: {
  issues: TemplateIssue[];
  serverError: string | null;
  onFix: (issue: TemplateIssue) => void;
}) {
  const errors = issues.filter((i) => i.level === "error").length;
  return (
    <section className="rounded-lg border border-ink/10">
      <header className="flex items-center gap-2 border-b border-ink/[0.07] px-3 py-2 text-sm font-semibold">
        {errors === 0 ? (
          <>
            <CheckCircle2 className="h-4 w-4 text-success-600" /> Ready to save
          </>
        ) : (
          <>
            <XCircle className="h-4 w-4 text-danger-600" />
            {errors} {errors === 1 ? "thing" : "things"} to fix
          </>
        )}
      </header>
      {(issues.length > 0 || serverError) && (
        <ul className="divide-y divide-ink/[0.06]">
          {serverError && (
            <li className="flex items-start gap-2 bg-danger-500/[0.06] px-3 py-2 text-xs text-danger-700 dark:text-danger-300">
              <XCircle className="mt-0.5 h-3.5 w-3.5 shrink-0" />
              <span className="min-w-0 break-words">The agent refused it: {serverError}</span>
            </li>
          )}
          {issues.map((issue) => (
            <li key={issue.id} className="flex items-start gap-2 px-3 py-1.5 text-xs">
              {issue.level === "error" ? (
                <XCircle className="mt-0.5 h-3.5 w-3.5 shrink-0 text-danger-600" />
              ) : (
                <AlertTriangle className="mt-0.5 h-3.5 w-3.5 shrink-0 text-warn-600" />
              )}
              <span className="min-w-0 flex-1 break-words text-ink/80">{issue.message}</span>
              {issue.fix && (
                <Button
                  type="button"
                  size="xs"
                  variant="outline"
                  className="h-6 shrink-0"
                  onClick={() => onFix(issue)}
                >
                  {issue.fix.label}
                </Button>
              )}
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}

function PlaceholderRow({ token, note }: { token: string; note: string }) {
  return (
    <li className="flex items-start justify-between gap-2 px-3 py-1.5 text-xs">
      <div className="min-w-0 flex-1">
        <code className="block break-all font-mono text-ink/85">{token}</code>
        <span className="block text-[11px] text-ink/50">{note}</span>
      </div>
      <Button
        type="button"
        size="icon_sm"
        variant="ghost"
        className="h-6 w-6 shrink-0"
        title="Copy"
        onClick={() => {
          navigator.clipboard.writeText(token);
          toast.success("Copied");
        }}
      >
        <Copy />
      </Button>
    </li>
  );
}

function NewConnectorTemplatePage() {
  const { from, version: fromVersion } = Route.useSearch();
  const navigate = useNavigate();
  const queryClient = useQueryClient();

  const { data: baseRes } = useGetConnectorTemplateByNameVersion(from ?? "", fromVersion ?? "", {
    query: { enabled: !!(from && fromVersion) },
  });
  const base = baseRes?.status === 200 ? baseRes.data : undefined;

  const [presetId, setPresetId] = useState<string | undefined>(PRESETS[0].id);
  const [name, setName] = useState(PRESETS[0].name);
  const [version, setVersion] = useState("1.0.0");
  const [author, setAuthor] = useState("");
  const [description, setDescription] = useState(PRESETS[0].description);
  // The JSON text is the one copy of the body: the form writes it and the JSON tab edits it.
  const [jsonText, setJsonText] = useState(() => serialize(PRESETS[0].body));
  const [loadedFromBase, setLoadedFromBase] = useState(false);
  const [serverError, setServerError] = useState<string | null>(null);

  useEffect(() => {
    if (!base || loadedFromBase) return;
    setName(base.name ?? from ?? "");
    setVersion(bumpVersion(base.version));
    setAuthor(base.author ?? "");
    setDescription(base.description ?? "");
    setJsonText(
      serialize({
        authentication: base.authentication,
        interaction: base.interaction,
        parameters: base.parameters ?? [],
      }),
    );
    setPresetId(undefined);
    setLoadedFromBase(true);
  }, [base, from, loadedFromBase]);

  const parsed = useMemo<{ value?: TemplateBody; error?: string }>(() => {
    try {
      const obj = JSON.parse(jsonText) as Partial<TemplateBody>;
      if (!obj || typeof obj !== "object" || !obj.authentication || !obj.interaction) {
        return { error: "The JSON needs `authentication`, `interaction` and `parameters`." };
      }
      return {
        value: {
          authentication: obj.authentication,
          interaction: obj.interaction,
          parameters: Array.isArray(obj.parameters) ? obj.parameters : [],
        },
      };
    } catch (e) {
      return { error: e instanceof Error ? e.message : "Invalid JSON." };
    }
  }, [jsonText]);

  const setBody = (body: TemplateBody) => {
    setJsonText(serialize(body));
    setServerError(null);
  };

  const applyPreset = (preset: TemplatePreset) => {
    if (!base) {
      setName(preset.name);
      setDescription(preset.description);
    }
    setPresetId(preset.id);
    setBody(preset.body);
  };

  const issues = useMemo<TemplateIssue[]>(() => {
    if (!parsed.value) {
      return [{ id: "json", level: "error", message: `JSON: ${parsed.error}` }];
    }
    const list = ConnectorTemplateModel.issues(name, version, parsed.value);
    if (base && version.trim() && version.trim() === base.version) {
      list.unshift({
        id: "version-duplicate",
        level: "error",
        message: `Version ${base.version} already exists; pick a new version number.`,
        fix: {
          label: `Use ${bumpVersion(base.version)}`,
          apply: (b) => {
            setVersion(bumpVersion(base.version));
            return b;
          },
        },
      });
    }
    return list;
  }, [parsed, name, version, base]);
  const blocked = issues.some((i) => i.level === "error");

  const createMutation = useCreateConnectorTemplate();

  const fullDto: ConnectorTemplateDto | undefined = parsed.value
    ? {
        name: name.trim(),
        version: version.trim(),
        author: author.trim() || undefined,
        description: description.trim() || undefined,
        ...parsed.value,
      }
    : undefined;

  const handleSave = async () => {
    if (!fullDto) return;
    setServerError(null);
    try {
      const res = await createMutation.mutateAsync({ data: fullDto });
      if (res.status !== 200) {
        const refusal = res.data as { details?: string; message?: string } | undefined;
        setServerError(refusal?.details || refusal?.message || "Template validation failed.");
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
      setServerError(err instanceof Error ? err.message : "Failed to save connector template.");
    }
  };

  return (
    <PageLayout>
      <PageHeader
        eyebrow="Connector templates"
        title={base ? `New version of ${base.name}` : "New connector template"}
        description="Describe once how the dataplane reaches a back-end; each distribution then only fills in the parameters."
        actions={
          <>
            <Button
              variant="outline"
              onClick={() =>
                navigate(
                  from
                    ? { to: "/my-catalog/connectors/$templateName", params: { templateName: from } }
                    : { to: "/my-catalog/connectors" },
                )
              }
            >
              Cancel
            </Button>
            <Button onClick={handleSave} disabled={blocked} isLoading={createMutation.isPending}>
              Save template
            </Button>
          </>
        }
      />

      <div className="grid grid-cols-1 gap-4 xl:grid-cols-[minmax(0,1fr)_26rem]">
        <div className="min-w-0 space-y-3">
          {!base && (
            <div className="grid grid-cols-2 gap-2 lg:grid-cols-4">
              {PRESETS.map((p) => (
                <button
                  key={p.id}
                  type="button"
                  onClick={() => applyPreset(p)}
                  aria-pressed={presetId === p.id}
                  className={cn(
                    "rounded-lg border px-3 py-2 text-left transition-colors",
                    presetId === p.id
                      ? "border-brand-sky/50 bg-brand-sky/[0.06]"
                      : "border-ink/10 hover:border-ink/25",
                  )}
                >
                  <span className="block text-sm font-medium text-ink/90">{p.label}</span>
                  <span className="block font-mono text-xs text-ink/50">
                    {p.mode} · {p.protocol}
                  </span>
                </button>
              ))}
            </div>
          )}

          <section className="grid grid-cols-1 gap-3 rounded-lg border border-ink/10 p-3 md:grid-cols-6">
            <label className="flex flex-col gap-1 md:col-span-3">
              <span className="text-xs font-medium text-ink/80">Name</span>
              <Input
                className="h-8 font-mono text-xs"
                value={name}
                onChange={(e) => setName(e.target.value)}
                disabled={!!base}
                placeholder="orders-api"
              />
            </label>
            <label className="flex flex-col gap-1">
              <span className="text-xs font-medium text-ink/80">Version</span>
              <Input
                className="h-8 font-mono text-xs"
                value={version}
                onChange={(e) => setVersion(e.target.value)}
                placeholder="1.0.0"
              />
            </label>
            <label className="flex flex-col gap-1 md:col-span-2">
              <span className="text-xs font-medium text-ink/80">Author</span>
              <Input
                className="h-8 text-xs"
                value={author}
                onChange={(e) => setAuthor(e.target.value)}
                placeholder="Optional"
              />
            </label>
            <label className="flex flex-col gap-1 md:col-span-6">
              <span className="text-xs font-medium text-ink/80">Description</span>
              <Input
                className="h-8 text-xs"
                value={description}
                onChange={(e) => setDescription(e.target.value)}
                placeholder="Which back-end this template reaches"
              />
            </label>
          </section>

          {parsed.value ? (
            <ConnectorTemplateBuilder value={parsed.value} onChange={setBody} />
          ) : (
            <p className="rounded-lg border border-dashed border-danger-500/40 px-3 py-6 text-center text-sm text-ink/60">
              The JSON is not valid, so the form cannot show it. Fix it in the JSON tab.
            </p>
          )}
        </div>

        <aside className="min-w-0 space-y-3 xl:sticky xl:top-20 xl:max-h-[calc(100dvh-6rem)] xl:self-start xl:overflow-y-auto">
          <Checks
            issues={issues}
            serverError={serverError}
            onFix={(issue) => parsed.value && issue.fix && setBody(issue.fix.apply(parsed.value))}
          />

          <Tabs defaultValue="placeholders">
            <TabsList>
              <TabsTrigger value="placeholders">Placeholders</TabsTrigger>
              <TabsTrigger value="form">Instantiation</TabsTrigger>
              <TabsTrigger value="json">JSON</TabsTrigger>
            </TabsList>
            <TabsContent value="placeholders" className="pt-3">
              <p className="mb-2 text-xs text-ink/55">
                Paste a placeholder into any field. User parameters are filled on instantiation;
                built-ins are resolved by the agent.
              </p>
              <ul className="divide-y divide-ink/[0.06] rounded-lg border border-ink/10">
                {(parsed.value?.parameters ?? [])
                  .filter((p) => p.name)
                  .map((p) => (
                    <PlaceholderRow
                      key={p.name}
                      token={`{{__${p.name}__}}`}
                      note={p.title || p.paramType}
                    />
                  ))}
                {ConnectorTemplateModel.BUILT_IN.map((b) => (
                  <PlaceholderRow key={b.token} token={b.token} note={b.what} />
                ))}
              </ul>
            </TabsContent>
            <TabsContent value="form" className="pt-3">
              <p className="mb-2 text-xs text-ink/55">
                Preview of what distribution publishers will be asked when instantiating this
                template.
              </p>
              {(parsed.value?.parameters ?? []).length === 0 ? (
                <p className="rounded-lg border border-dashed border-ink/15 px-3 py-5 text-center text-xs text-ink/50">
                  No instantiation inputs — bindings will use the template values directly.
                </p>
              ) : (
                <div className="space-y-2 rounded-lg border border-ink/10 p-3">
                  {(parsed.value?.parameters ?? []).map((p, idx) => (
                    <div
                      key={`${p.name}-${idx}`}
                      className="space-y-1 border-b border-ink/[0.06] pb-2 last:border-b-0 last:pb-0"
                    >
                      <div className="flex items-center justify-between gap-2 text-xs">
                        <span className="font-medium text-ink/90">
                          {p.title || p.name || `Parameter ${idx + 1}`}
                          {p.required && <span className="ml-1 text-danger-600">*</span>}
                        </span>
                        <code className="font-mono text-[11px] text-ink/50">{p.paramType}</code>
                      </div>
                      {p.description && <p className="text-[11px] text-ink/55">{p.description}</p>}
                      <div className="rounded border border-ink/10 bg-ink/[0.02] px-2 py-1 font-mono text-[11px] text-ink/55">
                        {p.defaultValue !== undefined
                          ? `Default: ${p.defaultValue}`
                          : p.name
                            ? `Fills {{__${p.name}__}}`
                            : "Unset"}
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </TabsContent>
            <TabsContent value="json" className="pt-3">
              <div className="mb-1.5 flex items-center justify-between">
                <span className="text-xs text-ink/55">The same template, editable as JSON.</span>
                <Button
                  type="button"
                  size="xs"
                  variant="ghost"
                  disabled={!parsed.value}
                  onClick={() => parsed.value && setJsonText(serialize(parsed.value))}
                >
                  <Wand2 /> Format
                </Button>
              </div>
              <Textarea
                rows={28}
                value={jsonText}
                onChange={(e) => {
                  setJsonText(e.target.value);
                  setPresetId(undefined);
                  setServerError(null);
                }}
                className="font-mono text-xs leading-relaxed"
                spellCheck={false}
              />
              {parsed.error && (
                <p className="mt-1.5 text-xs text-danger-600 dark:text-danger-400">
                  {parsed.error}
                </p>
              )}
            </TabsContent>
          </Tabs>
        </aside>
      </div>
    </PageLayout>
  );
}
