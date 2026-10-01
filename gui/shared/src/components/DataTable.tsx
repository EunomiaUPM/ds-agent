/**
 * DataTable.tsx
 *
 * Enterprise generic table with client and server-side sorting, live search filtering,
 * declarative filter selects, configurable pagination, and page size selection.
 */

import React, { useState, useMemo, useEffect, useCallback } from "react";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "./ui/table";
import { Input } from "./ui/input";
import { Badge } from "./ui/badge";
import { Button } from "./ui/button";
import {
  ArrowDown,
  ArrowUp,
  ArrowUpDown,
  ChevronLeft,
  ChevronRight,
  ChevronsLeft,
  ChevronsRight,
  Inbox,
  SearchX,
  X,
} from "lucide-react";
import { cn } from "shared/src/lib/utils";
import { mapToSortParam, TableQueryParams } from "../hooks/useTableQueryParams";

const toSnakeCase = (str: string) => str.replace(/[A-Z]/g, (l) => `_${l.toLowerCase()}`);

const isSortConfigMatch = (configKey: string | undefined | null, colKey: string) => {
  if (!configKey) return false;
  return configKey === colKey || configKey === toSnakeCase(colKey);
};

// =============================================================================
// TYPES
// =============================================================================

/**
 * Single option within a table filter.
 */
export interface TableFilterOption {
  label: string;
  value: string;
}

/**
 * Declarative filter definition for the DataTable toolbar.
 *
 * @template T - The type of data items in the table
 */
export interface TableFilter<T = any> {
  /** Unique identifier for the filter */
  id: string;

  /** Display label for the filter dropdown */
  label: string;

  /** Selectable options. Use value "all" for matching all items */
  options: TableFilterOption[];

  /** Controlled value for the filter */
  value?: string;

  /** Default selected value when uncontrolled */
  defaultValue?: string;

  /** Callback fired when the filter value changes */
  onChange?: (value: string) => void;

  /** Field key on data item to filter against when filterFn is omitted */
  accessorKey?: keyof T;

  /** Custom filter logic */
  filterFn?: (item: T, selectedValue: string) => boolean;
}

/**
 * Column definition for the DataTable.
 *
 * @template T - The type of data items in the table
 */
export interface Column<T> {
  /** Header text or component displayed in the table header */
  header: React.ReactNode | string;

  /**
   * Key to access the value from the data item.
   * Used when no custom cell renderer is provided.
   */
  accessorKey?: keyof T;

  /**
   * Custom cell renderer function.
   * Takes precedence over accessorKey when provided.
   */
  cell?: (item: T) => React.ReactNode;

  /** Additional CSS class for the cells */
  className?: string;

  /** Additional CSS class for the header cell */
  headerClassName?: string;

  /** Whether this column is sortable. Defaults to true when accessorKey/sortKey/sortValue is set */
  sortable?: boolean;

  /** Custom sort key name */
  sortKey?: string;

  /** Custom sort value extractor */
  sortValue?: (item: T) => string | number | boolean | Date | null | undefined;

  /** Whether this column should be searched during filtering. Defaults to true */
  searchable?: boolean;

  /** Custom search text extractor for this column */
  searchValue?: (item: T) => string;
}

/**
 * Props for the DataTable component.
 *
 * @template T - The type of data items in the table
 */
export interface DataTableProps<T> {
  /** Column definitions */
  columns: Column<T>[];

  /** Array of data items or Paginated response envelope */
  data: T[] | { items: T[]; total?: number; nextCursor?: string } | null | undefined;

  /** Function to extract a unique key from each item */
  keyExtractor: (item: T) => string;

  /** Optional handler for row clicks */
  onRowClick?: (item: T) => void;

  /** Additional CSS class for the table */
  className?: string;

  /** Additional CSS class for the table container */
  containerClassName?: string;

  /**
   * Message shown when `data` is empty. Defaults to a generic
   * "No records found". Pass a domain-specific phrase for clarity.
   */
  emptyMessage?: React.ReactNode;

  /** Table title displayed in the toolbar */
  title?: React.ReactNode;

  /** Subtitle displayed under the title */
  subtitle?: React.ReactNode;

  /** Extra actions/buttons displayed on the toolbar */
  toolbarActions?: React.ReactNode;

  /** Whether client-side search is enabled. Defaults to true. */
  searchable?: boolean;

  /** Custom placeholder for search input */
  searchPlaceholder?: string;

  /** Custom filter function for live text search */
  filterFn?: (item: T, query: string) => boolean;

  /** Available declarative filters displayed in the toolbar */
  filters?: TableFilter<T>[];

  /** Default column key to sort by */
  defaultSortKey?: string;

  /** Default sort direction */
  defaultSortDirection?: "asc" | "desc";

  /** Whether client-side pagination is enabled. Defaults to true. */
  paginated?: boolean;

  /** Initial or fixed items per page. Defaults to 10. Pass -1 or Infinity to show all. */
  pageSize?: number;

  /** Available options in the rows-per-page dropdown. Defaults to [10, 20, 50, 100]. */
  pageSizeOptions?: number[];

  /** Whether to hide the search and filter toolbar */
  hideToolbar?: boolean;

  /** Whether to hide the record count badge */
  hideCount?: boolean;

  /** Enable server-side pagination, sorting, and filtering via query parameters */
  serverSide?: boolean;

  /** Total number of records across all pages when using server-side pagination */
  totalCount?: number;

  /** Next cursor for cursor-based pagination */
  nextCursor?: string;

  /** Controlled query state object */
  queryParams?: Partial<TableQueryParams>;

  /** Callback fired when sorting, filtering, pagination, or search changes */
  onQueryChange?: (params: Partial<TableQueryParams>) => void;

  /** Automatically synchronize query parameters with the browser URL. Defaults to true */
  syncUrlParams?: boolean;

  /** Whether the table is currently fetching new data in background */
  loading?: boolean;
}

// =============================================================================
// COMPONENT
// =============================================================================

/**
 * Enterprise table component with built-in client/server sorting, filtering, and pagination.
 */
export function DataTable<T extends Record<string, any> = any>({
  columns,
  data,
  keyExtractor,
  onRowClick,
  className,
  containerClassName,
  emptyMessage,
  title,
  subtitle,
  toolbarActions,
  searchable = true,
  searchPlaceholder = "Filter records...",
  filterFn,
  filters,
  defaultSortKey,
  defaultSortDirection,
  paginated = true,
  pageSize = 10,
  pageSizeOptions = [10, 20, 50, 100],
  hideToolbar = false,
  hideCount = false,
  serverSide = false,
  totalCount,
  nextCursor,
  queryParams,
  onQueryChange,
  syncUrlParams = true,
  loading = false,
}: DataTableProps<T>) {
  // Extract items array and total count from either raw array or Paginated response envelope
  const rawItems: T[] = useMemo(() => {
    if (!data) return [];
    if (Array.isArray(data)) return data;
    if (typeof data === "object" && Array.isArray((data as any).items)) {
      return (data as any).items;
    }
    return [];
  }, [data]);

  const totalCountValue = useMemo(() => {
    if (totalCount !== undefined) return totalCount;
    if (data && typeof data === "object" && typeof (data as any).total === "number") {
      return (data as any).total;
    }
    return undefined;
  }, [totalCount, data]);

  const isServerDriven = serverSide || Boolean(onQueryChange);

  // Search query state
  const [searchQuery, setSearchQuery] = useState(queryParams?.search ?? "");
  useEffect(() => {
    if (queryParams?.search !== undefined && queryParams.search !== searchQuery) {
      setSearchQuery(queryParams.search);
    }
  }, [queryParams?.search]);

  // Sort configuration state
  const [sortConfig, setSortConfig] = useState<{
    key: string;
    direction: "asc" | "desc";
  } | null>(() => {
    if (queryParams?.sort) {
      const s = queryParams.sort;
      if (s.endsWith("_asc")) return { key: s.slice(0, -4), direction: "asc" };
      if (s.endsWith("_desc")) return { key: s.slice(0, -5), direction: "desc" };
    }
    return defaultSortKey ? { key: defaultSortKey, direction: defaultSortDirection ?? "asc" } : null;
  });

  // Sync sort state from queryParams if provided
  useEffect(() => {
    if (queryParams?.sort) {
      const s = queryParams.sort;
      if (s.endsWith("_asc")) {
        const key = s.slice(0, -4);
        setSortConfig({ key, direction: "asc" });
      } else if (s.endsWith("_desc")) {
        const key = s.slice(0, -5);
        setSortConfig({ key, direction: "desc" });
      }
    }
  }, [queryParams?.sort]);

  const [currentPage, setCurrentPage] = useState(queryParams?.page ?? 1);
  useEffect(() => {
    if (queryParams?.page !== undefined) {
      setCurrentPage(queryParams.page);
    }
  }, [queryParams?.page]);

  // Initialize and track active rows-per-page
  const [itemsPerPage, setItemsPerPage] = useState<number | undefined>(() => {
    if (!paginated) return undefined;
    const initialLimit = queryParams?.limit ?? pageSize;
    if (initialLimit <= 0 || initialLimit === Infinity) return undefined;
    return initialLimit;
  });

  useEffect(() => {
    if (!paginated) {
      setItemsPerPage(undefined);
    } else if (queryParams?.limit !== undefined) {
      setItemsPerPage(queryParams.limit <= 0 ? undefined : queryParams.limit);
    } else if (pageSize <= 0 || pageSize === Infinity) {
      setItemsPerPage(undefined);
    } else {
      setItemsPerPage(pageSize);
    }
  }, [paginated, pageSize, queryParams?.limit]);

  // Manage declarative toolbar filter states
  const [filterValues, setFilterValues] = useState<Record<string, string>>(() => {
    const initial: Record<string, string> = {};
    filters?.forEach((f) => {
      initial[f.id] = queryParams?.filters?.[f.id] ?? f.value ?? f.defaultValue ?? "all";
    });
    return initial;
  });

  // Sync controlled filter values
  useEffect(() => {
    if (filters || queryParams?.filters) {
      setFilterValues((prev) => {
        let changed = false;
        const next = { ...prev };
        filters?.forEach((f) => {
          const targetVal =
            queryParams?.filters?.[f.id] ?? f.value ?? f.defaultValue ?? "all";
          if (prev[f.id] !== targetVal) {
            next[f.id] = targetVal;
            changed = true;
          }
        });
        return changed ? next : prev;
      });
    }
  }, [filters, queryParams?.filters]);

  // Propagate filter selection changes
  const handleFilterChange = useCallback(
    (filter: TableFilter<T>, val: string) => {
      setFilterValues((prev) => {
        const next = { ...prev, [filter.id]: val };
        if (onQueryChange) {
          onQueryChange({ filters: next, page: 1 });
        }
        return next;
      });
      setCurrentPage(1);
      if (filter.onChange) {
        filter.onChange(val);
      }
    },
    [onQueryChange],
  );

  // Propagate search input updates
  const handleSearchChange = useCallback(
    (val: string) => {
      setSearchQuery(val);
      setCurrentPage(1);
      if (onQueryChange) {
        onQueryChange({ search: val, page: 1 });
      }
    },
    [onQueryChange],
  );

  // Propagate page size updates
  const handlePageSizeChange = useCallback(
    (newLimit: number) => {
      setItemsPerPage(newLimit === -1 ? undefined : newLimit);
      setCurrentPage(1);
      if (onQueryChange) {
        onQueryChange({ limit: newLimit === -1 ? 100 : newLimit, page: 1 });
      }
    },
    [onQueryChange],
  );

  // Propagate page navigation updates
  const handlePageChange = useCallback(
    (newPage: number) => {
      setCurrentPage(newPage);
      if (onQueryChange) {
        onQueryChange({ page: newPage });
      }
    },
    [onQueryChange],
  );

  // Client-side filtering when not server-driven
  const filteredData = useMemo(() => {
    if (isServerDriven) return rawItems;

    let result = rawItems;

    // Apply declarative select filters
    if (filters && filters.length > 0) {
      result = result.filter((item) => {
        for (const f of filters) {
          const selectedVal = f.value !== undefined ? f.value : (filterValues[f.id] ?? "all");
          if (!selectedVal || selectedVal === "all") continue;

          if (f.filterFn) {
            if (!f.filterFn(item, selectedVal)) return false;
          } else if (f.accessorKey) {
            const raw = item[f.accessorKey];
            if (raw === null || raw === undefined) return false;
            if (String(raw).toLowerCase() !== selectedVal.toLowerCase()) {
              return false;
            }
          }
        }
        return true;
      });
    }

    // Apply live search text query
    const trimmed = searchQuery.trim().toLowerCase();
    if (!trimmed) return result;

    const terms = trimmed.split(/\s+/).filter(Boolean);

    return result.filter((item) => {
      if (filterFn) return filterFn(item, trimmed);

      // Check each column's searchable value
      for (const col of columns) {
        if (col.searchable === false) continue;

        let valStr = "";
        if (col.searchValue) {
          valStr = col.searchValue(item).toLowerCase();
        } else if (col.accessorKey) {
          const raw = item[col.accessorKey];
          if (raw !== null && raw !== undefined) {
            valStr = String(raw).toLowerCase();
          }
        }

        if (valStr && terms.every((t) => valStr.includes(t))) {
          return true;
        }
      }

      // Fallback: check all string/number fields of the item
      const itemValues = Object.values(item)
        .filter((v) => typeof v === "string" || typeof v === "number")
        .map((v) => String(v).toLowerCase())
        .join(" ");

      return terms.every((t) => itemValues.includes(t));
    });
  }, [rawItems, isServerDriven, filters, filterValues, searchQuery, filterFn, columns]);

  // Client-side column sorting when not server-driven
  const sortedData = useMemo(() => {
    if (isServerDriven) return filteredData;
    if (!sortConfig) return filteredData;

    const col = columns.find((c, index) => {
      const k =
        c.sortKey ??
        (c.accessorKey as string) ??
        (typeof c.header === "string" ? c.header : `col_${index}`);
      return isSortConfigMatch(sortConfig.key, String(k));
    });

    return [...filteredData].sort((a, b) => {
      let aVal: any;
      let bVal: any;

      if (col?.sortValue) {
        aVal = col.sortValue(a);
        bVal = col.sortValue(b);
      } else if (col?.accessorKey) {
        aVal = a[col.accessorKey];
        bVal = b[col.accessorKey];
      } else {
        aVal = a[sortConfig.key];
        bVal = b[sortConfig.key];
      }

      if (aVal === bVal) return 0;
      if (aVal === null || aVal === undefined) return sortConfig.direction === "asc" ? 1 : -1;
      if (bVal === null || bVal === undefined) return sortConfig.direction === "asc" ? -1 : 1;

      // Handle Date instances
      if (aVal instanceof Date && bVal instanceof Date) {
        return sortConfig.direction === "asc"
          ? aVal.getTime() - bVal.getTime()
          : bVal.getTime() - aVal.getTime();
      }

      // Handle numbers
      if (typeof aVal === "number" && typeof bVal === "number") {
        return sortConfig.direction === "asc" ? aVal - bVal : bVal - aVal;
      }

      // Handle booleans
      if (typeof aVal === "boolean" && typeof bVal === "boolean") {
        return sortConfig.direction === "asc"
          ? aVal === bVal
            ? 0
            : aVal
              ? 1
              : -1
          : aVal === bVal
            ? 0
            : aVal
              ? -1
              : 1;
      }

      // Handle ISO date strings
      if (typeof aVal === "string" && typeof bVal === "string") {
        const isDateLike = /^\d{4}-\d{2}-\d{2}/.test(aVal) && /^\d{4}-\d{2}-\d{2}/.test(bVal);
        if (isDateLike) {
          const aTime = Date.parse(aVal);
          const bTime = Date.parse(bVal);
          if (!isNaN(aTime) && !isNaN(bTime)) {
            return sortConfig.direction === "asc" ? aTime - bTime : bTime - aTime;
          }
        }
      }

      const aStr = String(aVal).toLowerCase();
      const bStr = String(bVal).toLowerCase();
      return sortConfig.direction === "asc" ? aStr.localeCompare(bStr) : bStr.localeCompare(aStr);
    });
  }, [filteredData, isServerDriven, sortConfig, columns]);

  // Total items calculation (uses server totalCount if available)
  const activeItemsPerPage = itemsPerPage ?? (isServerDriven ? rawItems.length || 10 : undefined);

  // When server-side without totalCount, determine if more pages exist based on full page response
  const hasMore = isServerDriven
    ? totalCountValue !== undefined
      ? currentPage < Math.ceil(totalCountValue / (activeItemsPerPage || 10))
      : activeItemsPerPage
        ? rawItems.length === activeItemsPerPage
        : false
    : safeCurrentPageCheck();

  function safeCurrentPageCheck() {
    const calcPages = activeItemsPerPage ? Math.ceil(sortedData.length / activeItemsPerPage) || 1 : 1;
    return currentPage < calcPages;
  }

  const totalPages =
    totalCountValue !== undefined
      ? activeItemsPerPage
        ? Math.ceil(totalCountValue / activeItemsPerPage) || 1
        : 1
      : isServerDriven
        ? hasMore
          ? currentPage + 1
          : Math.max(1, currentPage)
        : activeItemsPerPage
          ? Math.ceil(sortedData.length / activeItemsPerPage) || 1
          : 1;

  const safeCurrentPage =
    totalCountValue !== undefined || !isServerDriven
      ? Math.min(Math.max(1, currentPage), totalPages)
      : Math.max(1, currentPage);

  const totalItems =
    totalCountValue !== undefined
      ? totalCountValue
      : isServerDriven
        ? (safeCurrentPage - 1) * (activeItemsPerPage || 10) + rawItems.length
        : sortedData.length;

  // Clamp current page when total pages shrink (only when total count is known or not server-driven)
  useEffect(() => {
    if ((totalCountValue !== undefined || !isServerDriven) && currentPage > totalPages) {
      setCurrentPage(totalPages);
    }
  }, [currentPage, totalPages, totalCountValue, isServerDriven]);

  // In server-side mode display rawItems directly; in client mode slice current page
  const paginatedData = useMemo(() => {
    if (isServerDriven) return rawItems;
    if (!itemsPerPage) return sortedData;
    const start = (safeCurrentPage - 1) * itemsPerPage;
    return sortedData.slice(start, start + itemsPerPage);
  }, [isServerDriven, rawItems, sortedData, safeCurrentPage, itemsPerPage]);

  const startItem =
    rawItems.length === 0
      ? 0
      : (safeCurrentPage - 1) * (activeItemsPerPage ?? (isServerDriven ? rawItems.length : totalItems)) + 1;
  const endItem =
    rawItems.length === 0
      ? 0
      : isServerDriven && totalCountValue === undefined
        ? startItem + rawItems.length - 1
        : activeItemsPerPage
          ? Math.min(safeCurrentPage * activeItemsPerPage, totalItems)
          : totalItems;

  const isColumnSortable = (col: Column<T>) => {
    if (col.sortable !== undefined) return col.sortable;
    return Boolean(col.sortKey || col.accessorKey || (!isServerDriven && col.sortValue));
  };

  const handleHeaderClick = (col: Column<T>, index: number) => {
    if (!isColumnSortable(col)) return;

    const colKey =
      col.sortKey ??
      (col.accessorKey as string) ??
      (typeof col.header === "string" ? col.header : `col_${index}`);

    const colKeyStr = String(colKey);
    const colSnakeKey = toSnakeCase(colKeyStr);

    const isCurrentKey = isSortConfigMatch(sortConfig?.key, colKeyStr);
    const nextDirection: "asc" | "desc" =
      isCurrentKey && sortConfig?.direction === "asc" ? "desc" : "asc";

    setSortConfig({ key: colSnakeKey, direction: nextDirection });

    if (onQueryChange) {
      const backendSort = mapToSortParam(colKeyStr, nextDirection);
      onQueryChange({ sort: backendSort, page: 1 });
    }
  };

  const hasActiveFilter = Boolean(
    filters?.some((f) => {
      const val = f.value !== undefined ? f.value : filterValues[f.id];
      return val && val !== "all";
    }),
  );

  const isFiltered = searchQuery.trim().length > 0 || hasActiveFilter;
  const showToolbar =
    !hideToolbar &&
    (searchable ||
      title ||
      toolbarActions ||
      !hideCount ||
      Boolean(filters && filters.length > 0));

  const handleResetAll = useCallback(() => {
    setSearchQuery("");
    const resetVals: Record<string, string> = {};
    if (filters) {
      filters.forEach((f) => {
        resetVals[f.id] = "all";
        if (f.onChange) f.onChange("all");
      });
      setFilterValues(resetVals);
    }
    setCurrentPage(1);
    if (onQueryChange) {
      onQueryChange({ search: "", filters: resetVals, page: 1 });
    }
  }, [filters, onQueryChange]);

  return (
    <div className="flex flex-col gap-2.5 w-full">
      {/* Table Toolbar */}
      {showToolbar && (
        <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3 px-1 py-1">
          <div className="flex flex-wrap items-center gap-3">
            {title && (
              <div className="flex flex-col">
                <h3 className="text-sm font-semibold text-brand-snow">{title}</h3>
                {subtitle && <p className="text-xs text-muted-foreground">{subtitle}</p>}
              </div>
            )}

            {searchable && (
              <div className="w-full sm:w-64 max-w-sm">
                <Input
                  type="search"
                  value={searchQuery}
                  onChange={(e) => handleSearchChange(e.target.value)}
                  onClear={() => handleSearchChange("")}
                  placeholder={searchPlaceholder}
                  className="text-xs"
                />
              </div>
            )}

            {/* Declarative Filter Selects */}
            {filters && filters.length > 0 && (
              <div className="flex flex-wrap items-center gap-2">
                {filters.map((f) => {
                  const currentVal =
                    f.value !== undefined ? f.value : (filterValues[f.id] ?? "all");
                  return (
                    <div key={f.id} className="flex items-center gap-1.5">
                      <select
                        aria-label={f.label}
                        value={currentVal}
                        onChange={(e) => handleFilterChange(f, e.target.value)}
                        className={cn(
                          "h-8 rounded-md border bg-background-800/90 px-2.5 py-1 text-xs shadow-xs transition-colors focus:outline-none focus:ring-1 focus:ring-brand-sky/60 cursor-pointer",
                          currentVal !== "all"
                            ? "border-brand-sky/60 bg-brand-sky/10 text-brand-sky font-medium"
                            : "border-ink/15 text-muted-foreground hover:text-foreground",
                        )}
                      >
                        <option value="all">{f.label}: All</option>
                        {f.options.map((opt) => (
                          <option key={opt.value} value={opt.value}>
                            {opt.label}
                          </option>
                        ))}
                      </select>
                    </div>
                  );
                })}
              </div>
            )}

            {!hideCount && (
              <div className="flex items-center gap-1.5">
                <Badge
                  variant={isFiltered ? "info" : "default"}
                  size="sm"
                  className="font-mono text-xs"
                >
                  {isFiltered
                    ? `${rawItems.length} of ${totalItems}`
                    : `${totalItems} records`}
                </Badge>
                {isFiltered && (
                  <Button
                    variant="ghost"
                    size="xs"
                    onClick={handleResetAll}
                    className="text-muted-foreground hover:text-ink gap-1"
                  >
                    <X className="h-3 w-3" />
                    Clear
                  </Button>
                )}
              </div>
            )}
          </div>

          {toolbarActions && (
            <div className="flex items-center gap-2 shrink-0">{toolbarActions}</div>
          )}
        </div>
      )}

      {/* Main Table */}
      <Table className={className} containerClassName={containerClassName}>
        <TableHeader className="sticky top-0 z-10 bg-background-800/90 backdrop-blur border-b border-ink/10">
          <TableRow>
            {columns.map((col, index) => {
              const sortable = isColumnSortable(col);

              const colKey =
                col.sortKey ??
                (col.accessorKey as string) ??
                (typeof col.header === "string" ? col.header : `col_${index}`);
              const colKeyStr = String(colKey);

              const isSortedAsc =
                isSortConfigMatch(sortConfig?.key, colKeyStr) &&
                sortConfig?.direction === "asc";
              const isSortedDesc =
                isSortConfigMatch(sortConfig?.key, colKeyStr) &&
                sortConfig?.direction === "desc";

              return (
                <TableHead
                  key={index}
                  className={cn(
                    sortable &&
                      "cursor-pointer select-none hover:text-ink transition-colors group",
                    col.className,
                    col.headerClassName,
                  )}
                  onClick={() => handleHeaderClick(col, index)}
                >
                  <div className="inline-flex items-center gap-1.5">
                    <span>{col.header}</span>
                    {sortable && (
                      <span className="shrink-0">
                        {isSortedAsc ? (
                          <ArrowUp className="h-3.5 w-3.5 text-brand-sky" />
                        ) : isSortedDesc ? (
                          <ArrowDown className="h-3.5 w-3.5 text-brand-sky" />
                        ) : (
                          <ArrowUpDown className="h-3.5 w-3.5 opacity-25 group-hover:opacity-70 transition-opacity" />
                        )}
                      </span>
                    )}
                  </div>
                </TableHead>
              );
            })}
          </TableRow>
        </TableHeader>

        <TableBody
          className={cn("transition-opacity duration-200", loading && "opacity-50 pointer-events-none")}
        >
          {paginatedData.length === 0 ? (
            <TableRow className="hover:bg-transparent">
              <TableCell colSpan={columns.length} className="text-center py-12">
                {isFiltered ? (
                  <div className="flex flex-col items-center justify-center gap-2">
                    <div className="p-2.5 rounded-full bg-ink/5 border border-ink/10 text-muted-foreground">
                      <SearchX className="h-5 w-5" />
                    </div>
                    <span className="text-xs font-medium text-foreground">
                      No matching records found
                    </span>
                    <span className="text-xs text-muted-foreground">
                      No items matched current search or filter criteria
                    </span>
                    <Button
                      variant="outline"
                      size="xs"
                      onClick={handleResetAll}
                      className="mt-2"
                    >
                      Reset filter
                    </Button>
                  </div>
                ) : (
                  <div className="flex flex-col items-center justify-center gap-2">
                    <div className="p-2.5 rounded-full bg-ink/5 border border-ink/10 text-muted-foreground/50">
                      <Inbox className="h-5 w-5" />
                    </div>
                    <span className="text-xs text-muted-foreground italic">
                      {emptyMessage ?? "Nothing here yet"}
                    </span>
                  </div>
                )}
              </TableCell>
            </TableRow>
          ) : (
            paginatedData.map((item, itemIndex) => (
              <TableRow
                key={keyExtractor ? (keyExtractor(item) || `row_${itemIndex}`) : `row_${itemIndex}`}
                onClick={() => onRowClick && onRowClick(item)}
                className={cn(
                  onRowClick &&
                    "cursor-pointer hover:bg-ink/[0.04] active:bg-ink/[0.06] transition-colors",
                )}
              >
                {columns.map((col, index) => (
                  <TableCell
                    key={index}
                    className={cn("text-ink [&>p]:text-ink", col.className)}
                  >
                    {col.cell ? col.cell(item) : (item[col.accessorKey!] as React.ReactNode)}
                  </TableCell>
                ))}
              </TableRow>
            ))
          )}
        </TableBody>
      </Table>

      {/* Pagination Footer */}
      {paginated && (totalItems > 0 || safeCurrentPage > 1) && (
        <div className="flex flex-col sm:flex-row items-center justify-between gap-3 px-3 py-2 text-xs text-muted-foreground border-t border-ink/10 bg-background-800/30 rounded-b-lg">
          {/* Record range summary */}
          <div className="flex items-center gap-1.5">
            <span>
              Showing <span className="font-medium text-brand-snow">{startItem}</span> to{" "}
              <span className="font-medium text-brand-snow">{endItem}</span>
              {totalCountValue !== undefined ? (
                <>
                  {" of "}
                  <span className="font-medium text-brand-snow">{totalItems}</span> records
                </>
              ) : hasMore ? (
                <>
                  {" of "}
                  <span className="font-medium text-brand-snow">{endItem}+</span> records
                </>
              ) : (
                " records"
              )}
            </span>
          </div>

          {/* Rows per page selector & Page navigation */}
          <div className="flex items-center gap-4 flex-wrap">
            {/* Rows per page selector */}
            <div className="flex items-center gap-1.5">
              <span className="text-xs text-muted-foreground">Rows per page:</span>
              <select
                aria-label="Rows per page"
                value={activeItemsPerPage ?? -1}
                onChange={(e) => handlePageSizeChange(Number(e.target.value))}
                className="h-7 rounded-md border border-ink/15 bg-background-800 px-2 py-0.5 text-xs text-brand-snow shadow-xs focus:outline-none focus:ring-1 focus:ring-brand-sky/60 cursor-pointer"
              >
                {pageSizeOptions.map((opt) => (
                  <option key={opt} value={opt}>
                    {opt}
                  </option>
                ))}
                <option value={-1}>All</option>
              </select>
            </div>

            {/* Page navigation */}
            <div className="flex items-center gap-1">
              <span className="text-xs mr-1">
                Page <span className="font-medium text-brand-snow">{safeCurrentPage}</span>
                {totalCountValue !== undefined ? (
                  <>
                    {" of "}
                    <span className="font-medium text-brand-snow">{totalPages}</span>
                  </>
                ) : hasMore ? null : (
                  <>
                    {" of "}
                    <span className="font-medium text-brand-snow">{safeCurrentPage}</span>
                  </>
                )}
              </span>
              <Button
                variant="outline"
                size="xs"
                disabled={safeCurrentPage <= 1}
                onClick={() => handlePageChange(1)}
                className="h-7 w-7 p-0"
                title="First page"
              >
                <ChevronsLeft className="h-3.5 w-3.5" />
              </Button>
              <Button
                variant="outline"
                size="xs"
                disabled={safeCurrentPage <= 1}
                onClick={() => handlePageChange(Math.max(1, safeCurrentPage - 1))}
                className="h-7 w-7 p-0"
                title="Previous page"
              >
                <ChevronLeft className="h-3.5 w-3.5" />
              </Button>
              <Button
                variant="outline"
                size="xs"
                disabled={
                  isServerDriven && totalCountValue === undefined
                    ? !hasMore
                    : safeCurrentPage >= totalPages
                }
                onClick={() => handlePageChange(safeCurrentPage + 1)}
                className="h-7 w-7 p-0"
                title="Next page"
              >
                <ChevronRight className="h-3.5 w-3.5" />
              </Button>
              {(!isServerDriven || totalCountValue !== undefined) && (
                <Button
                  variant="outline"
                  size="xs"
                  disabled={safeCurrentPage >= totalPages}
                  onClick={() => handlePageChange(totalPages)}
                  className="h-7 w-7 p-0"
                  title="Last page"
                >
                  <ChevronsRight className="h-3.5 w-3.5" />
                </Button>
              )}
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
