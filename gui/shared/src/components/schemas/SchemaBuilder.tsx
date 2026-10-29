// Builds a JSON Schema: inferred from sample documents, edited as a tree of fields or as JSON,
// tested against the samples, and saved as a new schema or the next version of one.

import React, { useMemo, useState } from "react";
import { toast } from "sonner";
import { CheckCircle2, FlaskConical, Sparkles, XCircle } from "lucide-react";
import { Button } from "shared/src/components/ui/button";
import { Input } from "shared/src/components/ui/input";
import { Textarea } from "shared/src/components/ui/textarea";
import { Checkbox } from "shared/src/components/ui/checkbox";
import { ConfirmDialog } from "shared/src/components/ui/confirm-dialog";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "shared/src/components/ui/tabs";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "shared/src/components/ui/select";
import { FormField } from "shared/src/components/catalog/DcatFields";
import { SchemaFieldTree } from "shared/src/components/schemas/SchemaFieldTree";
import { SchemaApi } from "shared/src/data/schemaApi";
import { SchemaTrees, type SchemaTree } from "shared/src/data/schemas";
import {
  Visibility,
  type SchemaValidationView,
  type SchemaVersionView,
  type SchemaView,
} from "shared/src/data/orval/model";

type Mode = "fields" | "json";

export interface SchemaBuilderProps {
  /** The version to start from; saving stores the next version of its schema. */
  base?: SchemaVersionView;
  onSaved: (view: SchemaView) => void;
  onCancel: () => void;
}

const SAMPLE_HINT = `{"station": "MAD-01", "at": "2026-10-29T10:00:00Z", "kwh": 12.5}`;

export function SchemaBuilder({ base, onSaved, onCancel }: SchemaBuilderProps) {
  const initialTree = useMemo(
    () => (base ? SchemaTrees.fromJsonSchema(base.schema) : SchemaTrees.empty()),
    [base],
  );
  const [title, setTitle] = useState(base?.title ?? "");
  const [description, setDescription] = useState(base?.description ?? "");
  const [visibility, setVisibility] = useState<Visibility>(Visibility.Public);
  const [tree, setTree] = useState<SchemaTree>(initialTree ?? SchemaTrees.empty());
  const [mode, setMode] = useState<Mode>(initialTree ? "fields" : "json");
  // Set when the schema cannot be shown as fields (a non-object root, say).
  const [jsonOnly, setJsonOnly] = useState(!initialTree);
  const [raw, setRaw] = useState(base ? JSON.stringify(base.schema, null, 2) : "");
  const [samplesText, setSamplesText] = useState("");
  const [records, setRecords] = useState(true);
  const [inferred, setInferred] = useState<Record<string, unknown> | null>(null);
  const [report, setReport] = useState<SchemaValidationView | null>(null);
  const [busy, setBusy] = useState<"infer" | "test" | "save" | null>(null);

  const problems = mode === "fields" ? SchemaTrees.problems(tree.fields) : {};

  const current = (): Record<string, unknown> => {
    if (mode === "fields") return SchemaTrees.toJsonSchema({ ...tree, title, description });
    const parsed = JSON.parse(raw);
    if (!parsed || typeof parsed !== "object" || Array.isArray(parsed)) {
      throw new Error("A JSON Schema is a JSON object.");
    }
    return parsed;
  };

  const switchMode = (next: string) => {
    if (next === mode) return;
    if (next === "json") {
      setRaw(JSON.stringify(current(), null, 2));
      setMode("json");
      return;
    }
    try {
      const parsed = SchemaTrees.fromJsonSchema(JSON.parse(raw));
      if (!parsed) {
        setJsonOnly(true);
        toast.error("This schema's root is not an object, so it can only be edited as JSON.");
        return;
      }
      setTree(parsed);
      if (parsed.title) setTitle(parsed.title);
      if (parsed.description) setDescription(parsed.description);
      setJsonOnly(false);
      setMode("fields");
    } catch {
      toast.error("Fix the JSON before going back to the fields.");
    }
  };

  const samples = () => SchemaApi.samples(samplesText, records);

  const apply = (schema: Record<string, unknown>) => {
    const next = SchemaTrees.fromJsonSchema(schema);
    if (next) {
      setTree({ ...next, title, description });
      setJsonOnly(false);
      setMode("fields");
    } else {
      setRaw(JSON.stringify(schema, null, 2));
      setJsonOnly(true);
      setMode("json");
    }
    setReport(null);
  };

  const infer = async () => {
    setBusy("infer");
    try {
      const list = samples();
      if (list.length === 0) throw new Error("Paste at least one sample document.");
      const schema = await SchemaApi.infer(list);
      const hasFields = mode === "fields" ? tree.fields.length > 0 : raw.trim().length > 0;
      if (hasFields) setInferred(schema);
      else apply(schema);
      toast.success(`Inferred from ${list.length} sample${list.length === 1 ? "" : "s"}.`);
    } catch (e) {
      toast.error(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(null);
    }
  };

  const test = async () => {
    setBusy("test");
    try {
      setReport(await SchemaApi.validate(current(), samples()));
    } catch (e) {
      toast.error(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(null);
    }
  };

  const save = async () => {
    if (!title.trim()) return toast.error("Give the schema a title.");
    if (Object.keys(problems).length > 0) return toast.error("Some fields need a name.");
    setBusy("save");
    try {
      const schema = current();
      const saved = base
        ? await SchemaApi.addVersion(base.id, { schema, title, description })
        : await SchemaApi.create({ title, description, schema, visibility });
      toast.success(`Saved version ${saved.version}.`);
      onSaved(saved);
    } catch (e) {
      toast.error(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(null);
    }
  };

  return (
    <div className="grid grid-cols-1 gap-6 xl:grid-cols-[minmax(0,1fr)_22rem]">
      <div className="space-y-5">
        <div className="grid grid-cols-1 gap-4 md:grid-cols-2">
          <FormField label="Title" required>
            <Input
              value={title}
              onChange={(e) => setTitle(e.target.value)}
              placeholder="e.g. Energy readings"
            />
          </FormField>
          {base ? (
            <FormField label="Saves as">
              <p className="pt-2 text-sm text-ink/70">
                Version {base.version + 1} of this schema; version {base.version} keeps its URL and
                content.
              </p>
            </FormField>
          ) : (
            <FormField label="Who can read it" hint="Consumers dereference public schemas.">
              <Select value={visibility} onValueChange={(v) => setVisibility(v as Visibility)}>
                <SelectTrigger className="text-xs">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value={Visibility.Public}>Public, at its URL</SelectItem>
                  <SelectItem value={Visibility.Private}>Private, only here</SelectItem>
                </SelectContent>
              </Select>
            </FormField>
          )}
          <FormField label="Description" className="md:col-span-2">
            <Input
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              placeholder="What documents following it look like"
            />
          </FormField>
        </div>

        <Tabs value={mode} onValueChange={switchMode}>
          <TabsList>
            <TabsTrigger value="fields">
              Fields {mode === "fields" && `(${SchemaTrees.count(tree.fields)})`}
            </TabsTrigger>
            <TabsTrigger value="json">JSON</TabsTrigger>
          </TabsList>
          <TabsContent value="fields" className="pt-4">
            {tree.fields.length === 0 && (
              <p className="mb-3 rounded-md border border-dashed border-ink/15 p-4 text-center text-sm text-ink/55">
                No fields yet. Infer them from a sample, or add them one by one.
              </p>
            )}
            <SchemaFieldTree
              fields={tree.fields}
              onChange={(fields) => setTree({ ...tree, fields })}
              problems={problems}
            />
          </TabsContent>
          <TabsContent value="json" className="pt-4">
            <Textarea
              value={raw}
              onChange={(e) => setRaw(e.target.value)}
              rows={22}
              spellCheck={false}
              placeholder='{ "type": "object", "properties": { … } }'
              className="font-mono text-xs"
            />
            {jsonOnly && (
              <p className="mt-2 text-xs text-ink/55">
                This schema has no object root, so it is edited as JSON only.
              </p>
            )}
          </TabsContent>
        </Tabs>

        <div className="flex justify-end gap-2 border-t border-ink/10 pt-4">
          <Button variant="outline" onClick={onCancel} disabled={busy === "save"}>
            Cancel
          </Button>
          <Button onClick={save} isLoading={busy === "save"}>
            {base ? `Save version ${base.version + 1}` : "Save schema"}
          </Button>
        </div>
      </div>

      <aside className="space-y-4">
        <section className="space-y-3 rounded-lg border border-ink/10 p-4">
          <div>
            <h3 className="text-sm font-semibold">Sample documents</h3>
            <p className="text-xs text-ink/55">
              Paste one JSON document, several as JSON lines, or an array of records. Fields in
              every sample come out required.
            </p>
          </div>
          <Textarea
            value={samplesText}
            onChange={(e) => setSamplesText(e.target.value)}
            rows={10}
            spellCheck={false}
            placeholder={SAMPLE_HINT}
            className="font-mono text-xs"
          />
          <label className="flex items-center gap-2 text-xs text-ink/70">
            <Checkbox checked={records} onCheckedChange={(v) => setRecords(v === true)} />A
            top-level array is a list of records
          </label>
          <div className="flex flex-wrap gap-2">
            <Button size="sm" onClick={infer} isLoading={busy === "infer"}>
              <Sparkles /> Infer schema
            </Button>
            <Button size="sm" variant="outline" onClick={test} isLoading={busy === "test"}>
              <FlaskConical /> Test samples
            </Button>
          </div>
        </section>

        {report && (
          <section className="space-y-2 rounded-lg border border-ink/10 p-4 text-xs">
            <h3 className="flex items-center gap-1.5 text-sm font-semibold">
              {report.valid ? (
                <CheckCircle2 className="h-4 w-4 text-success-600" />
              ) : (
                <XCircle className="h-4 w-4 text-danger-600" />
              )}
              {report.valid ? "Everything checks out" : "Problems found"}
            </h3>
            {report.issues.map((issue, i) => (
              <p key={i} className="text-danger-700 dark:text-danger-300">
                Schema <span className="font-mono">{issue.path}</span>: {issue.message}
              </p>
            ))}
            {report.samples.map((sample) => (
              <div key={sample.index}>
                <p
                  className={
                    sample.valid ? "text-success-700 dark:text-success-300" : "font-medium"
                  }
                >
                  Sample {sample.index + 1}: {sample.valid ? "valid" : "invalid"}
                </p>
                {sample.issues.map((issue, i) => (
                  <p key={i} className="pl-3 text-danger-700 dark:text-danger-300">
                    <span className="font-mono">{issue.path}</span> {issue.message}
                  </p>
                ))}
              </div>
            ))}
            {report.samples.length === 0 && report.issues.length === 0 && (
              <p className="text-ink/55">The schema is valid; paste samples to test them too.</p>
            )}
          </section>
        )}
      </aside>

      <ConfirmDialog
        open={!!inferred}
        onOpenChange={(open) => !open && setInferred(null)}
        title="Replace the current fields?"
        description="The inferred schema replaces what you have edited so far."
        confirmLabel="Replace"
        destructive={false}
        onConfirm={async () => {
          if (inferred) apply(inferred);
          setInferred(null);
        }}
      />
    </div>
  );
}
