import React, { useState } from "react";
import { createFileRoute, Link, useNavigate } from "@tanstack/react-router";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { toast } from "sonner";
import { Copy, GitBranchPlus, Trash2 } from "lucide-react";
import { cn } from "shared/src/lib/utils";
import { PageLayout } from "shared/src/components/layout/PageLayout";
import { PageHeader } from "shared/src/components/layout/PageHeader";
import { PageSection } from "shared/src/components/layout/PageSection";
import { FieldGrid } from "shared/src/components/layout/FieldGrid";
import { Button } from "shared/src/components/ui/button";
import { Badge } from "shared/src/components/ui/badge";
import { FormatDate } from "shared/src/components/ui/format-date";
import { ConfirmDialog } from "shared/src/components/ui/confirm-dialog";
import { JsonViewer } from "shared/src/components/json/JsonViewer";
import { SchemaUrl } from "shared/src/components/schemas/SchemaPicker";
import { useGetSchemaVersions } from "shared/src/data/orval/schemas/schemas";
import { useGetMainCatalogs } from "shared/src/data/orval/catalogs/catalogs";
import { CatalogViews, type CatalogDoc } from "shared/src/data/catalog";
import { SchemaApi } from "shared/src/data/schemaApi";
import type { SchemaView } from "shared/src/data/orval/model";

interface DetailSearch {
  version?: number;
}

export const Route = createFileRoute("/my-catalog/schemas/$schemaId")({
  component: SchemaPage,
  validateSearch: (search: Record<string, unknown>): DetailSearch => ({
    version: search.version !== undefined ? Number(search.version) : undefined,
  }),
});

type Deleting = { version?: number } | null;

function SchemaPage() {
  const { schemaId } = Route.useParams();
  const { version: chosen } = Route.useSearch();
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const [deleting, setDeleting] = useState<Deleting>(null);

  const { data: versionsData } = useGetSchemaVersions(schemaId);
  const versions = versionsData?.status === 200 ? (versionsData.data as SchemaView[]) : [];
  const shown = versions.find((v) => v.version === chosen) ?? versions[0];
  const { data: detail } = useQuery({
    queryKey: ["schema-version", schemaId, shown?.version],
    queryFn: () => SchemaApi.version(schemaId, shown!.version),
    enabled: !!shown,
  });
  const { data: mainData } = useGetMainCatalogs();
  const catalogId =
    mainData?.status === 200 ? CatalogViews.catalog(mainData.data as CatalogDoc).id : "";

  if (!shown) return null;
  const used = detail?.usedBy ?? [];

  const copy = () => {
    navigator.clipboard.writeText(shown.url);
    toast.success("URL copied.");
  };

  const remove = async () => {
    const version = deleting?.version;
    await SchemaApi.remove(schemaId, version);
    await SchemaApi.invalidate(queryClient);
    toast.success(version ? `Deleted version ${version}.` : "Deleted the schema.");
    if (version === undefined || versions.length === 1) navigate({ to: "/my-catalog/schemas" });
    else navigate({ to: "/my-catalog/schemas/$schemaId", params: { schemaId }, search: {} });
  };

  return (
    <PageLayout>
      <PageHeader
        eyebrow="Data schema"
        title={shown.title}
        badge={<Badge variant="outline">v{shown.version}</Badge>}
        description={shown.description}
        actions={
          <>
            <Button asChild variant="outline">
              <Link
                to="/my-catalog/schemas/new"
                search={{ from: schemaId, version: versions[0].version }}
              >
                <GitBranchPlus /> New version
              </Link>
            </Button>
            <Button variant="outline_destructive" onClick={() => setDeleting({})}>
              <Trash2 /> Delete schema
            </Button>
          </>
        }
      />

      <div className="grid grid-cols-1 gap-6 lg:grid-cols-[14rem_minmax(0,1fr)]">
        <nav className="space-y-1">
          <p className="mb-2 text-xs font-medium uppercase tracking-wide text-ink/50">Versions</p>
          {versions.map((v) => (
            <Link
              key={v.version}
              to="/my-catalog/schemas/$schemaId"
              params={{ schemaId }}
              search={{ version: v.version }}
              className={cn(
                "flex items-center justify-between rounded-md px-3 py-2 text-sm hover:bg-ink/[0.05]",
                v.version === shown.version && "bg-ink/[0.07] font-medium",
              )}
            >
              <span>v{v.version}</span>
              <span className="text-xs text-ink/50">
                <FormatDate date={v.createdAt} format="YYYY-MM-DD" />
              </span>
            </Link>
          ))}
        </nav>

        <div className="space-y-6">
          <FieldGrid
            columns={3}
            fields={[
              {
                label: "Public URL (what conformsTo points to)",
                span: 3,
                value: (
                  <span className="flex items-center gap-2">
                    <SchemaUrl url={shown.url} />
                    <Button size="icon_sm" variant="ghost" onClick={copy} title="Copy URL">
                      <Copy />
                    </Button>
                  </span>
                ),
              },
              { label: "Visibility", value: shown.visibility },
              {
                label: "Saved",
                value: <FormatDate date={shown.createdAt} format="YYYY-MM-DD HH:mm" />,
              },
              {
                label: "SHA-256",
                value: <span className="font-mono text-xs">{shown.sha256.slice(0, 16)}…</span>,
              },
            ]}
          />

          <PageSection
            title={`Used by (${used.length})`}
            action={
              <Button
                size="xs"
                variant="ghost"
                className="text-danger-700 dark:text-danger-300"
                disabled={used.length > 0}
                title={used.length > 0 ? "Point these resources elsewhere first" : undefined}
                onClick={() => setDeleting({ version: shown.version })}
              >
                <Trash2 /> Delete v{shown.version}
              </Button>
            }
          >
            {used.length === 0 ? (
              <p className="text-sm text-ink/55">
                No dataset or distribution you can see points to this version.
              </p>
            ) : (
              <ul className="space-y-1 text-sm">
                {used.map((u) => (
                  <li key={u.id} className="flex items-center gap-2">
                    <Badge variant="outline" className="text-xs">
                      {u.kind}
                    </Badge>
                    {u.kind === "dataset" && catalogId ? (
                      <Link
                        to="/catalog/$catalogId/dataset/$datasetId"
                        params={{ catalogId, datasetId: u.id }}
                        className="text-brand-sky hover:underline"
                      >
                        {u.title || u.id}
                      </Link>
                    ) : u.kind === "distribution" && catalogId ? (
                      <Link
                        to="/catalog/$catalogId/distribution-connector/$distributionId"
                        params={{ catalogId, distributionId: u.id }}
                        className="text-brand-sky hover:underline"
                      >
                        {u.title || u.id}
                      </Link>
                    ) : (
                      <span>{u.title || u.id}</span>
                    )}
                  </li>
                ))}
              </ul>
            )}
          </PageSection>

          <PageSection title="Schema">
            {detail ? (
              <JsonViewer value={detail.schema} />
            ) : (
              <p className="text-sm text-ink/55">Loading…</p>
            )}
          </PageSection>
        </div>
      </div>

      <ConfirmDialog
        open={!!deleting}
        onOpenChange={(open) => !open && setDeleting(null)}
        title={
          deleting?.version !== undefined
            ? `Delete version ${deleting.version}?`
            : "Delete this schema and every version?"
        }
        description="Its public URL stops answering. This cannot be undone."
        confirmLabel="Delete"
        onConfirm={remove}
      >
        The agent refuses while a dataset or distribution, yours or not, still points to it.
      </ConfirmDialog>
    </PageLayout>
  );
}
