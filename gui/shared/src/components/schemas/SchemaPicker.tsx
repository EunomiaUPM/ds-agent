// Chooses the data schema of a dataset or distribution: none (or the dataset's, inherited), a
// schema hosted here at a given version, or an external URL that is checked on demand.

import React, { useState } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { CheckCircle2, ExternalLink, Plus, XCircle } from "lucide-react";
import { Button } from "@eunomia/ui";
import { Input } from "@eunomia/ui";
import { SegmentedControl } from "@eunomia/ui";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@eunomia/ui";
import { Sheet, SheetContent, SheetHeader, SheetTitle } from "@eunomia/ui";
import { SchemaBuilder } from "shared/src/components/schemas/SchemaBuilder";
import { SchemaApi } from "shared/src/data/schemaApi";
import { useGetSchemas } from "shared/src/data/orval/schemas/schemas";
import type { PaginatedSchemaView, SchemaUrlCheckView } from "shared/src/data/orval/model";

type Mode = "none" | "hosted" | "external";

export interface SchemaPickerProps {
  /** The schema URL, or empty. */
  value: string;
  onChange: (url: string) => void;
  /** The dataset's schema a distribution gets when it sets none. */
  inherited?: string;
}

export function SchemaPicker({ value, onChange, inherited }: SchemaPickerProps) {
  const derived: Mode = !value ? "none" : SchemaApi.hosted(value) ? "hosted" : "external";
  const [chosen, setChosen] = useState<Mode | null>(null);
  const mode = chosen ?? derived;
  const [building, setBuilding] = useState(false);
  const [check, setCheck] = useState<SchemaUrlCheckView | null>(null);
  const [checking, setChecking] = useState(false);
  const queryClient = useQueryClient();

  const { data } = useGetSchemas({ limit: 100 }, { query: { enabled: mode === "hosted" } });
  const groups =
    data?.status === 200 ? SchemaApi.group((data.data as PaginatedSchemaView).items) : [];
  const hosted = SchemaApi.hosted(value);
  const group = groups.find((g) => g.id === hosted?.id);
  const newer = group && hosted && group.latest.version > hosted.version ? group.latest : null;

  const switchMode = (next: Mode) => {
    setChosen(next);
    setCheck(null);
    if (next === "none") onChange("");
    if (next === "hosted" && !SchemaApi.hosted(value)) onChange("");
    if (next === "external" && SchemaApi.hosted(value)) onChange("");
  };

  const runCheck = async () => {
    setChecking(true);
    try {
      setCheck(await SchemaApi.checkUrl(value));
    } catch (e) {
      setCheck({
        url: value,
        reachable: false,
        isJsonSchema: false,
        issues: [{ path: "/", message: e instanceof Error ? e.message : String(e) }],
      });
    } finally {
      setChecking(false);
    }
  };

  return (
    <div className="space-y-3">
      <SegmentedControl<Mode>
        value={mode}
        onChange={switchMode}
        ariaLabel="Schema source"
        options={[
          { value: "none", label: inherited ? "Inherit" : "None" },
          { value: "hosted", label: "Hosted here", hint: "A schema this connector publishes" },
          { value: "external", label: "External URL", hint: "A schema published elsewhere" },
        ]}
      />

      {mode === "none" && inherited && (
        <p className="text-xs text-ink/60">
          Uses the dataset's schema:{" "}
          <span className="break-all font-mono text-ink/80">{inherited}</span>
        </p>
      )}
      {mode === "none" && !inherited && (
        <p className="text-xs text-ink/55">No schema is published for it.</p>
      )}

      {mode === "hosted" && (
        <div className="space-y-2">
          <div className="flex flex-wrap items-center gap-2">
            <Select
              value={hosted?.id ?? ""}
              onValueChange={(id) => {
                const picked = groups.find((g) => g.id === id);
                if (picked) onChange(picked.latest.url);
              }}
            >
              <SelectTrigger className="w-64 text-xs">
                <SelectValue placeholder={groups.length ? "Choose a schema" : "No schemas yet"} />
              </SelectTrigger>
              <SelectContent>
                {groups.map((g) => (
                  <SelectItem key={g.id} value={g.id}>
                    {g.latest.title}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            {group && (
              <Select
                value={String(hosted?.version ?? group.latest.version)}
                onValueChange={(v) => {
                  const version = group.versions.find((x) => String(x.version) === v);
                  if (version) onChange(version.url);
                }}
              >
                <SelectTrigger className="w-24 text-xs">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {group.versions.map((v) => (
                    <SelectItem key={v.version} value={String(v.version)}>
                      v{v.version}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            )}
            <Button size="sm" variant="outline" onClick={() => setBuilding(true)}>
              <Plus /> New schema
            </Button>
          </div>
          {newer && (
            <p className="flex flex-wrap items-center gap-2 text-xs text-warn-700 dark:text-warn-300">
              Version {newer.version} is available.
              <Button size="xs" variant="ghost" onClick={() => onChange(newer.url)}>
                Use v{newer.version}
              </Button>
            </p>
          )}
          {value && <SchemaUrl url={value} />}
        </div>
      )}

      {mode === "external" && (
        <div className="space-y-2">
          <div className="flex gap-2">
            <Input
              value={value}
              onChange={(e) => {
                setCheck(null);
                onChange(e.target.value);
              }}
              placeholder="https://example.org/schemas/readings.json"
              className="font-mono text-xs"
            />
            <Button
              size="sm"
              variant="outline"
              onClick={runCheck}
              isLoading={checking}
              disabled={!value.trim()}
            >
              Check
            </Button>
          </div>
          {check && (
            <div className="text-xs">
              <p className="flex items-center gap-1.5">
                {check.isJsonSchema ? (
                  <CheckCircle2 className="h-3.5 w-3.5 text-success-600" />
                ) : (
                  <XCircle className="h-3.5 w-3.5 text-danger-600" />
                )}
                {check.isJsonSchema
                  ? `A JSON Schema${check.title ? `: ${check.title}` : ""}.`
                  : check.reachable
                    ? "Reachable, but not a valid JSON Schema."
                    : "Could not be fetched."}
              </p>
              {check.issues.map((issue, i) => (
                <p key={i} className="pl-5 text-ink/60">
                  {issue.message}
                </p>
              ))}
            </div>
          )}
        </div>
      )}

      <Sheet open={building} onOpenChange={setBuilding}>
        <SheetContent className="w-full overflow-y-auto sm:max-w-5xl">
          <SheetHeader className="mb-4">
            <SheetTitle>New schema</SheetTitle>
          </SheetHeader>
          {building && (
            <SchemaBuilder
              onCancel={() => setBuilding(false)}
              onSaved={(view) => {
                setBuilding(false);
                setChosen("hosted");
                onChange(view.url);
                SchemaApi.invalidate(queryClient);
              }}
            />
          )}
        </SheetContent>
      </Sheet>
    </div>
  );
}

/** A schema URL with a link to open it. */
export function SchemaUrl({ url }: { url: string }) {
  return (
    <a
      href={url}
      target="_blank"
      rel="noreferrer"
      className="inline-flex max-w-full items-center gap-1 break-all font-mono text-xs text-brand-sky hover:underline"
    >
      {url}
      <ExternalLink className="h-3 w-3 shrink-0" />
    </a>
  );
}
