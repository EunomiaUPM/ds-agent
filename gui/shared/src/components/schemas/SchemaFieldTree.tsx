// Editor of a schema's fields as a tree: each row names a field, its type and whether it is
// required; details (description, format, null) fold under it, nested fields indent below.

import React, { useState } from "react";
import { ArrowDown, ArrowUp, ChevronRight, Plus, Trash2 } from "lucide-react";
import { cn } from "@eunomia/ui";
import { Button } from "@eunomia/ui";
import { Input } from "@eunomia/ui";
import { Checkbox } from "@eunomia/ui";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@eunomia/ui";
import {
  FIELD_TYPES,
  STRING_FORMATS,
  SchemaTrees,
  type FieldType,
  type SchemaField,
} from "shared/src/data/schemas";

const NO_FORMAT = "__none";

interface TreeProps {
  fields: SchemaField[];
  onChange: (fields: SchemaField[]) => void;
  problems: Record<string, string>;
  depth?: number;
}

/** The fields of one object, with a button to add another. */
export function SchemaFieldTree({ fields, onChange, problems, depth = 0 }: TreeProps) {
  const replace = (index: number, field: SchemaField) =>
    onChange(fields.map((f, i) => (i === index ? field : f)));
  const remove = (index: number) => onChange(fields.filter((_, i) => i !== index));
  const move = (index: number, by: number) => {
    const target = index + by;
    if (target < 0 || target >= fields.length) return;
    const next = [...fields];
    [next[index], next[target]] = [next[target], next[index]];
    onChange(next);
  };

  return (
    <div className={cn("space-y-1.5", depth > 0 && "border-l border-ink/10 pl-4")}>
      {fields.map((field, index) => (
        <FieldRow
          key={field.key}
          field={field}
          problem={problems[field.key]}
          problems={problems}
          depth={depth}
          first={index === 0}
          last={index === fields.length - 1}
          onChange={(f) => replace(index, f)}
          onRemove={() => remove(index)}
          onMove={(by) => move(index, by)}
        />
      ))}
      <Button
        size="xs"
        variant="ghost"
        onClick={() => onChange([...fields, SchemaTrees.field()])}
        className="text-brand-sky"
      >
        <Plus /> {depth === 0 ? "Add field" : "Add nested field"}
      </Button>
    </div>
  );
}

interface RowProps {
  field: SchemaField;
  problem?: string;
  problems: Record<string, string>;
  depth: number;
  first: boolean;
  last: boolean;
  onChange: (field: SchemaField) => void;
  onRemove: () => void;
  onMove: (by: number) => void;
}

function FieldRow({
  field,
  problem,
  problems,
  depth,
  first,
  last,
  onChange,
  onRemove,
  onMove,
}: RowProps) {
  const [open, setOpen] = useState(false);
  const set = <K extends keyof SchemaField>(key: K, value: SchemaField[K]) =>
    onChange({ ...field, [key]: value });
  const nested = SchemaTrees.hasChildren(field);
  const kept = Object.keys(field.extra);

  return (
    <div className="rounded-md border border-ink/10 bg-background">
      <div className="flex flex-wrap items-center gap-2 px-2 py-1.5">
        <button
          type="button"
          onClick={() => setOpen(!open)}
          className="text-ink/50 hover:text-ink"
          aria-label={open ? "Hide details" : "Show details"}
        >
          <ChevronRight className={cn("h-4 w-4 transition-transform", open && "rotate-90")} />
        </button>
        <Input
          value={field.name}
          onChange={(e) => set("name", e.target.value)}
          placeholder="field name"
          className={cn("h-8 w-44 font-mono text-xs", problem && "border-danger-600")}
          title={problem}
        />
        <TypeSelect
          value={field.type}
          onChange={(type) =>
            onChange({ ...field, type, format: type === "string" ? field.format : "" })
          }
        />
        {field.type === "array" && (
          <>
            <span className="text-xs text-ink/50">of</span>
            <TypeSelect
              value={field.itemType}
              exclude="array"
              onChange={(itemType) => set("itemType", itemType)}
            />
          </>
        )}
        <label className="flex items-center gap-1.5 text-xs text-ink/70">
          <Checkbox checked={field.required} onCheckedChange={(v) => set("required", v === true)} />
          required
        </label>
        {field.description && !open && (
          <span className="hidden max-w-[16rem] truncate text-xs text-ink/50 lg:inline">
            {field.description}
          </span>
        )}
        <span className="ml-auto flex items-center">
          <Button
            size="icon_sm"
            variant="ghost"
            disabled={first}
            onClick={() => onMove(-1)}
            title="Move up"
          >
            <ArrowUp />
          </Button>
          <Button
            size="icon_sm"
            variant="ghost"
            disabled={last}
            onClick={() => onMove(1)}
            title="Move down"
          >
            <ArrowDown />
          </Button>
          <Button
            size="icon_sm"
            variant="ghost"
            onClick={onRemove}
            title="Remove field"
            className="text-danger-700 dark:text-danger-300"
          >
            <Trash2 />
          </Button>
        </span>
      </div>
      {problem && (
        <p className="px-9 pb-1.5 text-xs text-danger-700 dark:text-danger-300">{problem}</p>
      )}

      {open && (
        <div className="grid grid-cols-1 gap-3 border-t border-ink/10 px-9 py-3 md:grid-cols-3">
          <div className="space-y-1 md:col-span-2">
            <label className="text-xs font-medium text-ink/60">Description</label>
            <Input
              value={field.description}
              onChange={(e) => set("description", e.target.value)}
              placeholder="What the field holds, units…"
              className="h-8 text-xs"
            />
          </div>
          {field.type === "string" && (
            <div className="space-y-1">
              <label className="text-xs font-medium text-ink/60">Format</label>
              <Select
                value={field.format || NO_FORMAT}
                onValueChange={(v) => set("format", v === NO_FORMAT ? "" : v)}
              >
                <SelectTrigger className="h-8 text-xs">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value={NO_FORMAT}>Any text</SelectItem>
                  {[...new Set([...STRING_FORMATS, ...(field.format ? [field.format] : [])])].map(
                    (f) => (
                      <SelectItem key={f} value={f}>
                        {f}
                      </SelectItem>
                    ),
                  )}
                </SelectContent>
              </Select>
            </div>
          )}
          <label className="flex items-center gap-1.5 text-xs text-ink/70">
            <Checkbox
              checked={field.nullable}
              onCheckedChange={(v) => set("nullable", v === true)}
            />
            may be null
          </label>
          {kept.length > 0 && (
            <p className="text-xs text-ink/50 md:col-span-3">
              Also keeps: <span className="font-mono">{kept.join(", ")}</span> (edit them in JSON).
            </p>
          )}
        </div>
      )}

      {nested && (
        <div className="px-3 pb-2">
          <SchemaFieldTree
            fields={field.children}
            onChange={(children) => set("children", children)}
            problems={problems}
            depth={depth + 1}
          />
        </div>
      )}
    </div>
  );
}

function TypeSelect({
  value,
  onChange,
  exclude,
}: {
  value: FieldType;
  onChange: (type: FieldType) => void;
  exclude?: FieldType;
}) {
  return (
    <Select value={value} onValueChange={(v) => onChange(v as FieldType)}>
      <SelectTrigger className="h-8 w-28 text-xs">
        <SelectValue />
      </SelectTrigger>
      <SelectContent>
        {FIELD_TYPES.filter((t) => t !== exclude).map((t) => (
          <SelectItem key={t} value={t}>
            {t}
          </SelectItem>
        ))}
      </SelectContent>
    </Select>
  );
}
