import React, { useMemo, useState } from "react";
import { createFileRoute, Link, useNavigate } from "@tanstack/react-router";
import { FileJson, Plus } from "lucide-react";
import { PageLayout } from "shared/src/components/layout/PageLayout";
import { PageHeader } from "shared/src/components/layout/PageHeader";
import { DataTable } from "shared/src/components/DataTable";
import { Button } from "shared/src/components/ui/button";
import { Badge } from "shared/src/components/ui/badge";
import { Input } from "shared/src/components/ui/input";
import { FormatDate } from "shared/src/components/ui/format-date";
import { SchemaUrl } from "shared/src/components/schemas/SchemaPicker";
import { useGetSchemas } from "shared/src/data/orval/schemas/schemas";
import { SchemaApi, type SchemaGroup } from "shared/src/data/schemaApi";
import type { PaginatedSchemaView } from "shared/src/data/orval/model";

export const Route = createFileRoute("/my-catalog/schemas/")({
  component: SchemasPage,
});

function SchemasPage() {
  const navigate = useNavigate();
  const [search, setSearch] = useState("");
  const { data, isLoading } = useGetSchemas({ limit: 100, title: search.trim() || undefined });
  const groups = useMemo(
    () => (data?.status === 200 ? SchemaApi.group((data.data as PaginatedSchemaView).items) : []),
    [data],
  );

  return (
    <PageLayout>
      <PageHeader
        eyebrow="My catalog"
        title="Data schemas"
        description="JSON Schemas this connector publishes. Datasets point to them as conformsTo, and their distributions inherit them."
        actions={
          <Button asChild>
            <Link to="/my-catalog/schemas/new">
              <Plus /> New schema
            </Link>
          </Button>
        }
      />

      <div className="mb-4 flex items-center gap-2">
        <Input
          placeholder="Search by title…"
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          onClear={() => setSearch("")}
          className="h-8 w-72 text-xs"
        />
      </div>

      {!isLoading && groups.length === 0 && !search ? (
        <div className="rounded-lg border border-dashed border-ink/15 px-6 py-12 text-center">
          <FileJson className="mx-auto h-10 w-10 text-ink/30" />
          <p className="mt-3 text-sm font-medium">No schemas yet</p>
          <p className="mx-auto mt-1 max-w-md text-sm text-ink/55">
            Paste a sample of your data and the builder infers its schema; then publish it and point
            your datasets to it.
          </p>
          <Button asChild size="sm" className="mt-4">
            <Link to="/my-catalog/schemas/new">
              <Plus /> Build one from a sample
            </Link>
          </Button>
        </div>
      ) : (
        <DataTable<SchemaGroup>
          data={groups}
          keyExtractor={(g) => g.id}
          hideToolbar
          emptyMessage="No schema matches."
          onRowClick={(g) =>
            navigate({ to: "/my-catalog/schemas/$schemaId", params: { schemaId: g.id } })
          }
          columns={[
            {
              header: "Schema",
              cell: (g) => (
                <span className="flex flex-col">
                  <span className="font-medium">{g.latest.title}</span>
                  {g.latest.description && (
                    <span className="text-xs text-ink/55">{g.latest.description}</span>
                  )}
                </span>
              ),
            },
            {
              header: "Latest",
              cell: (g) => <Badge variant="outline">v{g.latest.version}</Badge>,
            },
            {
              header: "Visibility",
              cell: (g) => <span className="text-sm text-ink/70">{g.latest.visibility}</span>,
            },
            {
              header: "Public URL",
              cell: (g) => (
                <span onClick={(e: React.MouseEvent) => e.stopPropagation()}>
                  <SchemaUrl url={g.latest.url} />
                </span>
              ),
            },
            {
              header: "Updated",
              cell: (g) => <FormatDate date={g.latest.createdAt} />,
            },
          ]}
        />
      )}
    </PageLayout>
  );
}
