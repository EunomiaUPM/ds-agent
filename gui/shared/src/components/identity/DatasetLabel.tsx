import React from "react";
import { CatalogViews, type CatalogDoc } from "../../data/catalog";
import { cn } from "shared/src/lib/utils";
import { useGetDatasetById } from "shared/src/data/orval/datasets/datasets";
import { Identifier } from "./Identifier";

/** A dataset by its title when it is in the local catalog, else by its identifier. */
export function DatasetLabel({
  id,
  className,
  compact = false,
}: {
  id?: string | null;
  className?: string;
  compact?: boolean;
}) {
  const { data } = useGetDatasetById(id ?? "", {
    query: { enabled: !!id, retry: false, staleTime: 5 * 60_000 },
  });
  if (!id) return <span className="text-ink/35">—</span>;
  const title = data?.status === 200 ? CatalogViews.resource(data.data as CatalogDoc).title : undefined;

  if (!title) return <Identifier value={id} className={className} />;
  return (
    <span className={cn("inline-flex min-w-0 flex-col", className)}>
      <span className="truncate font-medium">{title}</span>
      {!compact && <Identifier value={id} />}
    </span>
  );
}
