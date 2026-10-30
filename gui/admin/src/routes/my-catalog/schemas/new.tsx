import React from "react";
import { createFileRoute, useNavigate } from "@tanstack/react-router";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { PageLayout } from "@eunomia/ui";
import { PageHeader } from "@eunomia/ui";
import { SchemaBuilder } from "shared/src/components/schemas/SchemaBuilder";
import { SchemaApi } from "shared/src/data/schemaApi";

/** `from` and `version` name the version a new one starts from. */
interface BuilderSearch {
  from?: string;
  version?: number;
}

export const Route = createFileRoute("/my-catalog/schemas/new")({
  component: NewSchemaPage,
  validateSearch: (search: Record<string, unknown>): BuilderSearch => ({
    from: typeof search.from === "string" ? search.from : undefined,
    version: search.version !== undefined ? Number(search.version) : undefined,
  }),
});

function NewSchemaPage() {
  const { from, version } = Route.useSearch();
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const { data: base, error } = useQuery({
    queryKey: ["schema-builder-base", from, version],
    queryFn: () => SchemaApi.version(from!, version!),
    enabled: !!from && !!version,
  });
  const waiting = !!from && !base && !error;

  return (
    <PageLayout>
      <PageHeader
        eyebrow="Data schemas"
        title={base ? `New version of ${base.title}` : "New schema"}
        description="Infer it from sample documents, adjust the fields, test it and publish it."
      />
      {error && <p className="text-sm text-danger-700">{(error as Error).message}</p>}
      {!waiting && !error && (
        <SchemaBuilder
          base={base}
          onCancel={() =>
            navigate({
              to: from ? "/my-catalog/schemas/$schemaId" : "/my-catalog/schemas",
              params: { schemaId: from ?? "" },
            })
          }
          onSaved={async (view) => {
            await SchemaApi.invalidate(queryClient);
            navigate({
              to: "/my-catalog/schemas/$schemaId",
              params: { schemaId: view.id },
              search: { version: view.version },
            });
          }}
        />
      )}
    </PageLayout>
  );
}
