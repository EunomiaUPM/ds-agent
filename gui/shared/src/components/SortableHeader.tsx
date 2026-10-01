import { ArrowDown, ArrowUp, ArrowUpDown } from "lucide-react";
import { cn } from "shared/src/lib/utils";

export type SortDirection = "asc" | "desc";

export interface SortConfig<K extends string> {
  key: K;
  direction: SortDirection;
}

interface SortableHeaderProps<K extends string> {
  label: string;
  sortKey: K;
  sortConfig: SortConfig<K> | null;
  onSort: (key: K) => void;
  className?: string;
}

export function SortableHeader<K extends string>({
  label,
  sortKey,
  sortConfig,
  onSort,
  className,
}: SortableHeaderProps<K>) {
  const active = sortConfig?.key === sortKey;
  const Icon = !active ? ArrowUpDown : sortConfig!.direction === "asc" ? ArrowUp : ArrowDown;
  return (
    <span
      onClick={() => onSort(sortKey)}
      className={cn(
        "inline-flex items-center cursor-pointer select-none transition-colors font-semibold",
        active ? "text-brand-sky hover:text-brand-sky/90" : "text-foreground/70 hover:text-ink",
        className,
      )}
    >
      {label}
      <Icon
        className={cn(
          "ml-1.5 h-3.5 w-3.5 transition-transform",
          active ? "text-brand-sky" : "opacity-40",
        )}
      />
    </span>
  );
}
