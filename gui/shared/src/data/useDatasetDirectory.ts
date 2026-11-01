import { useMemo } from "react";
import { useQuery } from "@tanstack/react-query";
import { getBatchDatasets } from "shared/src/data/orval/datasets/datasets";
import { CatalogViews, type CatalogDoc } from "shared/src/data/catalog";

export interface DatasetDirectory {
  isLoading: boolean;
  titleOf: (id?: string | null) => string | undefined;
}

/** Titles of the given local datasets, fetched in one batch call; unknown ids are left out. */
export const useDatasetDirectory = (ids: (string | null | undefined)[]): DatasetDirectory => {
  const key = [...new Set(ids.filter((id): id is string => !!id))].sort();
  const { data, isLoading } = useQuery({
    queryKey: ["datasets", "batch", key],
    queryFn: () => getBatchDatasets({ ids: key }),
    enabled: key.length > 0,
    staleTime: 5 * 60_000,
  });

  return useMemo(() => {
    const titles = new Map<string, string>();
    const docs = data?.status === 200 ? data.data["@graph"] : [];
    for (const doc of docs) {
      const view = CatalogViews.resource(doc as CatalogDoc);
      if (view.id && view.title) titles.set(view.id, view.title);
    }
    return { isLoading, titleOf: (id) => (id ? titles.get(id) : undefined) };
  }, [data, isLoading]);
};
