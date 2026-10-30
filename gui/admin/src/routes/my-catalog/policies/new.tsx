import React, { useEffect, useMemo, useState } from "react";
import { createFileRoute, useNavigate } from "@tanstack/react-router";
import { useQueryClient } from "@tanstack/react-query";
import { toast } from "sonner";
import { cn } from "shared/src/lib/utils";
import { PageLayout } from "shared/src/components/layout/PageLayout";
import { PageHeader } from "shared/src/components/layout/PageHeader";
import { PageSection } from "shared/src/components/layout/PageSection";
import { Button } from "shared/src/components/ui/button";
import { Input } from "shared/src/components/ui/input";
import { SegmentedControl } from "shared/src/components/ui/segmented";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "shared/src/components/ui/select";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "shared/src/components/ui/tabs";
import { JsonViewer } from "shared/src/components/json/JsonViewer";
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
  PolicyPreviewDto,
  PolicyTemplateDto,
  PolicyTemplateKind,
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
  const [values, setValues] = useState<ParameterValues>({});
  const [preview, setPreview] = useState<PolicyPreviewDto | null>(null);
  const [previewError, setPreviewError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  const [loaded, setLoaded] = useState(false);

  // A new version starts from the old one, its pieces looked up in the library once it is in.
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
    setTree(PolicyTreeOps.parse(base.content, k, library));
    setLoaded(true);
  }, [base, templates, loaded]);

  const content = useMemo(() => PolicyTreeOps.content(tree, kind), [tree, kind]);
  const parameters = useMemo(() => PolicyTreeOps.parameters(tree), [tree]);
  const treeProblem = PolicyTreeOps.problems(tree, kind)[0];
  const problem = !id.trim()
    ? "Give it an id."
    : !version.trim()
      ? "Give it a version."
      : treeProblem;
  const missing = PolicyTemplateApi.missing(parameters, values);

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
    if (treeProblem || missing.length > 0) {
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
    // The draft is rebuilt each render; its inputs are what matter.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [content, parameters, kind, points, values, treeProblem, missing.length]);

  const save = async () => {
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
      toast.error((e as Error).message);
    } finally {
      setSaving(false);
    }
  };

  return (
    <PageLayout>
      <PageHeader
        eyebrow="Policy library"
        title={base ? `New version of ${PolicyTemplateApi.title(base)}` : "New template"}
        description="Drag pieces from the library and blocks onto the canvas. Mark a value as asked and it becomes a parameter of this template."
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
            <Button onClick={save} disabled={!!problem} isLoading={saving} title={problem}>
              Save
            </Button>
          </>
        }
      />

      <div className="mb-6 grid grid-cols-1 gap-3 md:grid-cols-6">
        <label className="flex flex-col gap-1 md:col-span-2">
          <span className="text-xs font-medium">Id</span>
          <Input
            value={id}
            onChange={(e) => setId(e.target.value)}
            disabled={!!base}
            placeholder="research-in-spain"
          />
        </label>
        <label className="flex flex-col gap-1">
          <span className="text-xs font-medium">Version</span>
          <Input value={version} onChange={(e) => setVersion(e.target.value)} placeholder="1.0" />
        </label>
        <label className="flex flex-col gap-1 md:col-span-2">
          <span className="text-xs font-medium">Title</span>
          <Input value={title} onChange={(e) => setTitle(e.target.value)} />
        </label>
        <label className="flex flex-col gap-1">
          <span className="text-xs font-medium">Visibility</span>
          <Select value={visibility} onValueChange={(v) => setVisibility(v as Visibility)}>
            <SelectTrigger className="h-9">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="Private">Private</SelectItem>
              <SelectItem value="Public">Public</SelectItem>
            </SelectContent>
          </Select>
        </label>
        <label className="flex flex-col gap-1 md:col-span-3">
          <span className="text-xs font-medium">Description</span>
          <Input value={description} onChange={(e) => setDescription(e.target.value)} />
        </label>
        <div className="flex flex-col gap-1">
          <span className="text-xs font-medium">Kind</span>
          <SegmentedControl value={kind} onChange={setKind} options={KINDS} ariaLabel="Kind" />
        </div>
        <div className="flex flex-col gap-1 md:col-span-2">
          <span className="text-xs font-medium">
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

      <div className="grid grid-cols-1 gap-6 2xl:grid-cols-[minmax(0,1fr)_26rem]">
        <PolicyBuilder
          mode={kind}
          tree={tree}
          onChange={setTree}
          templates={templates}
          self={base?.id}
        />

        <div className="space-y-5">
          {Object.keys(parameters).length > 0 && (
            <PageSection title="Parameters of this template">
              <p className="mb-3 text-xs text-ink/55">
                What is asked when the template is used; the values here are only for the preview.
              </p>
              <TemplateParameterFields
                parameters={parameters}
                values={values}
                onChange={setValues}
              />
            </PageSection>
          )}
          <PageSection title="Preview">
            {(treeProblem || previewError) && (
              <p className="mb-3 rounded-md border border-ink/10 bg-ink/[0.03] px-3 py-2 text-xs text-ink/70">
                {treeProblem ?? previewError}
              </p>
            )}
            {!preview && !treeProblem && !previewError && (
              <p className="text-sm text-ink/55">
                {missing.length > 0
                  ? `Give sample values to ${missing.join(", ")}.`
                  : "Flattening…"}
              </p>
            )}
            <Tabs defaultValue="flat">
              <TabsList>
                <TabsTrigger value="flat">Flattened</TabsTrigger>
                <TabsTrigger value="content">Template JSON</TabsTrigger>
              </TabsList>
              <TabsContent value="flat" className="space-y-3 pt-4">
                {preview &&
                  (kind === "policy" ? (
                    <OdrlRulesView policy={preview.odrl} />
                  ) : (
                    <JsonViewer value={preview.odrl} />
                  ))}
                {preview && <ControlPointChips points={preview.controlPoints} />}
              </TabsContent>
              <TabsContent value="content" className="pt-4">
                <JsonViewer value={{ content, parameters }} />
              </TabsContent>
            </Tabs>
          </PageSection>
        </div>
      </div>
    </PageLayout>
  );
}
