import React, { useState } from "react";
import dayjs from "dayjs";
import { Braces, ChevronRight, Table2 } from "lucide-react";
import { cn } from "../lib/utils";
import { JsonViewer } from "./JsonViewer";
import { Identifier } from "../identity/Identifier";

type Json = null | boolean | number | string | Json[] | { [key: string]: Json };

export interface JsonTableProps {
  value: unknown;
  className?: string;
  /** Nesting levels expanded on first render. */
  expandDepth?: number;
  /** Shows a Table/JSON switch above the table. */
  rawToggle?: boolean;
  /** Text shown when the value is empty. */
  emptyText?: string;
}

/** Any JSON value as a nested key/value table, with an optional switch to the raw JSON. */
export function JsonTable({
  value,
  className,
  expandDepth = 1,
  rawToggle = true,
  emptyText = "No data",
}: JsonTableProps) {
  const [raw, setRaw] = useState(false);
  const data = JsonTable.normalize(value);

  if (JsonTable.isEmpty(data)) {
    return <p className={cn("text-sm text-ink/50 italic", className)}>{emptyText}</p>;
  }

  return (
    <div className={cn("space-y-2", className)}>
      {rawToggle && (
        <div className="flex justify-end">
          <div className="inline-flex rounded-md border border-ink/10 p-0.5 text-xs">
            {[
              { on: false, label: "Table", Icon: Table2 },
              { on: true, label: "JSON", Icon: Braces },
            ].map(({ on, label, Icon }) => (
              <button
                key={label}
                type="button"
                onClick={() => setRaw(on)}
                className={cn(
                  "inline-flex items-center gap-1 rounded px-2 py-1 transition-colors",
                  raw === on ? "bg-ink/10 text-ink font-medium" : "text-ink/55 hover:text-ink",
                )}
              >
                <Icon className="h-3.5 w-3.5" />
                {label}
              </button>
            ))}
          </div>
        </div>
      )}
      {raw ? (
        <JsonViewer value={data} />
      ) : (
        <div className="overflow-hidden rounded-lg border border-ink/10">
          <JsonTable.Node value={data} depth={0} expandDepth={expandDepth} />
        </div>
      )}
    </div>
  );
}

/** Parses JSON strings so stringified payloads also render as tables. */
JsonTable.normalize = (value: unknown): Json => {
  if (typeof value === "string") {
    const t = value.trim();
    if (t.startsWith("{") || t.startsWith("[")) {
      try {
        return JSON.parse(t);
      } catch {
        return value;
      }
    }
  }
  return (value ?? null) as Json;
};

JsonTable.isEmpty = (v: Json): boolean =>
  v === null ||
  v === "" ||
  (Array.isArray(v) && v.length === 0) ||
  (typeof v === "object" && !Array.isArray(v) && Object.keys(v).length === 0);

JsonTable.isComposite = (v: Json): v is Json[] | { [key: string]: Json } =>
  v !== null && typeof v === "object" && !JsonTable.isEmpty(v);

JsonTable.summary = (v: Json[] | { [key: string]: Json }): string =>
  Array.isArray(v)
    ? `${v.length} item${v.length === 1 ? "" : "s"}`
    : `${Object.keys(v).length} field${Object.keys(v).length === 1 ? "" : "s"}`;

/** Rows of an object or array; primitives render inline. */
JsonTable.Node = function Node({
  value,
  depth,
  expandDepth,
}: {
  value: Json;
  depth: number;
  expandDepth: number;
}) {
  if (!JsonTable.isComposite(value)) return <JsonTable.Primitive value={value} />;

  // Arrays of primitives read better as a list of chips than as indexed rows.
  if (Array.isArray(value) && value.every((v) => !JsonTable.isComposite(v))) {
    return (
      <div className="flex flex-wrap gap-1.5 p-2.5">
        {value.map((v, i) => (
          <span key={i} className="rounded-md bg-ink/[0.06] px-2 py-0.5 text-[0.8125rem]">
            <JsonTable.Primitive value={v} />
          </span>
        ))}
      </div>
    );
  }

  const entries: [string, Json][] = Array.isArray(value)
    ? value.map((v, i) => [`#${i + 1}`, v])
    : Object.entries(value);

  return (
    <table className="w-full table-fixed border-collapse text-[0.8125rem]">
      <tbody>
        {entries.map(([k, v]) => (
          <JsonTable.Row key={k} name={k} value={v} depth={depth} expandDepth={expandDepth} />
        ))}
      </tbody>
    </table>
  );
};

JsonTable.Row = function Row({
  name,
  value,
  depth,
  expandDepth,
}: {
  name: string;
  value: Json;
  depth: number;
  expandDepth: number;
}) {
  const composite = JsonTable.isComposite(value);
  const [open, setOpen] = useState(depth < expandDepth);

  if (!composite) {
    return (
      <tr className="border-b border-ink/[0.06] last:border-0 align-top">
        <th
          scope="row"
          className="w-[34%] px-3 py-2 text-left font-mono text-xs font-normal text-ink/55 break-all"
        >
          {name}
        </th>
        <td className="px-3 py-2 break-words">
          <JsonTable.Primitive value={value} />
        </td>
      </tr>
    );
  }

  return (
    <tr className="border-b border-ink/[0.06] last:border-0">
      <td colSpan={2} className="p-0">
        <button
          type="button"
          onClick={() => setOpen(!open)}
          aria-expanded={open}
          className="flex w-full items-center gap-1.5 px-3 py-2 text-left hover:bg-ink/[0.03]"
        >
          <ChevronRight
            className={cn("h-3.5 w-3.5 text-ink/40 transition-transform", open && "rotate-90")}
          />
          <span className="font-mono text-xs text-ink/75">{name}</span>
          <span className="text-xs text-ink/40">{JsonTable.summary(value)}</span>
        </button>
        {open && (
          <div className="mb-2 ml-4 mr-2 overflow-hidden rounded-md border border-ink/[0.08] bg-ink/[0.015]">
            <JsonTable.Node value={value} depth={depth + 1} expandDepth={expandDepth} />
          </div>
        )}
      </td>
    </tr>
  );
};

const ISO_DATE = /^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}/;
const IDENTIFIER = /^(urn:|did:|https?:\/\/)/;

JsonTable.Primitive = function Primitive({ value }: { value: Json }) {
  if (value === null || value === "") return <span className="text-ink/35 italic">empty</span>;
  if (typeof value === "boolean")
    return (
      <span className={cn("font-medium", value ? "text-emerald-500" : "text-ink/50")}>
        {String(value)}
      </span>
    );
  if (typeof value === "number") return <span className="font-mono tabular-nums">{value}</span>;
  if (typeof value === "object") return <span className="text-ink/35 italic">empty</span>;
  if (ISO_DATE.test(value) && dayjs(value).isValid())
    return <span title={value}>{dayjs(value).format("YYYY-MM-DD HH:mm:ss")}</span>;
  if (IDENTIFIER.test(value))
    return value.length > 64 ? (
      <Identifier value={value} tail={16} />
    ) : (
      <span className="font-mono text-xs break-all">{value}</span>
    );
  return <span className="whitespace-pre-wrap">{value}</span>;
};
