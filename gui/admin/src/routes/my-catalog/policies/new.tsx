import React, { useEffect, useMemo, useState } from "react";
import { createFileRoute, useNavigate } from "@tanstack/react-router";
import { useQueryClient } from "@tanstack/react-query";
import { CheckCircle2, XCircle } from "lucide-react";
import { toast } from "sonner";
import {
  Button,
  cn,
  Input,
  JsonViewer,
  PageHeader,
  PageLayout,
  SegmentedControl,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
  TagInput,
} from "@eunomia/ui";
import { OdrlRulesView } from "shared/src/components/policy/OdrlRulesView";
import { ControlPointChips } from "shared/src/components/policy/ControlPointChips";
import { TemplateParameterFields } from "shared/src/components/policy/TemplateParameterFields";
import { PolicyBuilder } from "shared/src/components/policy/builder/PolicyBuilder";
import { PolicyTreeOps, type PolicyTree } from "shared/src/components/policy/builder/policyTree";
import {
  useGetAllPolicyTemplates,
  useGetPolicyTemplateByIdAndVersion,
} from "shared/src/data/orval/policy-templates/policy-templates";
import {
  CONTROL_POINTS,
  PolicyTemplateApi,
  type ParameterValues,
} from "shared/src/data/policyTemplateApi";
import type {
  ControlPoint,
  NewPolicyTemplateDto,
  PaginatedPolicyTemplateDto,
  ParameterValue,
  PolicyPreviewDto,
  PolicyTemplateDto,
  PolicyTemplateKind,
  PolicyTemplateParameterDefinition,
  PolicyTemplateParameterDefinitionDataType,
  Visibility,
} from "shared/src/data/orval/model";

/** `from` and `version` name the version a new one starts from. */
interface NewSearch {
  from?: string;
  version?: string;
}

export const Route = createFileRoute("/my-catalog/policies/new")({
  component: NewTemplatePage,
  validateSearch: (search: Record<string, unknown>): NewSearch => ({
    from: typeof search.from === "string" ? search.from : undefined,
    version: typeof search.version === "string" ? search.version : undefined,
  }),
});

const REGEX_ID = /^[a-zA-Z0-9\-:._]+$/;
const REGEX_VERSION = /^[a-zA-Z0-9.]+$/;

const PARAM_DATA_TYPES: {
  value: PolicyTemplateParameterDefinitionDataType;
  label: string;
}[] = [
  { value: "string", label: "string (text)" },
  { value: "integer", label: "integer" },
  { value: "float", label: "float" },
  { value: "boolean", label: "boolean" },
  { value: "duration", label: "duration (P30D)" },
  { value: "date", label: "date (YYYY-MM-DD)" },
  { value: "time", label: "time (HH:MM)" },
  { value: "dateTime", label: "dateTime (RFC 3339)" },
  { value: "iri", label: "iri (URI / prefix)" },
  { value: "selection", label: "selection (single)" },
  { value: "list", label: "list (multiple)" },
];

const KINDS: { value: PolicyTemplateKind; label: string; hint: string }[] = [
  { value: "policy", label: "Policy", hint: "Rule lists, possibly including other policies" },
  { value: "rule", label: "Rule", hint: "One permission, prohibition or obligation" },
  { value: "constraint", label: "Constraint", hint: "One condition, possibly a group" },
];

/** The next version of a dotted number (`1.0` → `1.1`); otherwise empty, for the user to type. */
const nextVersion = (v?: string) => {
  const m = v?.match(/^(.*?)(\d+)$/);
  return m ? `${m[1]}${Number(m[2]) + 1}` : "";
};

interface PolicyCheckIssue {
  id: string;
  message: string;
  fix?: { label: string; apply: () => void };
}

function restrictionValuesToStrings(
  raw?: PolicyTemplateParameterDefinition["restrictions"],
): string[] {
  const list = raw?.values ?? [];
  return list.map((item) => (typeof item === "string" ? item : item.value));
}

function ParameterSchemaList({
  parameters,
  onChangeDef,
  onChangeType,
}: {
  parameters: Record<string, PolicyTemplateParameterDefinition>;
  onChangeDef: (name: string, next: PolicyTemplateParameterDefinition) => void;
  onChangeType: (name: string, nextType: PolicyTemplateParameterDefinitionDataType) => void;
}) {
  const entries = Object.entries(parameters);
  if (entries.length === 0) return null;

  return (
    <div className="space-y-2.5">
      {entries.map(([name, def]) => {
        const label = PolicyTemplateApi.text(def.ui?.label) ?? "";
        const desc = PolicyTemplateApi.text(def.ui?.description) ?? "";
        const hasChoices = def.dataType === "selection" || def.dataType === "list";
        const isNumber = def.dataType === "integer" || def.dataType === "float";
        const isText = def.dataType === "string" || def.dataType === "iri";
        const choiceValues = restrictionValuesToStrings(def.restrictions);

        const patchUi = (patch: Partial<NonNullable<PolicyTemplateParameterDefinition["ui"]>>) =>
          onChangeDef(name, {
            ...def,
            ui: { ...def.ui, ...patch },
          });

        const patchRestrictions = (
          patch: Partial<NonNullable<PolicyTemplateParameterDefinition["restrictions"]>>,
        ) => {
          const next = { ...def.restrictions, ...patch };
          const cleaned = Object.fromEntries(
            Object.entries(next).filter(
              ([, v]) => v !== undefined && v !== "" && (!Array.isArray(v) || v.length > 0),
            ),
          );
          onChangeDef(name, {
            ...def,
            restrictions: Object.keys(cleaned).length ? cleaned : undefined,
          });
        };

        return (
          <div key={name} className="space-y-2 rounded-md border border-ink/10 p-2.5">
            <div className="flex flex-wrap items-center justify-between gap-2">
              <code className="rounded bg-brand-sky/10 px-1.5 py-0.5 font-mono text-xs text-brand-sky">
                {name}
              </code>
              <Select
                value={def.dataType}
                onValueChange={(v) =>
                  onChangeType(name, v as PolicyTemplateParameterDefinitionDataType)
                }
              >
                <SelectTrigger className="h-7 w-44 text-xs" aria-label={`Data type for ${name}`}>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {PARAM_DATA_TYPES.map((dt) => (
                    <SelectItem key={dt.value} value={dt.value}>
                      {dt.label}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>

            <div className="grid grid-cols-1 gap-2 sm:grid-cols-2">
              <label className="flex flex-col gap-0.5">
                <span className="text-[11px] font-medium text-ink/65">Label</span>
                <Input
                  className="h-7 text-xs"
                  placeholder={name.replace(/^\$/, "")}
                  value={label}
                  onChange={(e) => patchUi({ label: e.target.value || undefined })}
                />
              </label>
              <label className="flex flex-col gap-0.5">
                <span className="text-[11px] font-medium text-ink/65">Help text</span>
                <Input
                  className="h-7 text-xs"
                  placeholder="Optional hint"
                  value={desc}
                  onChange={(e) => patchUi({ description: e.target.value || undefined })}
                />
              </label>
            </div>

            <div className="space-y-1">
              <div className="flex items-center justify-between text-[11px]">
                <span className="font-medium text-ink/65">Default value (optional)</span>
                {def.defaultValue !== undefined && (
                  <button
                    type="button"
                    onClick={() => onChangeDef(name, { ...def, defaultValue: undefined })}
                    className="text-ink/50 hover:text-ink hover:underline"
                  >
                    Clear
                  </button>
                )}
              </div>
              <TemplateParameterFields.Field
                def={def}
                value={def.defaultValue}
                onChange={(v) =>
                  onChangeDef(name, {
                    ...def,
                    defaultValue: v === undefined || v === "" ? undefined : (v as ParameterValue),
                  })
                }
              />
            </div>

            {hasChoices && (
              <div className="space-y-1">
                <span className="block text-[11px] font-medium text-ink/65">
                  {def.dataType === "selection"
                    ? "Allowed options (Enter after each)"
                    : "Suggested values (optional)"}
                </span>
                <TagInput
                  value={choiceValues}
                  onChange={(vals) => patchRestrictions({ values: vals })}
                  placeholder="Add choice and press Enter"
                />
              </div>
            )}

            {isNumber && (
              <div className="grid grid-cols-2 gap-2">
                <label className="flex flex-col gap-0.5">
                  <span className="text-[11px] font-medium text-ink/65">Min</span>
                  <Input
                    type="number"
                    className="h-7 text-xs"
                    step={def.dataType === "integer" ? 1 : "any"}
                    value={def.restrictions?.minValue ?? ""}
                    onChange={(e) =>
                      patchRestrictions({
                        minValue: e.target.value === "" ? undefined : Number(e.target.value),
                      })
                    }
                  />
                </label>
                <label className="flex flex-col gap-0.5">
                  <span className="text-[11px] font-medium text-ink/65">Max</span>
                  <Input
                    type="number"
                    className="h-7 text-xs"
                    step={def.dataType === "integer" ? 1 : "any"}
                    value={def.restrictions?.maxValue ?? ""}
                    onChange={(e) =>
                      patchRestrictions({
                        maxValue: e.target.value === "" ? undefined : Number(e.target.value),
                      })
                    }
                  />
                </label>
              </div>
            )}

            {isText && (
              <label className="flex flex-col gap-0.5">
                <span className="text-[11px] font-medium text-ink/65">
                  Regex constraint (optional)
                </span>
                <Input
                  className="h-7 font-mono text-xs"
                  placeholder="e.g. ^[A-Z]{2}$"
                  value={def.restrictions?.regex ?? ""}
                  onChange={(e) => patchRestrictions({ regex: e.target.value || undefined })}
                />
              </label>
            )}
          </div>
        );
      })}
    </div>
  );
}

function NewTemplatePage() {
  const { from, version: fromVersion } = Route.useSearch();
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const { data: baseData } = useGetPolicyTemplateByIdAndVersion(from ?? "", fromVersion ?? "", {
    query: { enabled: !!from && !!fromVersion },
  });
  const base = baseData?.status === 200 ? (baseData.data as PolicyTemplateDto) : undefined;
  const { data: libraryData } = useGetAllPolicyTemplates({ limit: 100 });
  const templates = useMemo(() => {
    const items =
      libraryData?.status === 200 ? (libraryData.data as PaginatedPolicyTemplateDto).items : [];
    return PolicyTemplateApi.group(items).map((g) => g.latest);
  }, [libraryData]);

  const [id, setId] = useState("");
  const [version, setVersion] = useState("1.0");
  const [kind, setKind] = useState<PolicyTemplateKind>("policy");
  const [title, setTitle] = useState("");
  const [description, setDescription] = useState("");
  const [visibility, setVisibility] = useState<Visibility>("Private");
  const [points, setPoints] = useState<ControlPoint[]>([]);
  const [tree, setTree] = useState<PolicyTree>(PolicyTreeOps.empty);
  const [paramOverrides, setParamOverrides] = useState<
    Record<string, PolicyTemplateParameterDefinition>
  >({});
  const [values, setValues] = useState<ParameterValues>({});
  const [preview, setPreview] = useState<PolicyPreviewDto | null>(null);
  const [previewError, setPreviewError] = useState<string | null>(null);
  const [serverError, setServerError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  const [loaded, setLoaded] = useState(false);

  // A new version starts from the old one, preserving its custom parameter definitions too.
  useEffect(() => {
    if (!base || loaded || templates.length === 0) return;
    const library = new Map(templates.map((t) => [PolicyTemplateApi.reference(t), t]));
    const k = base.kind ?? "policy";
    setId(base.id);
    setVersion(nextVersion(base.version));
    setKind(k);
    setTitle(PolicyTemplateApi.text(base.title) ?? "");
    setDescription(PolicyTemplateApi.text(base.description) ?? "");
    setVisibility(base.visibility);
    setPoints(base.controlPoints ?? []);
    setParamOverrides(base.parameters ?? {});
    setTree(PolicyTreeOps.parse(base.content, k, library));
    setLoaded(true);
  }, [base, templates, loaded]);

  const content = useMemo(() => PolicyTreeOps.content(tree, kind), [tree, kind]);
  const parameters = useMemo(
    () => PolicyTreeOps.parameters(tree, paramOverrides),
    [tree, paramOverrides],
  );
  const treeProblems = useMemo(() => PolicyTreeOps.problems(tree, kind), [tree, kind]);

  const issues = useMemo<PolicyCheckIssue[]>(() => {
    const out: PolicyCheckIssue[] = [];
    const trimmedId = id.trim();
    const trimmedVer = version.trim();
    if (!trimmedId) {
      out.push({ id: "id-empty", message: "Give the template an id." });
    } else if (!REGEX_ID.test(trimmedId)) {
      out.push({
        id: "id-syntax",
        message: "Id: use only letters, digits, '-', ':', '.', and '_'.",
      });
    }
    if (!trimmedVer) {
      out.push({ id: "ver-empty", message: "Give it a version (e.g. 1.0)." });
    } else if (!REGEX_VERSION.test(trimmedVer)) {
      out.push({
        id: "ver-syntax",
        message: "Version: use only letters, digits, and '.'.",
      });
    } else if (base && trimmedVer === base.version) {
      const bumped = nextVersion(base.version);
      out.push({
        id: "ver-dup",
        message: `Version ${base.version} already exists; pick a new version.`,
        fix: bumped ? { label: `Use ${bumped}`, apply: () => setVersion(bumped) } : undefined,
      });
    }
    treeProblems.forEach((msg, idx) => out.push({ id: `tree-${idx}`, message: msg }));
    for (const [pname, pdef] of Object.entries(parameters)) {
      if (
        pdef.dataType === "selection" &&
        restrictionValuesToStrings(pdef.restrictions).length === 0
      ) {
        out.push({
          id: `sel-${pname}`,
          message: `${pname}: selection type needs at least one allowed option.`,
        });
      }
    }
    return out;
  }, [id, version, base, treeProblems, parameters]);

  const blocked = issues.length > 0;
  const missing = PolicyTemplateApi.missing(parameters, values);
  const metaSyntaxInvalid =
    (!!id.trim() && !REGEX_ID.test(id.trim())) ||
    (!!version.trim() && !REGEX_VERSION.test(version.trim()));

  const draft: NewPolicyTemplateDto = {
    id: id.trim() || undefined,
    version: version.trim() || undefined,
    kind,
    controlPoints: points,
    title: title.trim() || undefined,
    description: description.trim() || undefined,
    visibility,
    content: content as NewPolicyTemplateDto["content"],
    parameters: parameters as NewPolicyTemplateDto["parameters"],
  };

  useEffect(() => {
    setValues((v) => ({ ...PolicyTemplateApi.defaults(parameters), ...v }));
  }, [parameters]);

  useEffect(() => {
    if (treeProblems.length > 0 || missing.length > 0 || metaSyntaxInvalid) {
      setPreview(null);
      setPreviewError(null);
      return;
    }
    let current = true;
    const timer = setTimeout(() => {
      PolicyTemplateApi.preview({ template: draft, parameters: PolicyTemplateApi.given(values) })
        .then((p) => current && (setPreview(p), setPreviewError(null)))
        .catch((e: Error) => current && (setPreview(null), setPreviewError(e.message)));
    }, 500);
    return () => {
      current = false;
      clearTimeout(timer);
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [
    content,
    parameters,
    kind,
    points,
    values,
    treeProblems.length,
    missing.length,
    metaSyntaxInvalid,
  ]);

  const handleParamDefChange = (name: string, next: PolicyTemplateParameterDefinition) => {
    setParamOverrides((prev) => ({ ...prev, [name]: next }));
    setServerError(null);
  };

  const handleParamTypeChange = (
    name: string,
    nextType: PolicyTemplateParameterDefinitionDataType,
  ) => {
    const prev = parameters[name] ?? { dataType: "string" };
    setParamOverrides((all) => ({
      ...all,
      [name]: {
        dataType: nextType,
        ...(prev.ui ? { ui: prev.ui } : {}),
      },
    }));
    setValues((v) => {
      const { [name]: _dropped, ...rest } = v;
      return rest;
    });
    setServerError(null);
  };

  const save = async () => {
    setServerError(null);
    setSaving(true);
    try {
      const saved = await PolicyTemplateApi.create(draft);
      await PolicyTemplateApi.invalidate(queryClient);
      toast.success(`Saved ${PolicyTemplateApi.reference(saved)}.`);
      navigate({
        to: "/my-catalog/policies/$templateId",
        params: { templateId: saved.id },
        search: { version: saved.version },
      });
    } catch (e) {
      const msg = (e as Error).message;
      setServerError(msg);
      toast.error(msg);
    } finally {
      setSaving(false);
    }
  };

  const paramCount = Object.keys(parameters).length;

  return (
    <PageLayout>
      <PageHeader
        eyebrow="Policy library"
        title={base ? `New version of ${PolicyTemplateApi.title(base)}` : "New policy template"}
        description="Compose ODRL pieces on the canvas and click Ask on any value to parameterize it."
        actions={
          <>
            <Button
              variant="outline"
              onClick={() =>
                navigate(
                  from
                    ? { to: "/my-catalog/policies/$templateId", params: { templateId: from } }
                    : { to: "/my-catalog/policies" },
                )
              }
            >
              Cancel
            </Button>
            <Button onClick={save} disabled={blocked} isLoading={saving} title={issues[0]?.message}>
              Save template
            </Button>
          </>
        }
      />

      <div className="mb-4 grid grid-cols-1 gap-3 rounded-lg border border-ink/10 p-3 md:grid-cols-6">
        <label className="flex flex-col gap-1 md:col-span-2">
          <span className="text-xs font-medium text-ink/80">Id</span>
          <Input
            className="h-8 font-mono text-xs"
            value={id}
            onChange={(e) => {
              setId(e.target.value);
              setServerError(null);
            }}
            disabled={!!base}
            placeholder="research-in-spain"
          />
        </label>
        <label className="flex flex-col gap-1">
          <span className="text-xs font-medium text-ink/80">Version</span>
          <Input
            className="h-8 font-mono text-xs"
            value={version}
            onChange={(e) => {
              setVersion(e.target.value);
              setServerError(null);
            }}
            placeholder="1.0"
          />
        </label>
        <label className="flex flex-col gap-1 md:col-span-2">
          <span className="text-xs font-medium text-ink/80">Title</span>
          <Input
            className="h-8 text-xs"
            value={title}
            onChange={(e) => setTitle(e.target.value)}
            placeholder="Optional human-readable title"
          />
        </label>
        <label className="flex flex-col gap-1">
          <span className="text-xs font-medium text-ink/80">Visibility</span>
          <Select value={visibility} onValueChange={(v) => setVisibility(v as Visibility)}>
            <SelectTrigger className="h-8 text-xs">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="Private">Private</SelectItem>
              <SelectItem value="Public">Public</SelectItem>
            </SelectContent>
          </Select>
        </label>
        <label className="flex flex-col gap-1 md:col-span-2">
          <span className="text-xs font-medium text-ink/80">Description</span>
          <Input
            className="h-8 text-xs"
            value={description}
            onChange={(e) => setDescription(e.target.value)}
            placeholder="What this template restricts or grants"
          />
        </label>
        <div className="flex flex-col gap-1 md:col-span-2">
          <span className="text-xs font-medium text-ink/80">Kind</span>
          <SegmentedControl value={kind} onChange={setKind} options={KINDS} ariaLabel="Kind" />
        </div>
        <div className="flex flex-col gap-1 md:col-span-2">
          <span className="text-xs font-medium text-ink/80">
            Meant to apply at <span className="font-normal text-ink/50">(informative)</span>
          </span>
          <div className="flex flex-wrap gap-1.5 pt-1">
            {CONTROL_POINTS.map((p) => {
              const on = points.includes(p.value);
              return (
                <button
                  key={p.value}
                  type="button"
                  title={p.hint}
                  aria-pressed={on}
                  onClick={() =>
                    setPoints((ps) => (on ? ps.filter((x) => x !== p.value) : [...ps, p.value]))
                  }
                  className={cn(
                    "rounded-full border px-2.5 py-0.5 text-xs transition-colors",
                    on
                      ? "border-brand-sky/40 bg-brand-sky/15 text-brand-sky"
                      : "border-ink/15 text-ink/60 hover:border-ink/30",
                  )}
                >
                  {p.label}
                </button>
              );
            })}
          </div>
        </div>
      </div>

      <div className="grid grid-cols-1 gap-4 2xl:grid-cols-[minmax(0,1fr)_26rem]">
        <div className="min-w-0">
          <PolicyBuilder
            mode={kind}
            tree={tree}
            onChange={setTree}
            templates={templates}
            self={base?.id}
          />
        </div>

        <aside className="min-w-0 space-y-3 2xl:sticky 2xl:top-20 2xl:max-h-[calc(100dvh-6rem)] 2xl:self-start 2xl:overflow-y-auto">
          <section className="rounded-lg border border-ink/10">
            <header className="flex items-center gap-2 border-b border-ink/[0.07] px-3 py-2 text-sm font-semibold">
              {issues.length === 0 ? (
                <>
                  <CheckCircle2 className="h-4 w-4 text-success-600" /> Ready to save
                </>
              ) : (
                <>
                  <XCircle className="h-4 w-4 text-danger-600" />
                  {issues.length} {issues.length === 1 ? "thing" : "things"} to fix
                </>
              )}
            </header>
            {(issues.length > 0 || serverError) && (
              <ul className="divide-y divide-ink/[0.06]">
                {serverError && (
                  <li className="flex items-start gap-2 bg-danger-500/[0.06] px-3 py-2 text-xs text-danger-700 dark:text-danger-300">
                    <XCircle className="mt-0.5 h-3.5 w-3.5 shrink-0" />
                    <span className="min-w-0 break-words">Agent error: {serverError}</span>
                  </li>
                )}
                {issues.map((issue) => (
                  <li key={issue.id} className="flex items-start gap-2 px-3 py-1.5 text-xs">
                    <XCircle className="mt-0.5 h-3.5 w-3.5 shrink-0 text-danger-600" />
                    <span className="min-w-0 flex-1 break-words text-ink/80">{issue.message}</span>
                    {issue.fix && (
                      <Button
                        type="button"
                        size="xs"
                        variant="outline"
                        className="h-6 shrink-0"
                        onClick={issue.fix.apply}
                      >
                        {issue.fix.label}
                      </Button>
                    )}
                  </li>
                ))}
              </ul>
            )}
          </section>

          <Tabs defaultValue="preview">
            <TabsList>
              <TabsTrigger value="preview">Preview</TabsTrigger>
              <TabsTrigger value="parameters">Parameters ({paramCount})</TabsTrigger>
              <TabsTrigger value="json">Template JSON</TabsTrigger>
            </TabsList>

            <TabsContent value="preview" className="space-y-3 pt-3">
              {paramCount > 0 && (
                <section className="rounded-lg border border-ink/10 p-3">
                  <p className="mb-2 text-xs font-medium text-ink/75">
                    Test values <span className="font-normal text-ink/50">(preview only)</span>
                  </p>
                  <TemplateParameterFields
                    parameters={parameters}
                    values={values}
                    onChange={setValues}
                  />
                </section>
              )}

              {(treeProblems[0] || previewError) && (
                <p className="rounded-md border border-danger-500/30 bg-danger-500/10 px-3 py-2 text-xs text-danger-700 dark:text-danger-300">
                  {treeProblems[0] ?? previewError}
                </p>
              )}

              {!preview && !treeProblems[0] && !previewError && (
                <p className="rounded-lg border border-dashed border-ink/15 px-3 py-4 text-center text-xs text-ink/55">
                  {missing.length > 0
                    ? `Fill test values for ${missing.join(", ")} to preview flattened ODRL.`
                    : metaSyntaxInvalid
                      ? "Fix id or version format to preview."
                      : "Flattening…"}
                </p>
              )}

              {preview && (
                <div className="space-y-2 rounded-lg border border-ink/10 p-3">
                  {kind === "policy" ? (
                    <OdrlRulesView policy={preview.odrl} />
                  ) : (
                    <JsonViewer value={preview.odrl} />
                  )}
                  <ControlPointChips points={preview.controlPoints} />
                </div>
              )}
            </TabsContent>

            <TabsContent value="parameters" className="pt-3">
              {paramCount === 0 ? (
                <p className="rounded-lg border border-dashed border-ink/15 px-3 py-5 text-center text-xs text-ink/55">
                  No template parameters yet. Click <span className="font-medium">ask</span> on any
                  condition value or exposed piece parameter to declare one.
                </p>
              ) : (
                <ParameterSchemaList
                  parameters={parameters}
                  onChangeDef={handleParamDefChange}
                  onChangeType={handleParamTypeChange}
                />
              )}
            </TabsContent>

            <TabsContent value="json" className="pt-3">
              <JsonViewer value={{ content, parameters }} />
            </TabsContent>
          </Tabs>
        </aside>
      </div>
    </PageLayout>
  );
}
