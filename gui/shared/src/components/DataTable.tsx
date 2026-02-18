/**
 * DataTable.tsx
 *
 * Generic, type-safe table component for displaying tabular data.
 * Supports custom cell renderers, row click handlers, and flexible column configuration.
 *
 * The component uses TypeScript generics to ensure type safety between
 * column definitions and data items.
 *
 * @example
 * // Define columns with type safety
 * const columns: Column<Dataset>[] = [
 *   { header: "Name", accessorKey: "name" },
 *   { header: "Status", cell: (item) => <Badge>{item.status}</Badge> },
 *   { header: "Created", accessorKey: "createdAt", className: "w-32" },
 * ];
 *
 * <DataTable
 *   columns={columns}
 *   data={datasets}
 *   keyExtractor={(item) => item.id}
 *   onRowClick={(item) => navigate(`/datasets/${item.id}`)}
 * />
 */

import React from "react";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "./ui/table";

// =============================================================================
// TYPES
// =============================================================================

/**
 * Column definition for the DataTable.
 *
 * @template T - The type of data items in the table
 */
export interface Column<T> {
  /** Header text displayed in the table header */
  header: string;

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

  /** Additional CSS class for the column (header and cells) */
  className?: string;
}

/**
 * Props for the DataTable component.
 *
 * @template T - The type of data items in the table
 */
export interface DataTableProps<T> {
  /** Column definitions */
  columns: Column<T>[];

  /** Array of data items to display */
  data: T[];

  /** Function to extract a unique key from each item */
  keyExtractor: (item: T) => string;

  /** Optional handler for row clicks */
  onRowClick?: (item: T) => void;

  /** Additional CSS class for the table */
  className?: string;
}

// =============================================================================
// COMPONENT
// =============================================================================

/**
 * Generic table component for displaying structured data.
 *
 * Features:
 * - Type-safe column definitions
 * - Custom cell renderers for complex content
 * - Optional row click handling with visual feedback
 * - Consistent styling across the application
 *
 * @template T - The type of data items
 * @param props - DataTable properties
 * @returns A styled table with the provided data
 */
export function DataTable<T>({
  columns,
  data,
  keyExtractor,
  onRowClick,
  className,
}: DataTableProps<T>) {
  return (
    <Table className={className} containerClassName="overflow-visible">
      {/* Table header */}
      <TableHeader className="sticky top-[4.8rem] z-20 bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60 rounded-t-2xl overflow-hidden">
        <TableRow>
          {columns.map((col, index) => (
            <TableHead key={index} className={col.className}>
              {col.header}
            </TableHead>
          ))}
        </TableRow>
      </TableHeader>

      {/* Table body */}
      <TableBody>
        {data.map((item) => (
          <TableRow
            key={keyExtractor(item)}
            onClick={() => onRowClick && onRowClick(item)}
            className={onRowClick ? "cursor-pointer hover:bg-muted/50" : ""}
          >
            {columns.map((col, index) => (
              <TableCell key={index} className={col.className}>
                {/* Use custom cell renderer if provided, otherwise use accessorKey */}
                {col.cell ? col.cell(item) : (item[col.accessorKey!] as React.ReactNode)}
              </TableCell>
            ))}
          </TableRow>
        ))}
      </TableBody>
    </Table>
  );
}
