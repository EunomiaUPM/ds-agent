// Form fields of DCAT datasets, distributions and data services, shared by the offering
// wizard and the editors. Optional metadata sits in sections that start folded.

import React, { useState } from "react";
import { ChevronDown } from "lucide-react";
import { cn } from "@eunomia/ui";
import { Input } from "@eunomia/ui";
import { Textarea } from "@eunomia/ui";
import { TagInput } from "@eunomia/ui";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@eunomia/ui";
import {
  DcatVocabularies,
  type DataServiceForm,
  type DatasetForm,
  type DistributionForm,
  type FormProblems,
  type VocabularyTerm,
} from "shared/src/data/dcat";
import type { DataServiceView } from "shared/src/data/catalog";
import { SchemaPicker } from "shared/src/components/schemas/SchemaPicker";

interface FieldProps {
  label: string;
  hint?: React.ReactNode;
  error?: string;
  required?: boolean;
  className?: string;
  children: React.ReactNode;
}

/** A labelled control with its hint or error underneath. */
export function FormField({ label, hint, error, required, className, children }: FieldProps) {
  return (
    <div className={cn("space-y-1.5", className)}>
      <label className="flex items-center gap-1 text-xs font-semibold text-foreground">
        {label}
        {required && <span className="text-rose-500">*</span>}
      </label>
      {children}
      {error ? (
        <p className="text-xs text-danger-700 dark:text-danger-300">{error}</p>
      ) : (
        hint && <p className="text-xs text-muted-foreground">{hint}</p>
      )}
    </div>
  );
}

interface SectionProps {
  title: string;
  description?: string;
  defaultOpen?: boolean;
  /** How many fields of the section are filled, shown while folded. */
  filled?: number;
  children: React.ReactNode;
}

/** A foldable group of fields. */
export function FormSection({ title, description, defaultOpen, filled, children }: SectionProps) {
  const [open, setOpen] = useState(!!defaultOpen);
  return (
    <section className="rounded-lg border border-ink/10">
      <button
        type="button"
        onClick={() => setOpen(!open)}
        className="flex w-full items-center justify-between gap-3 px-4 py-3 text-left"
      >
        <span className="min-w-0">
          <span className="block text-sm font-semibold text-foreground">{title}</span>
          {description && (
            <span className="block text-xs text-muted-foreground">{description}</span>
          )}
        </span>
        <span className="flex shrink-0 items-center gap-2 text-xs text-muted-foreground">
          {!open && !!filled && <span>{filled} set</span>}
          <ChevronDown className={cn("h-4 w-4 transition-transform", open && "rotate-180")} />
        </span>
      </button>
      {open && <div className="space-y-4 border-t border-ink/10 px-4 py-4">{children}</div>}
    </section>
  );
}

const NONE = "__none";
const CUSTOM = "__custom";

interface VocabularySelectProps {
  value: string;
  onChange: (value: string) => void;
  options: VocabularyTerm[];
  placeholder?: string;
  /** Whether an IRI outside `options` can be typed. */
  allowCustom?: boolean;
}

/** One term of a vocabulary, none, or an IRI of one's own. */
export function VocabularySelect({
  value,
  onChange,
  options,
  placeholder = "Not set",
  allowCustom = true,
}: VocabularySelectProps) {
  const known = options.some((o) => o.value === value);
  const [chosen, setCustom] = useState(false);
  // A value outside the list, e.g. loaded after mount, is a custom one too.
  const custom = allowCustom && (chosen || (!!value && !known));
  const selected = custom ? CUSTOM : value || NONE;
  return (
    <div className="space-y-1.5">
      <Select
        value={selected}
        onValueChange={(v) => {
          setCustom(v === CUSTOM);
          if (v === NONE) onChange("");
          // Switching to "Other…" from a listed term starts an empty IRI.
          else if (v === CUSTOM) onChange(known ? "" : value);
          else onChange(v);
        }}
      >
        <SelectTrigger className="text-xs">
          <SelectValue placeholder={placeholder} />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value={NONE}>{placeholder}</SelectItem>
          {options.map((o) => (
            <SelectItem key={o.value} value={o.value}>
              {o.label}
            </SelectItem>
          ))}
          {!allowCustom && value && !known && <SelectItem value={value}>{value}</SelectItem>}
          {allowCustom && <SelectItem value={CUSTOM}>Other…</SelectItem>}
        </SelectContent>
      </Select>
      {custom && (
        <Input
          value={value}
          onChange={(e) => onChange(e.target.value)}
          placeholder="https://…"
          className="font-mono text-xs"
        />
      )}
    </div>
  );
}

interface FormProps<F> {
  value: F;
  onChange: (value: F) => void;
  problems?: FormProblems;
}

const count = (...values: (string | string[])[]) =>
  values.filter((v) => (Array.isArray(v) ? v.length > 0 : !!v.trim())).length;

/** Dataset fields: the essentials open, DCAT-AP recommended metadata in folded sections. */
export function DatasetFields({ value: f, onChange, problems = {} }: FormProps<DatasetForm>) {
  const set = <K extends keyof DatasetForm>(key: K, v: DatasetForm[K]) =>
    onChange({ ...f, [key]: v });
  return (
    <div className="space-y-4">
      <FormField
        label="Title"
        required
        error={problems.title}
        hint="How the dataset is named in every catalog of the dataspace."
      >
        <Input
          value={f.title}
          onChange={(e) => set("title", e.target.value)}
          placeholder="e.g. European Renewable Energy Production Q1-2026"
          className="text-sm"
        />
      </FormField>
      <FormField label="Description">
        <Textarea
          rows={3}
          value={f.description}
          onChange={(e) => set("description", e.target.value)}
          placeholder="What the dataset holds, its scope, how it was collected…"
          className="text-sm"
        />
      </FormField>
      <div className="grid grid-cols-1 gap-4 md:grid-cols-2">
        <FormField label="Keywords" hint="Press Enter or comma after each one.">
          <TagInput
            value={f.keywords}
            onChange={(v) => set("keywords", v)}
            placeholder="energy, solar, grid…"
          />
        </FormField>
        <FormField label="Themes" hint="EU data themes; consumers filter by them.">
          <TagInput
            value={f.themes}
            onChange={(v) => set("themes", v)}
            options={DcatVocabularies.themes}
            placeholder="Search a theme…"
          />
        </FormField>
      </div>

      <FormSection
        title="Data schema"
        description="What the data looks like; published as conformsTo and inherited by distributions."
        filled={count(f.schema)}
      >
        <FormField label="Schema" error={problems.schema}>
          <SchemaPicker value={f.schema} onChange={(v) => set("schema", v)} />
        </FormField>
      </FormSection>

      <FormSection
        title="Licensing & access"
        description="Terms a consumer reads before negotiating."
        filled={count(f.license, f.accessRights)}
      >
        <div className="grid grid-cols-1 gap-4 md:grid-cols-2">
          <FormField label="License" error={problems.license}>
            <VocabularySelect
              value={f.license}
              onChange={(v) => set("license", v)}
              options={DcatVocabularies.licenses}
            />
          </FormField>
          <FormField label="Access rights">
            <VocabularySelect
              value={f.accessRights}
              onChange={(v) => set("accessRights", v)}
              options={DcatVocabularies.accessRights}
              allowCustom={false}
            />
          </FormField>
        </div>
      </FormSection>

      <FormSection
        title="Coverage"
        description="Where and when the data applies, its languages and how often it changes."
        filled={count(f.spatial, f.temporalStart, f.temporalEnd, f.languages, f.accrualPeriodicity)}
      >
        <FormField label="Spatial coverage" hint="Countries, or any place IRI.">
          <TagInput
            value={f.spatial}
            onChange={(v) => set("spatial", v)}
            options={DcatVocabularies.countries}
            placeholder="Search a country…"
          />
        </FormField>
        <div className="grid grid-cols-1 gap-4 md:grid-cols-2">
          <FormField label="Period from">
            <Input
              type="date"
              value={f.temporalStart}
              onChange={(e) => set("temporalStart", e.target.value)}
              className="text-sm"
            />
          </FormField>
          <FormField label="Period to" error={problems.temporalEnd}>
            <Input
              type="date"
              value={f.temporalEnd}
              onChange={(e) => set("temporalEnd", e.target.value)}
              className="text-sm"
            />
          </FormField>
          <FormField label="Languages">
            <TagInput
              value={f.languages}
              onChange={(v) => set("languages", v)}
              options={DcatVocabularies.languages}
              placeholder="Search a language…"
            />
          </FormField>
          <FormField label="Update frequency">
            <VocabularySelect
              value={f.accrualPeriodicity}
              onChange={(v) => set("accrualPeriodicity", v)}
              options={DcatVocabularies.frequencies}
              allowCustom={false}
            />
          </FormField>
        </div>
      </FormSection>

      <FormSection
        title="Provenance & contact"
        description="Who publishes the data and whom to ask about it."
        filled={count(f.publisher, f.creator, f.contactName, f.contactEmail, f.landingPage)}
      >
        <div className="grid grid-cols-1 gap-4 md:grid-cols-2">
          <FormField label="Publisher" error={problems.publisher} hint="DID or URL.">
            <Input
              value={f.publisher}
              onChange={(e) => set("publisher", e.target.value)}
              placeholder="did:web:provider.example.org"
              className="font-mono text-xs"
            />
          </FormField>
          <FormField label="Creator" error={problems.creator} hint="DID or URL.">
            <Input
              value={f.creator}
              onChange={(e) => set("creator", e.target.value)}
              placeholder="did:web:lab.example.org"
              className="font-mono text-xs"
            />
          </FormField>
          <FormField label="Contact name">
            <Input
              value={f.contactName}
              onChange={(e) => set("contactName", e.target.value)}
              placeholder="Data stewardship team"
              className="text-sm"
            />
          </FormField>
          <FormField label="Contact email" error={problems.contactEmail}>
            <Input
              type="email"
              value={f.contactEmail}
              onChange={(e) => set("contactEmail", e.target.value)}
              placeholder="data@example.org"
              className="text-sm"
            />
          </FormField>
          <FormField
            label="Landing page"
            error={problems.landingPage}
            hint="A web page describing the dataset."
            className="md:col-span-2"
          >
            <Input
              value={f.landingPage}
              onChange={(e) => set("landingPage", e.target.value)}
              placeholder="https://example.org/datasets/energy"
              className="font-mono text-xs"
            />
          </FormField>
        </div>
      </FormSection>

      <FormSection
        title="Identification & versioning"
        filled={count(f.identifier, f.version, f.conformsTo)}
      >
        <div className="grid grid-cols-1 gap-4 md:grid-cols-2">
          <FormField label="Identifier" hint="Your own reference, e.g. an internal code.">
            <Input
              value={f.identifier}
              onChange={(e) => set("identifier", e.target.value)}
              placeholder="ENERGY-2026-Q1"
              className="font-mono text-xs"
            />
          </FormField>
          <FormField label="Version">
            <Input
              value={f.version}
              onChange={(e) => set("version", e.target.value)}
              placeholder="1.0.0"
              className="font-mono text-xs"
            />
          </FormField>
          <FormField
            label="Other standard (conforms to)"
            error={problems.conformsTo}
            hint="A model, ontology or rule the data follows; its schema goes above."
            className="md:col-span-2"
          >
            <Input
              value={f.conformsTo}
              onChange={(e) => set("conformsTo", e.target.value)}
              placeholder="https://schema.example.org/energy/v1"
              className="font-mono text-xs"
            />
          </FormField>
        </div>
      </FormSection>
    </div>
  );
}

interface DistributionFieldsProps extends FormProps<DistributionForm> {
  dataServices: DataServiceView[];
  /** The dataset's schema, used when the distribution sets none. */
  inheritedSchema?: string;
}

/** Distribution fields: how the data is transferred and what it looks like. */
export function DistributionFields({
  value: f,
  onChange,
  problems = {},
  dataServices,
  inheritedSchema,
}: DistributionFieldsProps) {
  const set = <K extends keyof DistributionForm>(key: K, v: DistributionForm[K]) =>
    onChange({ ...f, [key]: v });
  return (
    <div className="space-y-4">
      <FormField label="Title" required error={problems.title}>
        <Input
          value={f.title}
          onChange={(e) => set("title", e.target.value)}
          placeholder="e.g. Hourly readings over HTTP"
          className="text-sm"
        />
      </FormField>
      <FormField label="Description">
        <Input
          value={f.description}
          onChange={(e) => set("description", e.target.value)}
          placeholder="e.g. JSON payload served by the pull endpoint"
          className="text-sm"
        />
      </FormField>
      <div className="grid grid-cols-1 gap-4 md:grid-cols-2">
        <FormField
          label="Transfer format"
          required
          error={problems.format}
          hint="dct:format; a DSP transfer request names it."
        >
          <VocabularySelect
            value={f.format}
            onChange={(v) => set("format", v)}
            options={DcatVocabularies.transferFormats}
            placeholder="Choose a format"
          />
        </FormField>
        <FormField label="Media type" hint="What the payload is (IANA).">
          <VocabularySelect
            value={f.mediaType}
            onChange={(v) => set("mediaType", v)}
            options={DcatVocabularies.mediaTypes}
          />
        </FormField>
        <FormField
          label="Data service"
          hint="Left empty, the connector's main data service serves it."
          className="md:col-span-2"
        >
          <Select
            value={f.accessService || NONE}
            onValueChange={(v) => set("accessService", v === NONE ? "" : v)}
          >
            <SelectTrigger className="text-xs">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value={NONE}>Main data service</SelectItem>
              {dataServices.map((ds) => (
                <SelectItem key={ds.id} value={ds.id}>
                  {ds.title || ds.id}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </FormField>
      </div>
      <FormSection
        title="Schema"
        description="Leave it on Inherit to use the dataset's; set one when this serialization differs."
        filled={count(f.schema)}
      >
        <FormField label="Schema" error={problems.schema}>
          <SchemaPicker
            value={f.schema}
            onChange={(v) => set("schema", v)}
            inherited={inheritedSchema || undefined}
          />
        </FormField>
      </FormSection>
      <FormSection title="More details" filled={count(f.license, f.downloadURL, f.byteSize)}>
        <div className="grid grid-cols-1 gap-4 md:grid-cols-2">
          <FormField label="License" hint="When it differs from the dataset's.">
            <VocabularySelect
              value={f.license}
              onChange={(v) => set("license", v)}
              options={DcatVocabularies.licenses}
            />
          </FormField>
          <FormField label="Size in bytes" error={problems.byteSize}>
            <Input
              inputMode="numeric"
              value={f.byteSize}
              onChange={(e) => set("byteSize", e.target.value)}
              placeholder="1048576"
              className="font-mono text-xs"
            />
          </FormField>
          <FormField
            label="Download URL"
            error={problems.downloadURL}
            hint="Only for files anyone may download outside a transfer."
            className="md:col-span-2"
          >
            <Input
              value={f.downloadURL}
              onChange={(e) => set("downloadURL", e.target.value)}
              placeholder="https://example.org/files/sample.csv"
              className="font-mono text-xs"
            />
          </FormField>
        </div>
      </FormSection>
    </div>
  );
}

/** Data service fields: the endpoint consumers reach. */
export function DataServiceFields({
  value: f,
  onChange,
  problems = {},
}: FormProps<DataServiceForm>) {
  const set = <K extends keyof DataServiceForm>(key: K, v: DataServiceForm[K]) =>
    onChange({ ...f, [key]: v });
  return (
    <div className="space-y-4">
      <FormField label="Title" required error={problems.title}>
        <Input value={f.title} onChange={(e) => set("title", e.target.value)} className="text-sm" />
      </FormField>
      <FormField label="Description">
        <Textarea
          rows={2}
          value={f.description}
          onChange={(e) => set("description", e.target.value)}
          className="text-sm"
        />
      </FormField>
      <FormField label="Endpoint URL" error={problems.endpointURL}>
        <Input
          value={f.endpointURL}
          onChange={(e) => set("endpointURL", e.target.value)}
          placeholder="https://provider.example.org/api/v1/dsp"
          className="font-mono text-xs"
        />
      </FormField>
      <FormField
        label="Endpoint description"
        error={problems.endpointDescription}
        hint="URL of an OpenAPI or similar description."
      >
        <Input
          value={f.endpointDescription}
          onChange={(e) => set("endpointDescription", e.target.value)}
          className="font-mono text-xs"
        />
      </FormField>
      <FormSection title="More details" filled={count(f.conformsTo, f.license, f.accessRights)}>
        <FormField label="Conforms to" error={problems.conformsTo}>
          <Input
            value={f.conformsTo}
            onChange={(e) => set("conformsTo", e.target.value)}
            className="font-mono text-xs"
          />
        </FormField>
        <div className="grid grid-cols-1 gap-4 md:grid-cols-2">
          <FormField label="License">
            <VocabularySelect
              value={f.license}
              onChange={(v) => set("license", v)}
              options={DcatVocabularies.licenses}
            />
          </FormField>
          <FormField label="Access rights">
            <VocabularySelect
              value={f.accessRights}
              onChange={(v) => set("accessRights", v)}
              options={DcatVocabularies.accessRights}
              allowCustom={false}
            />
          </FormField>
        </div>
      </FormSection>
    </div>
  );
}
