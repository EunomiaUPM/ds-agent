import React from "react";
import { Input } from "@eunomia/ui";
import { Checkbox } from "@eunomia/ui";
import { TagInput } from "@eunomia/ui";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@eunomia/ui";
import { PolicyTemplateApi, type ParameterValues } from "shared/src/data/policyTemplateApi";
import type { PolicyTemplateParameterDefinition } from "shared/src/data/orval/model";

type Definition = PolicyTemplateParameterDefinition;

/** Placeholder of each type that has a textual format to follow. */
const FORMAT: Partial<Record<Definition["dataType"], string>> = {
  dateTime: "2026-12-31T23:59:59Z",
  duration: "P30D",
  iri: "gx:LegalParticipant",
};

/** One input per template parameter, shaped by its type; hidden parameters keep their default. */
export function TemplateParameterFields({
  parameters,
  values,
  onChange,
}: {
  parameters: Record<string, Definition>;
  values: ParameterValues;
  onChange: (values: ParameterValues) => void;
}) {
  const shown = Object.entries(parameters).filter(([, def]) => !def.ui?.hidden);
  if (shown.length === 0) {
    return <p className="text-xs text-ink/50">No parameters.</p>;
  }
  const set = (name: string, value: ParameterValues[string]) =>
    onChange({ ...values, [name]: value });

  return (
    <div className="grid grid-cols-1 gap-3 sm:grid-cols-2">
      {shown.map(([name, def]) => (
        <label key={name} className="flex flex-col gap-1">
          <span className="text-xs font-medium">
            {PolicyTemplateApi.text(def.ui?.label) ?? name.replace(/^\$/, "")}
            <span className="ml-1.5 font-mono text-[10px] text-ink/40">{def.dataType}</span>
          </span>
          <TemplateParameterFields.Field
            def={def}
            value={values[name]}
            onChange={(v) => set(name, v)}
          />
          {def.ui?.description && (
            <span className="text-[11px] text-ink/50">
              {PolicyTemplateApi.text(def.ui.description)}
            </span>
          )}
        </label>
      ))}
    </div>
  );
}

/** The input for one value of a definition's type. */
TemplateParameterFields.Field = function Field({
  def,
  value,
  onChange,
}: {
  def: Definition;
  value: ParameterValues[string];
  onChange: (value: ParameterValues[string]) => void;
}) {
  const placeholder = PolicyTemplateApi.text(def.ui?.placeholder) ?? FORMAT[def.dataType];
  const allowed = (def.restrictions?.values ?? []) as (
    | string
    | { value: string; label?: unknown }
  )[];
  const options = allowed.map((o) =>
    typeof o === "string"
      ? { value: o, label: o }
      : { value: o.value, label: PolicyTemplateApi.text(o.label as never) ?? o.value },
  );

  switch (def.dataType) {
    case "boolean":
      return (
        <span className="flex h-8 items-center">
          <Checkbox checked={value === true} onCheckedChange={(c) => onChange(c === true)} />
        </span>
      );
    case "integer":
    case "float":
      return (
        <Input
          className="h-8"
          type="number"
          step={def.dataType === "integer" ? 1 : "any"}
          min={def.restrictions?.minValue ?? undefined}
          max={def.restrictions?.maxValue ?? undefined}
          value={typeof value === "number" ? value : ""}
          onChange={(e) => onChange(e.target.value === "" ? undefined : Number(e.target.value))}
        />
      );
    case "selection":
      return (
        <Select value={value === undefined ? "" : String(value)} onValueChange={onChange}>
          <SelectTrigger className="h-8">
            <SelectValue placeholder="Choose" />
          </SelectTrigger>
          <SelectContent>
            {options.map((o) => (
              <SelectItem key={o.value} value={o.value}>
                {o.label}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      );
    case "list":
      return (
        <TagInput
          value={Array.isArray(value) ? value : []}
          onChange={onChange}
          options={options}
          placeholder={placeholder ?? "Type and press Enter"}
        />
      );
    case "date":
    case "time":
      return (
        <Input
          className="h-8"
          type={def.dataType}
          value={typeof value === "string" ? value : ""}
          onChange={(e) => onChange(e.target.value || undefined)}
        />
      );
    default:
      return (
        <Input
          className="h-8"
          placeholder={placeholder}
          value={typeof value === "string" ? value : ""}
          onChange={(e) => onChange(e.target.value || undefined)}
        />
      );
  }
};
