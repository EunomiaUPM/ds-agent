/**
 * useTableQueryParams.ts
 *
 * Hook for synchronizing DataTable state (pagination, sorting, filtering, and search)
 * with browser URL search parameters and backend QuerySpec specifications.
 */

import { useState, useMemo, useCallback, useEffect } from "react";

export interface TableQueryParams {
  limit: number;
  page: number;
  cursor?: string;
  sort: string;
  search: string;
  filters: Record<string, string>;
}

export interface UseTableQueryParamsOptions {
  defaultLimit?: number;
  defaultSort?: string;
  defaultFilters?: Record<string, string>;
  syncUrl?: boolean;
}

/**
 * Maps frontend column key and direction to backend Sort enum representation.
 */
export function mapToSortParam(key: string, direction: "asc" | "desc"): string {
  if (key === "createdAt" || key === "created_at") {
    return direction === "asc" ? "created_at_asc" : "created_at_desc";
  }
  if (key === "updatedAt" || key === "updated_at" || key === "last_interaction") {
    return direction === "asc" ? "updated_at_asc" : "updated_at_desc";
  }
  const snakeKey = key.replace(/[A-Z]/g, (letter) => `_${letter.toLowerCase()}`);
  return `${snakeKey}_${direction}`;
}

/**
 * Hook for bidirectional synchronization of table queries with URL search params.
 */
export function useTableQueryParams(options: UseTableQueryParamsOptions = {}) {
  const {
    defaultLimit = 10,
    defaultSort = "created_at_desc",
    defaultFilters = {},
    syncUrl = true,
  } = options;

  // Initialize query state from window URL search params or defaults
  const [params, setParams] = useState<TableQueryParams>(() => {
    if (typeof window === "undefined") {
      return {
        limit: defaultLimit,
        page: 1,
        cursor: undefined,
        sort: defaultSort,
        search: "",
        filters: { ...defaultFilters },
      };
    }

    const sp = new URLSearchParams(window.location.search);
    const limit = Number(sp.get("limit")) || defaultLimit;
    const page = Number(sp.get("page")) || 1;
    const cursor = sp.get("cursor") || undefined;
    const sort = sp.get("sort") || defaultSort;
    const search = sp.get("search") || "";

    const filters: Record<string, string> = { ...defaultFilters };
    sp.forEach((value, key) => {
      if (!["limit", "page", "cursor", "sort", "search"].includes(key)) {
        filters[key] = value;
      }
    });

    return { limit, page, cursor, sort, search, filters };
  });

  // Sync state changes with browser URL search string
  useEffect(() => {
    if (!syncUrl || typeof window === "undefined") return;

    const url = new URL(window.location.href);
    const sp = url.searchParams;

    if (params.limit !== defaultLimit) {
      sp.set("limit", String(params.limit));
    } else {
      sp.delete("limit");
    }

    if (params.page > 1) {
      sp.set("page", String(params.page));
    } else {
      sp.delete("page");
    }

    if (params.cursor) {
      sp.set("cursor", params.cursor);
    } else {
      sp.delete("cursor");
    }

    if (params.sort && params.sort !== defaultSort) {
      sp.set("sort", params.sort);
    } else {
      sp.delete("sort");
    }

    if (params.search && params.search.trim()) {
      sp.set("search", params.search.trim());
    } else {
      sp.delete("search");
    }

    Object.entries(params.filters).forEach(([key, value]) => {
      if (value && value !== "all") {
        sp.set(key, value);
      } else {
        sp.delete(key);
      }
    });

    const newSearch = sp.toString();
    const newRelativePath = url.pathname + (newSearch ? `?${newSearch}` : "");
    window.history.replaceState(null, "", newRelativePath);
  }, [params, defaultLimit, defaultSort, syncUrl]);

  // Clean flattened parameters ready for backend QuerySpec requests
  const apiParams = useMemo(() => {
    const p: Record<string, any> = {
      limit: params.limit,
      sort: params.sort,
    };

    if (params.cursor) {
      p.cursor = params.cursor;
    } else if (params.page > 1) {
      p.page = params.page;
    }

    if (params.search && params.search.trim()) {
      p.search = params.search.trim();
    }

    Object.entries(params.filters).forEach(([k, v]) => {
      if (v && v !== "all") {
        p[k] = v;
      }
    });

    return p;
  }, [params]);

  const onQueryChange = useCallback((updated: Partial<TableQueryParams>) => {
    setParams((prev) => {
      const nextFilters = updated.filters ? { ...prev.filters, ...updated.filters } : prev.filters;
      return {
        ...prev,
        ...updated,
        filters: nextFilters,
      };
    });
  }, []);

  const setPage = useCallback((page: number) => {
    setParams((p) => ({ ...p, page, cursor: undefined }));
  }, []);

  const setLimit = useCallback((limit: number) => {
    setParams((p) => ({ ...p, limit, page: 1, cursor: undefined }));
  }, []);

  const setSort = useCallback((sort: string) => {
    setParams((p) => ({ ...p, sort, page: 1, cursor: undefined }));
  }, []);

  const setSearch = useCallback((search: string) => {
    setParams((p) => ({ ...p, search, page: 1, cursor: undefined }));
  }, []);

  const setFilter = useCallback((id: string, value: string) => {
    setParams((p) => ({
      ...p,
      page: 1,
      cursor: undefined,
      filters: { ...p.filters, [id]: value },
    }));
  }, []);

  const resetAll = useCallback(() => {
    setParams({
      limit: defaultLimit,
      page: 1,
      cursor: undefined,
      sort: defaultSort,
      search: "",
      filters: { ...defaultFilters },
    });
  }, [defaultLimit, defaultSort, defaultFilters]);

  return {
    params,
    apiParams,
    onQueryChange,
    setPage,
    setLimit,
    setSort,
    setSearch,
    setFilter,
    resetAll,
  };
}
