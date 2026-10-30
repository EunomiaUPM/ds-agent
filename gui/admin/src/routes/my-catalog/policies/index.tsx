import React, { useMemo, useState } from "react";
import { createFileRoute, Link, useNavigate } from "@tanstack/react-router";
import { Library, Plus } from "lucide-react";
import { PageLayout } from "shared/src/components/layout/PageLayout";
import { PageHeader } from "shared/src/components/layout/PageHeader";
import { DataTable } from "shared/src/components/DataTable";
import { Button } from "shared/src/components/ui/button";
import { Badge } from "shared/src/components/ui/badge";
import { Input } from "shared/src/components/ui/input";
import { FormatDate } from "shared/src/components/ui/format-date";
import { SegmentedControl } from "shared/src/components/ui/segmented";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "shared/src/components/ui/select";
import { ControlPointChips } from "shared/src/components/policy/ControlPointChips";
import { useGetAllPolicyTemplates } from "shared/src/data/orval/policy-templates/policy-templates";
import {
  CONTROL_POINTS,
  PolicyTemplateApi,
  type TemplateGroup,
} from "shared/src/data/policyTemplateApi";
import type {
  ControlPoint,
  PaginatedPolicyTemplateDto,
  PolicyTemplateKind,
} from "shared/src/data/orval/model";

export const Route = createFileRoute("/my-catalog/policies/")({
  component: PolicyLibraryPage,
});

type KindFilter = PolicyTemplateKind | "all";

const KIND_OPTIONS: { value: KindFilter; label: string; hint?: string }[] = [
  { value: "all", label: "All" },
  { value: "policy", label: "Policies", hint: "Whole policies" },
  { value: "rule", label: "Rules", hint: "One permission, prohibition or obligation" },
  { value: "constraint", label: "Constraints", hint: "A condition that joins rules" },
];

function PolicyLibraryPage() {
  const navigate = useNavigate();
  const [kind, setKind] = useState<KindFilter>("all");
  const [point, setPoint] = useState<ControlPoint | "any">("any");
  const [search, setSearch] = useState("");
  const { data, isLoading } = useGetAllPolicyTemplates({
    limit: 100,
    kind: kind === "all" ? undefined : kind,
    control_point: point === "any" ? undefined : point,
  });
  const groups = useMemo(() => {
    const items = data?.status === 200 ? (data.data as PaginatedPolicyTemplateDto).items : [];
    const q = search.trim().toLowerCase();
    return PolicyTemplateApi.group(items).filter(
      (g) => !q || `${PolicyTemplateApi.title(g.latest)} ${g.id}`.toLowerCase().includes(q),
    );
  }, [data, search]);
  const filtered = kind !== "all" || point !== "any" || !!search;

  return (
    <PageLayout>
      <PageHeader
        eyebrow="My catalog"
        title="Policy library"
        description="Reusable ODRL pieces: constraints, rules and whole policies that reference each other. Compose your offers from them."
        actions={
          <Button asChild>
            <Link to="/my-catalog/policies/new">
              <Plus /> New template
            </Link>
          </Button>
        }
      />

      <div className="mb-4 flex flex-wrap items-center gap-2">
        <SegmentedControl value={kind} onChange={setKind} options={KIND_OPTIONS} ariaLabel="Kind" />
        <Select value={point} onValueChange={(v) => setPoint(v as ControlPoint | "any")}>
          <SelectTrigger className="h-8 w-48 text-xs">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="any">Any control point</SelectItem>
            {CONTROL_POINTS.map((p) => (
              <SelectItem key={p.value} value={p.value}>
                {p.label}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
        <Input
          placeholder="Search by title…"
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          onClear={() => setSearch("")}
          className="h-8 w-64 text-xs"
        />
      </div>

      {!isLoading && groups.length === 0 && !filtered ? (
        <div className="rounded-lg border border-dashed border-ink/15 px-6 py-12 text-center">
          <Library className="mx-auto h-10 w-10 text-ink/30" />
          <p className="mt-3 text-sm font-medium">No templates yet</p>
          <p className="mx-auto mt-1 max-w-md text-sm text-ink/55">
            The connector seeds a library at start-up from its policy templates folder; you can add
            your own pieces too.
          </p>
        </div>
      ) : (
        <DataTable<TemplateGroup>
          data={groups}
          keyExtractor={(g) => g.id}
          hideToolbar
          emptyMessage="No template matches."
          onRowClick={(g) =>
            navigate({ to: "/my-catalog/policies/$templateId", params: { templateId: g.id } })
          }
          columns={[
            {
              header: "Template",
              cell: (g) => (
                <span className="flex flex-col">
                  <span className="font-medium">{PolicyTemplateApi.title(g.latest)}</span>
                  {g.latest.description && (
                    <span className="text-xs text-ink/55">
                      {PolicyTemplateApi.text(g.latest.description)}
                    </span>
                  )}
                </span>
              ),
            },
            {
              header: "Kind",
              cell: (g) => <Badge variant="outline">{g.latest.kind ?? "policy"}</Badge>,
            },
            {
              header: "Applies at",
              cell: (g) => <ControlPointChips points={g.latest.controlPoints} />,
            },
            {
              header: "Latest",
              cell: (g) => <span className="font-mono text-xs">{g.latest.version}</span>,
            },
            {
              header: "Pieces used",
              cell: (g) => (
                <span className="text-sm text-ink/70">
                  {PolicyTemplateApi.references(g.latest.content).length || "—"}
                </span>
              ),
            },
            {
              header: "Updated",
              cell: (g) => <FormatDate date={g.latest.date} />,
            },
          ]}
        />
      )}
    </PageLayout>
  );
}
