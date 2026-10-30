import React, { useState } from "react";
import { ExternalLink } from "lucide-react";
import { FieldGrid } from "shared/src/components/layout/FieldGrid";
import { SchemaUrl } from "shared/src/components/schemas/SchemaPicker";
import {
  DcatForms,
  DcatVocabularies,
  type DcatDocument,
  type VocabularyTerm,
} from "shared/src/data/dcat";

// Each returns null when there is nothing, so the grid shows its dash.
const chips = (values: string[], vocabulary: VocabularyTerm[] = []) =>
  values.length === 0 ? null : (
    <span className="flex flex-wrap gap-1">
      {values.map((v) => (
        <span key={v} title={v} className="rounded bg-ink/[0.07] px-1.5 py-0.5 text-xs">
          {DcatVocabularies.label(vocabulary, v)}
        </span>
      ))}
    </span>
  );

const term = (value: string, vocabulary: VocabularyTerm[]) =>
  value ? <span title={value}>{DcatVocabularies.label(vocabulary, value)}</span> : null;

const link = (href: string) =>
  href ? (
    <a
      href={href}
      target="_blank"
      rel="noreferrer"
      className="inline-flex max-w-full items-center gap-1 break-all text-brand-sky hover:underline"
    >
      {href}
      <ExternalLink className="h-3 w-3 shrink-0" />
    </a>
  ) : null;

const iri = (value: string) =>
  value ? <span className="break-all font-mono text-xs">{value}</span> : null;

/** The DCAT metadata of a dataset document as label/value pairs; empty ones folded away. */
export function DatasetDcatSummary({ doc, action }: { doc: DcatDocument; action?: React.ReactNode }) {
  const [showEmpty, setShowEmpty] = useState(false);
  const d = DcatForms.readDataset(doc);
  const period =
    d.temporalStart || d.temporalEnd ? `${d.temporalStart || "…"} → ${d.temporalEnd || "…"}` : null;
  const contact = [d.contactName, d.contactEmail].filter(Boolean).join(" · ");
  const fields = [
        { label: "Keywords", value: chips(d.keywords) },
        { label: "Themes", value: chips(d.themes, DcatVocabularies.themes) },
        { label: "License", value: term(d.license, DcatVocabularies.licenses) },
        {
          label: "Access rights",
          value: term(d.accessRights, DcatVocabularies.accessRights),
        },
        {
          label: "Spatial coverage",
          value: chips(d.spatial, DcatVocabularies.countries),
        },
        { label: "Period", value: period },
        {
          label: "Languages",
          value: chips(d.languages, DcatVocabularies.languages),
        },
        {
          label: "Update frequency",
          value: term(d.accrualPeriodicity, DcatVocabularies.frequencies),
        },
        { label: "Version", value: d.version || null },
        { label: "Publisher", value: iri(d.publisher) },
        { label: "Creator", value: iri(d.creator) },
        { label: "Contact", value: contact || null },
        { label: "Identifier", value: d.identifier || null },
        { label: "Landing page", value: link(d.landingPage), span: 2 as const },
        {
          label: "Data schema",
          value: d.schema ? <SchemaUrl url={d.schema} /> : null,
          span: 3 as const,
        },
        { label: "Other standard", value: link(d.conformsTo), span: 3 as const },
  ];
  const empty = fields.filter((f) => f.value === null).length;
  return (
    <div className="space-y-2">
      <div className="flex items-center justify-end gap-2">
        {empty > 0 && (
          <button
            type="button"
            onClick={() => setShowEmpty(!showEmpty)}
            className="text-xs text-ink/50 hover:text-ink"
          >
            {showEmpty ? "Hide empty fields" : `Show ${empty} empty fields`}
          </button>
        )}
        {action}
      </div>
      {fields.length - empty > 0 || showEmpty ? (
        <FieldGrid columns={3} fields={fields.map((f) => ({ ...f, hidden: !showEmpty && f.value === null }))} />
      ) : (
        <p className="rounded-lg border border-dashed border-ink/15 p-4 text-sm text-ink/50">
          No DCAT metadata yet beyond the title and description.
        </p>
      )}
    </div>
  );
}
