import { ArrowDown, ArrowUp, ArrowUpDown } from "lucide-react";

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
  const Icon = !active
    ? ArrowUpDown
    : sortConfig!.direction === "asc"
      ? ArrowUp
      : ArrowDown;
  return (
    <span
      onClick={() => onSort(sortKey)}
      className={
        "inline-flex items-center cursor-pointer select-none text-white/80 hover:text-white transition-colors font-semibold" +
        (className ? " " + className : "")
      }
    >
      {label}
      <Icon className={"ml-2 h-3.5 w-3.5 " + (active ? "" : "opacity-50")} />
    </span>
  );
}
