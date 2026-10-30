import React, { useEffect, useState } from "react";
import { createFileRoute, Link, useNavigate } from "@tanstack/react-router";
import { useQueryClient } from "@tanstack/react-query";
import { toast } from "sonner";
import {
  ArrowDownToLine,
  ArrowUpFromLine,
  Check,
  Copy,
  GitBranchPlus,
  History,
  Trash2,
} from "lucide-react";
import { cn } from "@eunomia/ui";
import { PageLayout } from "@eunomia/ui";
import { PageHeader } from "@eunomia/ui";
import { Button } from "@eunomia/ui";
import { Badge } from "@eunomia/ui";
import { FormatDate } from "@eunomia/ui";
import { ConfirmDialog } from "@eunomia/ui";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@eunomia/ui";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@eunomia/ui";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@eunomia/ui";
import { JsonViewer } from "@eunomia/ui";
import { Identifier } from "@eunomia/ui";
import { OdrlRulesView } from "shared/src/components/policy/OdrlRulesView";
import { ControlPointChips } from "shared/src/components/policy/ControlPointChips";
import { TemplateParameterFields } from "shared/src/components/policy/TemplateParameterFields";
import { TemplateContentView } from "shared/src/components/policy/TemplateContentView";
import {
  useGetPolicyTemplateById,
  useGetPolicyTemplateUsages,
} from "shared/src/data/orval/policy-templates/policy-templates";
import { PolicyTemplateApi, type ParameterValues } from "shared/src/data/policyTemplateApi";
import type {
  PolicyPreviewDto,
  PolicyTemplateDto,
  PolicyTemplateParameterDefinition,
  TemplateUsagesDto,
} from "shared/src/data/orval/model";

interface DetailSearch {
  version?: string;
}

const KIND_LABEL = { constraint: "Constraint", rule: "Rule", policy: "Policy" } as const;

export const Route = createFileRoute("/my-catalog/policies/$templateId")({
  component: TemplatePage,
  // A version typed by hand (`?version=1.0`) arrives as a number; versions are strings.
  validateSearch: (search: Record<string, unknown>): DetailSearch => ({
    version: search.version != null && search.version !== "" ? String(search.version) : undefined,
  }),
});

function TemplatePage() {
  const { templateId } = Route.useParams();
  const { version: chosen } = Route.useSearch();
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const [deleting, setDeleting] = useState(false);

  const { data: versionsData, isLoading } = useGetPolicyTemplateById(templateId);
  const versions =
    versionsData?.status === 200
      ? (PolicyTemplateApi.group(versionsData.data as PolicyTemplateDto[])[0]?.versions ?? [])
      : [];
  const shown = versions.find((v) => v.version === chosen) ?? versions[0];
  const { data: usagesData } = useGetPolicyTemplateUsages(templateId, shown?.version ?? "", {
    query: { enabled: !!shown },
  });
  const usages: TemplateUsagesDto =
    usagesData?.status === 200
      ? (usagesData.data as TemplateUsagesDto)
      : { templates: [], offers: [] };

  if (!shown) {
    return (
      <PageLayout>
        <p className="py-16 text-center text-sm text-ink/55">
          {isLoading ? "Loading…" : `There is no template ${templateId} you can see.`}
        </p>
      </PageLayout>
    );
  }
  const kind = shown.kind ?? "policy";
  const pieces = PolicyTemplateApi.references(shown.content);
  const blocked = usages.templates.length > 0;
  const usedBy = usages.templates.length + usages.offers.length;
  const goTo = (version: string) =>
    navigate({
      to: "/my-catalog/policies/$templateId",
      params: { templateId },
      search: { version },
    });

  const remove = async () => {
    await PolicyTemplateApi.remove(templateId, shown.version);
    await PolicyTemplateApi.invalidate(queryClient);
    toast.success(`Deleted version ${shown.version}.`);
    if (versions.length === 1) navigate({ to: "/my-catalog/policies" });
    else navigate({ to: "/my-catalog/policies/$templateId", params: { templateId }, search: {} });
  };

  return (
    <PageLayout>
      <PageHeader
        eyebrow={
          <Link to="/my-catalog/policies" className="hover:underline">
            Policy library
          </Link>
        }
        title={PolicyTemplateApi.title(shown)}
        badge={<Badge variant="outline">{KIND_LABEL[kind]}</Badge>}
        description={PolicyTemplateApi.text(shown.description)}
        actions={
          <>
            <Select value={shown.version} onValueChange={goTo}>
              <SelectTrigger className="h-9 w-40" aria-label="Version">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {versions.map((v, i) => (
                  <SelectItem key={v.version} value={v.version}>
                    v{v.version}
                    {i === 0 ? " · latest" : ""}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            <Button asChild variant="outline">
              <Link
                to="/my-catalog/policies/new"
                search={{ from: templateId, version: shown.version }}
              >
                <GitBranchPlus /> New version
              </Link>
            </Button>
            <Button
              variant="outline_destructive"
              disabled={blocked}
              title={blocked ? "Other templates reference this version" : undefined}
              onClick={() => setDeleting(true)}
            >
              <Trash2 /> Delete
            </Button>
          </>
        }
      />

      <Summary template={shown} />

      <div className="mt-6 grid grid-cols-1 gap-6 xl:grid-cols-[minmax(0,1fr)_21rem]">
        <div className="min-w-0 space-y-6">
          <Panel
            title="Definition"
            hint="What the template stands for, with its references unexpanded."
          >
            <Tabs defaultValue="structure">
              <TabsList>
                <TabsTrigger value="structure">Structure</TabsTrigger>
                <TabsTrigger value="json">JSON</TabsTrigger>
              </TabsList>
              <TabsContent value="structure" className="pt-4">
                <TemplateContentView
                  kind={kind}
                  content={shown.content}
                  renderRef={(r) => <RefLink reference={r} />}
                />
              </TabsContent>
              <TabsContent value="json" className="pt-4">
                <JsonViewer value={{ content: shown.content, parameters: shown.parameters }} />
              </TabsContent>
            </Tabs>
          </Panel>

          <Panel
            title="Parameters"
            count={Object.keys(shown.parameters).length}
            hint="Asked whenever the template is used; a default fills in for an answer not given."
          >
            <ParameterTable parameters={shown.parameters} />
          </Panel>

          <TryIt template={shown} />
        </div>

        <aside className="space-y-6">
          <Panel
            title="Uses"
            icon={<ArrowDownToLine className="h-4 w-4 text-ink/45" />}
            count={pieces.length}
          >
            {pieces.length === 0 ? (
              <p className="text-sm text-ink/55">No other template.</p>
            ) : (
              <ul className="space-y-1.5">
                {pieces.map((ref) => (
                  <li key={ref}>
                    <RefLink reference={ref} />
                  </li>
                ))}
              </ul>
            )}
          </Panel>

          <Panel
            title="Used by"
            icon={<ArrowUpFromLine className="h-4 w-4 text-ink/45" />}
            count={usedBy}
          >
            {usedBy === 0 ? (
              <p className="text-sm text-ink/55">Nothing you can see uses this version.</p>
            ) : (
              <div className="space-y-3">
                {usages.templates.length > 0 && (
                  <div>
                    <p className="mb-1 text-[11px] font-medium uppercase tracking-wide text-ink/45">
                      Templates
                    </p>
                    <ul className="space-y-1.5">
                      {usages.templates.map((ref) => (
                        <li key={ref}>
                          <RefLink reference={ref} />
                        </li>
                      ))}
                    </ul>
                  </div>
                )}
                {usages.offers.length > 0 && (
                  <div>
                    <p className="mb-1 text-[11px] font-medium uppercase tracking-wide text-ink/45">
                      Offers
                    </p>
                    <ul className="space-y-1.5 text-sm">
                      {usages.offers.map((id) => (
                        <li key={id}>
                          <Identifier value={id} />
                        </li>
                      ))}
                    </ul>
                  </div>
                )}
              </div>
            )}
          </Panel>

          <Panel
            title="History"
            icon={<History className="h-4 w-4 text-ink/45" />}
            count={versions.length}
          >
            <ol className="space-y-0.5">
              {versions.map((v, i) => (
                <li key={v.version}>
                  <button
                    type="button"
                    onClick={() => goTo(v.version)}
                    className={cn(
                      "flex w-full items-center justify-between rounded-md px-2.5 py-1.5 text-left text-sm hover:bg-ink/[0.05]",
                      v.version === shown.version && "bg-ink/[0.07] font-medium",
                    )}
                  >
                    <span className="font-mono">
                      v{v.version}
                      {i === 0 && (
                        <span className="ml-2 font-sans text-[11px] font-normal text-ink/45">
                          latest
                        </span>
                      )}
                    </span>
                    <span className="text-xs text-ink/50">
                      <FormatDate date={v.date} format="YYYY-MM-DD" />
                    </span>
                  </button>
                </li>
              ))}
            </ol>
          </Panel>
        </aside>
      </div>

      <ConfirmDialog
        open={deleting}
        onOpenChange={setDeleting}
        title={`Delete version ${shown.version}?`}
        description="Offers composed with it keep their ODRL; they just stop pointing to it."
        confirmLabel="Delete"
        onConfirm={remove}
      >
        The agent refuses while another template version references it.
      </ConfirmDialog>
    </PageLayout>
  );
}

/** A bordered block with a title row. */
function Panel({
  title,
  hint,
  icon,
  count,
  children,
}: {
  title: string;
  hint?: string;
  icon?: React.ReactNode;
  count?: number;
  children: React.ReactNode;
}) {
  return (
    <section className="rounded-xl border border-ink/10 bg-ink/[0.012]">
      <header className="border-b border-ink/[0.07] px-4 py-3">
        <h2 className="flex items-center gap-2 text-sm font-semibold">
          {icon}
          {title}
          {count !== undefined && (
            <span className="rounded-full bg-ink/[0.07] px-1.5 text-xs font-normal text-ink/60">
              {count}
            </span>
          )}
        </h2>
        {hint && <p className="mt-0.5 text-xs text-ink/50">{hint}</p>}
      </header>
      <div className="p-4">{children}</div>
    </section>
  );
}

/** Reference, visibility, author, date and control points, in one row. */
function Summary({ template }: { template: PolicyTemplateDto }) {
  const [copied, setCopied] = useState(false);
  const reference = PolicyTemplateApi.reference(template);
  const copy = async () => {
    await navigator.clipboard.writeText(reference);
    setCopied(true);
    setTimeout(() => setCopied(false), 1500);
  };
  const cells: { label: string; value: React.ReactNode; wide?: boolean }[] = [
    {
      label: "Reference",
      wide: true,
      value: (
        <span className="flex items-center gap-1.5">
          <code className="truncate font-mono text-sm">{reference}</code>
          <Button size="icon_sm" variant="ghost" onClick={copy} aria-label="Copy reference">
            {copied ? <Check /> : <Copy />}
          </Button>
        </span>
      ),
    },
    { label: "Visibility", value: template.visibility },
    { label: "Author", value: template.author || template.userId },
    { label: "Saved", value: <FormatDate date={template.date} /> },
    {
      label: "Meant to apply at",
      wide: true,
      value: template.controlPoints?.length ? (
        <ControlPointChips points={template.controlPoints} />
      ) : (
        "—"
      ),
    },
  ];
  return (
    <dl className="grid grid-cols-2 overflow-hidden rounded-xl border border-ink/10 md:grid-cols-7 md:divide-x md:divide-ink/[0.07]">
      {cells.map((c) => (
        <div key={c.label} className={cn("min-w-0 px-4 py-3", c.wide && "col-span-2")}>
          <dt className="text-[11px] font-medium uppercase tracking-wide text-ink/45">{c.label}</dt>
          <dd className="mt-1 text-sm">{c.value}</dd>
        </div>
      ))}
    </dl>
  );
}

/** Each parameter: its name, type, default and what limits it. */
function ParameterTable({
  parameters,
}: {
  parameters: Record<string, PolicyTemplateParameterDefinition>;
}) {
  const rows = Object.entries(parameters);
  if (rows.length === 0)
    return <p className="text-sm text-ink/55">None: the template is used as it is.</p>;
  const limits = (d: PolicyTemplateParameterDefinition) => {
    const r = d.restrictions ?? {};
    const out: string[] = [];
    if (r.minValue != null || r.maxValue != null)
      out.push(`${r.minValue ?? "…"} – ${r.maxValue ?? "…"}`);
    if (r.minLength != null || r.maxLength != null)
      out.push(`${r.minLength ?? 0}–${r.maxLength ?? "…"} chars`);
    if (r.minItems != null || r.maxItems != null)
      out.push(`${r.minItems ?? 0}–${r.maxItems ?? "…"} items`);
    if (r.minDate || r.maxDate) out.push(`${r.minDate ?? "…"} → ${r.maxDate ?? "…"}`);
    if (r.regex) out.push(`matches ${r.regex}`);
    if (Array.isArray(r.values))
      out.push(
        `one of ${(r.values as (string | { value: string })[])
          .map((v) => (typeof v === "string" ? v : v.value))
          .join(", ")}`,
      );
    return out;
  };
  return (
    <div className="overflow-x-auto">
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Parameter</TableHead>
            <TableHead>Type</TableHead>
            <TableHead>Default</TableHead>
            <TableHead>Limits</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {rows.map(([name, def]) => (
            <TableRow key={name}>
              <TableCell>
                <span className="block font-mono text-[13px] text-brand-sky">{name}</span>
                {(def.ui?.label || def.ui?.description) && (
                  <span className="block text-xs text-ink/55">
                    {PolicyTemplateApi.text(def.ui?.label)}
                    {def.ui?.label && def.ui?.description && " — "}
                    {PolicyTemplateApi.text(def.ui?.description)}
                  </span>
                )}
              </TableCell>
              <TableCell>
                <Badge variant="outline" size="xs">
                  {def.dataType}
                </Badge>
              </TableCell>
              <TableCell>
                {def.defaultValue === undefined ? (
                  <span className="text-xs text-ink/45">required</span>
                ) : (
                  <code className="font-mono text-xs">
                    {typeof def.defaultValue === "string"
                      ? def.defaultValue
                      : JSON.stringify(def.defaultValue)}
                  </code>
                )}
              </TableCell>
              <TableCell className="text-xs text-ink/65">
                {limits(def).join(" · ") || "—"}
              </TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </div>
  );
}

/** A link to the page of the template an `id@version` names. */
function RefLink({ reference }: { reference: string }) {
  const at = reference.lastIndexOf("@");
  return (
    <Link
      to="/my-catalog/policies/$templateId"
      params={{ templateId: reference.slice(0, at) }}
      search={{ version: reference.slice(at + 1) }}
      className="font-mono text-[13px] text-brand-sky hover:underline"
    >
      {reference}
    </Link>
  );
}

/** The template filled with values the user types, flattened by the agent as they change. */
function TryIt({ template }: { template: PolicyTemplateDto }) {
  const [values, setValues] = useState<ParameterValues>(() =>
    PolicyTemplateApi.defaults(template.parameters),
  );
  const [preview, setPreview] = useState<PolicyPreviewDto | null>(null);
  const [error, setError] = useState<string | null>(null);
  const missing = PolicyTemplateApi.missing(template.parameters, values);
  const hasParameters = Object.keys(template.parameters).length > 0;

  useEffect(() => setValues(PolicyTemplateApi.defaults(template.parameters)), [template]);
  useEffect(() => {
    if (missing.length > 0) {
      setPreview(null);
      setError(null);
      return;
    }
    let current = true;
    const timer = setTimeout(() => {
      PolicyTemplateApi.preview({
        id: template.id,
        version: template.version,
        parameters: PolicyTemplateApi.given(values),
      })
        .then((p) => current && (setPreview(p), setError(null)))
        .catch((e: Error) => current && (setPreview(null), setError(e.message)));
    }, 400);
    return () => {
      current = false;
      clearTimeout(timer);
    };
  }, [template, values, missing.length]);

  const kind = template.kind ?? "policy";
  return (
    <Panel
      title="Try it"
      hint="The plain ODRL the template flattens to with these values, as offers built from it carry it."
    >
      <div
        className={cn(
          "grid grid-cols-1 gap-5",
          hasParameters && "lg:grid-cols-[minmax(0,0.9fr)_minmax(0,1fr)]",
        )}
      >
        {hasParameters && (
          <TemplateParameterFields
            parameters={template.parameters}
            values={values}
            onChange={setValues}
          />
        )}
        <div className="min-w-0 rounded-lg border border-ink/10 bg-background p-4">
          {error && <p className="text-sm text-danger-700 dark:text-danger-300">{error}</p>}
          {!error && !preview && (
            <p className="text-sm text-ink/55">
              {missing.length > 0
                ? `Fill in ${missing.map((m) => m.slice(1)).join(", ")}.`
                : "Flattening…"}
            </p>
          )}
          {preview && (
            <Tabs defaultValue="view">
              <TabsList>
                <TabsTrigger value="view">{kind === "policy" ? "Rules" : "Result"}</TabsTrigger>
                <TabsTrigger value="json">ODRL</TabsTrigger>
              </TabsList>
              <TabsContent value="view" className="space-y-3 pt-3">
                {kind === "policy" ? (
                  <OdrlRulesView policy={preview.odrl} />
                ) : (
                  <TemplateContentView kind={kind} content={preview.odrl} />
                )}
                <ControlPointChips points={preview.controlPoints} />
              </TabsContent>
              <TabsContent value="json" className="pt-3">
                <JsonViewer value={preview.odrl} />
              </TabsContent>
            </Tabs>
          )}
        </div>
      </div>
    </Panel>
  );
}
