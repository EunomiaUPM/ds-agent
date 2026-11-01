import React from "react";
import { CatalogViews, type CatalogDoc } from "../../data/catalog";
import { cn, Skeleton } from "@eunomia/ui";
import { useGetDatasetById } from "shared/src/data/orval/datasets/datasets";
import { Identifier } from "@eunomia/ui";
import type { DatasetDirectory } from "shared/src/data/useDatasetDirectory";

/**
 * A dataset by its title when it is in the local catalog, else by its identifier.
 * Lists pass a `directory` so titles come from one batch call instead of one per row.
 */
export function DatasetLabel({
  id,
  className,
  compact = false,
  loading = false,
  directory,
}: {
  id?: string | null;
  className?: string;
  compact?: boolean;
  loading?: boolean;
  directory?: DatasetDirectory;
}) {
  const single = useGetDatasetById(id ?? "", {
    query: { enabled: !!id && !directory, retry: false, staleTime: 5 * 60_000 },
  });
  const pending = loading || (directory ? directory.isLoading : single.isLoading);

  // The ghost keeps the shape of the resolved label so rows do not jump.
  if (pending) {
    return (
      <span className={cn("inline-flex min-w-0 flex-col gap-1 py-px", className)}>
        <Skeleton className="h-3.5 w-32" />
        {!compact && <Skeleton className="h-3.5 w-24" />}
      </span>
    );
  }
  if (!id) return <span className="text-ink/35">—</span>;

  const title = directory
    ? directory.titleOf(id)
    : single.data?.status === 200
      ? CatalogViews.resource(single.data.data as CatalogDoc).title
      : undefined;

  if (!title && compact) return <Identifier value={id} className={className} />;
  return (
    <span className={cn("inline-flex min-w-0 flex-col", className)}>
      <span className={cn("truncate", title ? "font-medium" : "italic text-ink/45")}>
        {title ?? "Not in the local catalog"}
      </span>
      {!compact && <Identifier value={id} />}
    </span>
  );
}
