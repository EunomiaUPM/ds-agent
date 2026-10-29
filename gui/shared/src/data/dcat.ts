// DCAT metadata as the catalog forms edit it: EU vocabularies, form state and the JSON-LD
// documents of the catalog agent's management API, keeping whatever a form does not show.

import { CATALOG_CONTEXT, text, texts } from "./catalog";

/** An IRI of a controlled vocabulary with the label the forms show. */
export interface VocabularyTerm {
  value: string;
  label: string;
}

const EU = "http://publications.europa.eu/resource/authority";
const IANA = "http://www.iana.org/assignments/media-types";
const XSD_DATE = "http://www.w3.org/2001/XMLSchema#date";

const terms = (base: string, entries: [string, string][]): VocabularyTerm[] =>
  entries.map(([code, label]) => ({ value: `${base}/${code}`, label }));

/** Vocabularies DCAT-AP recommends, trimmed to the terms a dataspace offering needs. */
export const DcatVocabularies = {
  themes: terms(`${EU}/data-theme`, [
    ["AGRI", "Agriculture, fisheries, forestry and food"],
    ["ECON", "Economy and finance"],
    ["EDUC", "Education, culture and sport"],
    ["ENER", "Energy"],
    ["ENVI", "Environment"],
    ["GOVE", "Government and public sector"],
    ["HEAL", "Health"],
    ["INTR", "International issues"],
    ["JUST", "Justice, legal system and public safety"],
    ["REGI", "Regions and cities"],
    ["SOCI", "Population and society"],
    ["TECH", "Science and technology"],
    ["TRAN", "Transport"],
  ]),
  frequencies: terms(`${EU}/frequency`, [
    ["UPDATE_CONT", "Continuously updated"],
    ["HOURLY", "Hourly"],
    ["DAILY", "Daily"],
    ["WEEKLY", "Weekly"],
    ["MONTHLY", "Monthly"],
    ["QUARTERLY", "Quarterly"],
    ["ANNUAL", "Annual"],
    ["IRREG", "Irregular"],
    ["NEVER", "Never"],
    ["UNKNOWN", "Unknown"],
  ]),
  accessRights: terms(`${EU}/access-right`, [
    ["PUBLIC", "Public"],
    ["RESTRICTED", "Restricted"],
    ["NON_PUBLIC", "Non-public"],
  ]),
  licenses: terms(`${EU}/licence`, [
    ["CC_BY_4_0", "CC BY 4.0"],
    ["CC_BYSA_4_0", "CC BY-SA 4.0"],
    ["CC_BYNC_4_0", "CC BY-NC 4.0"],
    ["CC_BYND_4_0", "CC BY-ND 4.0"],
    ["CC0", "CC0 (public domain)"],
    ["ODC_BY", "ODC Attribution"],
    ["ODC_ODBL", "ODC Open Database License"],
    ["ODC_PDDL", "ODC Public Domain Dedication"],
  ]),
  languages: terms(`${EU}/language`, [
    ["ENG", "English"],
    ["SPA", "Spanish"],
    ["FRA", "French"],
    ["DEU", "German"],
    ["ITA", "Italian"],
    ["POR", "Portuguese"],
    ["NLD", "Dutch"],
    ["CAT", "Catalan"],
    ["EUS", "Basque"],
    ["GLG", "Galician"],
  ]),
  countries: terms(`${EU}/country`, [
    ["ESP", "Spain"],
    ["PRT", "Portugal"],
    ["FRA", "France"],
    ["DEU", "Germany"],
    ["ITA", "Italy"],
    ["NLD", "Netherlands"],
    ["BEL", "Belgium"],
    ["AUT", "Austria"],
    ["IRL", "Ireland"],
    ["GRC", "Greece"],
    ["POL", "Poland"],
    ["SWE", "Sweden"],
  ]),
  mediaTypes: terms(IANA, [
    ["application/json", "JSON"],
    ["application/ld+json", "JSON-LD"],
    ["text/csv", "CSV"],
    ["application/vnd.apache.parquet", "Apache Parquet"],
    ["application/xml", "XML"],
    ["text/turtle", "Turtle"],
    ["application/zip", "ZIP"],
    ["application/pdf", "PDF"],
    ["application/octet-stream", "Binary"],
  ]),
  /** `dct:format` values the transfer agent matches a DSP transfer request against. */
  transferFormats: [
    { value: "http+pull", label: "HTTP pull" },
    { value: "http+push", label: "HTTP push" },
  ] as VocabularyTerm[],
  /** The label of `value` in `vocabulary`, or its last IRI segment. */
  label(vocabulary: VocabularyTerm[], value: string): string {
    return (
      vocabulary.find((t) => t.value === value)?.label ??
      value
        .replace(/^mailto:/, "")
        .split(/[/#]/)
        .filter(Boolean)
        .pop() ??
      value
    );
  },
};

export interface DatasetForm {
  title: string;
  description: string;
  keywords: string[];
  themes: string[];
  identifier: string;
  version: string;
  publisher: string;
  creator: string;
  contactName: string;
  contactEmail: string;
  landingPage: string;
  conformsTo: string;
  /** Data schema URL, hosted here or external; its distributions inherit it. */
  schema: string;
  license: string;
  accessRights: string;
  languages: string[];
  spatial: string[];
  temporalStart: string;
  temporalEnd: string;
  accrualPeriodicity: string;
}

export interface DistributionForm {
  title: string;
  description: string;
  format: string;
  mediaType: string;
  accessService: string;
  downloadURL: string;
  license: string;
  byteSize: string;
  /** Own data schema URL; empty inherits the dataset's. */
  schema: string;
}

export interface DataServiceForm {
  title: string;
  description: string;
  endpointURL: string;
  endpointDescription: string;
  conformsTo: string;
  license: string;
  accessRights: string;
}

/** A catalog document as the API sends it: JSON-LD in the catalog context. */
export type DcatDocument = Record<string, unknown>;

/** Field name to what is wrong with it. */
export type FormProblems = Partial<Record<string, string>>;

type Node = Record<string, unknown>;

const IRI = /^[a-z][a-z0-9+.-]*:\S+$/i;
const EMAIL = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;

const first = (value: unknown): Node | undefined => {
  const list = Array.isArray(value) ? value : value == null ? [] : [value];
  const node = list[0];
  return node && typeof node === "object" ? (node as Node) : undefined;
};

const one = (value: string) => value.trim() || undefined;
const many = (values: string[]) => {
  const kept = values.map((v) => v.trim()).filter(Boolean);
  return kept.length > 0 ? kept : undefined;
};

/** Reading and writing the DCAT forms. */
export const DcatForms = {
  emptyDataset(): DatasetForm {
    return {
      title: "",
      description: "",
      keywords: [],
      themes: [],
      identifier: "",
      version: "",
      publisher: "",
      creator: "",
      contactName: "",
      contactEmail: "",
      landingPage: "",
      conformsTo: "",
      schema: "",
      license: "",
      accessRights: "",
      languages: [],
      spatial: [],
      temporalStart: "",
      temporalEnd: "",
      accrualPeriodicity: "",
    };
  },

  emptyDistribution(): DistributionForm {
    return {
      title: "",
      description: "",
      format: "http+pull",
      mediaType: "",
      accessService: "",
      downloadURL: "",
      license: "",
      byteSize: "",
      schema: "",
    };
  },

  emptyDataService(): DataServiceForm {
    return {
      title: "",
      description: "",
      endpointURL: "",
      endpointDescription: "",
      conformsTo: "",
      license: "",
      accessRights: "",
    };
  },

  readDataset(doc: DcatDocument): DatasetForm {
    const temporal = first(doc.temporal);
    const contact = first(doc.contactPoint);
    return {
      title: text(doc.title),
      description: text(doc.description),
      keywords: DcatForms.english(doc.keyword),
      themes: texts(doc.theme),
      identifier: text(doc.identifier),
      version: text(doc.version),
      publisher: text(doc.publisher),
      creator: text(doc.creator),
      contactName: text(contact?.fn),
      contactEmail: text(contact?.hasEmail).replace(/^mailto:/, ""),
      landingPage: texts(doc.landingPage)[0] ?? "",
      conformsTo: texts(doc.conformsTo)[0] ?? "",
      schema: text(doc.schema),
      license: text(doc.license),
      accessRights: text(doc.accessRights),
      languages: texts(doc.language),
      spatial: texts(doc.spatial),
      temporalStart: text(temporal?.startDate).slice(0, 10),
      temporalEnd: text(temporal?.endDate).slice(0, 10),
      accrualPeriodicity: text(doc.accrualPeriodicity),
    };
  },

  readDistribution(doc: DcatDocument): DistributionForm {
    return {
      title: text(doc.title),
      description: text(doc.description),
      format: text(doc.format),
      mediaType: text(doc.mediaType),
      accessService: texts(doc.accessService)[0] ?? "",
      downloadURL: texts(doc.downloadURL)[0] ?? "",
      license: text(doc.license),
      byteSize: text(doc.byteSize),
      schema: text(doc.schema),
    };
  },

  readDataService(doc: DcatDocument): DataServiceForm {
    return {
      title: text(doc.title),
      description: text(doc.description),
      endpointURL: text(doc.endpointURL),
      endpointDescription: text(doc.endpointDescription),
      conformsTo: texts(doc.conformsTo)[0] ?? "",
      license: text(doc.license),
      accessRights: text(doc.accessRights),
    };
  },

  /** `base` with the dataset form written over it; `base` is the stored document on edits. */
  datasetDocument(form: DatasetForm, base: DcatDocument = {}): DcatDocument {
    const contact = DcatForms.contact(form, first(base.contactPoint));
    const temporal = DcatForms.temporal(form, first(base.temporal));
    return {
      ...DcatForms.writable(base, "Dataset"),
      title: DcatForms.withEnglish(base.title, form.title),
      description: DcatForms.withEnglish(base.description, form.description),
      keyword: DcatForms.withEnglish(base.keyword, form.keywords),
      theme: many(form.themes),
      identifier: one(form.identifier),
      version: one(form.version),
      publisher: one(form.publisher),
      creator: one(form.creator),
      contactPoint: DcatForms.replaceFirst(base.contactPoint, contact),
      landingPage: DcatForms.replaceFirstValue(base.landingPage, form.landingPage),
      conformsTo: DcatForms.replaceFirstValue(base.conformsTo, form.conformsTo),
      schema: one(form.schema),
      license: one(form.license),
      accessRights: one(form.accessRights),
      language: many(form.languages),
      spatial: many(form.spatial),
      temporal: DcatForms.replaceFirst(base.temporal, temporal),
      accrualPeriodicity: one(form.accrualPeriodicity),
    };
  },

  distributionDocument(form: DistributionForm, base: DcatDocument = {}): DcatDocument {
    return {
      ...DcatForms.writable(base, "Distribution"),
      title: DcatForms.withEnglish(base.title, form.title),
      description: DcatForms.withEnglish(base.description, form.description),
      format: one(form.format),
      mediaType: one(form.mediaType),
      accessService: DcatForms.replaceFirstValue(base.accessService, form.accessService),
      downloadURL: DcatForms.replaceFirstValue(base.downloadURL, form.downloadURL),
      license: one(form.license),
      byteSize: one(form.byteSize),
      schema: one(form.schema),
    };
  },

  dataServiceDocument(form: DataServiceForm, base: DcatDocument = {}): DcatDocument {
    return {
      ...DcatForms.writable(base, "DataService"),
      title: DcatForms.withEnglish(base.title, form.title),
      description: DcatForms.withEnglish(base.description, form.description),
      endpointURL: one(form.endpointURL),
      endpointDescription: one(form.endpointDescription),
      conformsTo: DcatForms.replaceFirstValue(base.conformsTo, form.conformsTo),
      license: one(form.license),
      accessRights: one(form.accessRights),
    };
  },

  datasetProblems(form: DatasetForm): FormProblems {
    const problems: FormProblems = {};
    if (!form.title.trim()) problems.title = "A dataset needs a title.";
    for (const key of [
      "publisher",
      "creator",
      "landingPage",
      "conformsTo",
      "license",
      "schema",
    ] as const) {
      if (form[key].trim() && !IRI.test(form[key].trim())) {
        problems[key] = "Must be an IRI, such as https://… or did:web:…";
      }
    }
    if (form.contactEmail.trim() && !EMAIL.test(form.contactEmail.trim())) {
      problems.contactEmail = "Not an email address.";
    }
    if (form.temporalStart && form.temporalEnd && form.temporalStart > form.temporalEnd) {
      problems.temporalEnd = "The period ends before it starts.";
    }
    return problems;
  },

  distributionProblems(form: DistributionForm): FormProblems {
    const problems: FormProblems = {};
    if (!form.title.trim()) problems.title = "A distribution needs a title.";
    if (!form.format.trim()) problems.format = "Consumers ask for a distribution by its format.";
    if (form.downloadURL.trim() && !IRI.test(form.downloadURL.trim())) {
      problems.downloadURL = "Must be a URL.";
    }
    if (form.schema.trim() && !IRI.test(form.schema.trim())) {
      problems.schema = "Must be a URL.";
    }
    if (form.byteSize.trim() && !/^\d+$/.test(form.byteSize.trim())) {
      problems.byteSize = "A whole number of bytes.";
    }
    return problems;
  },

  dataServiceProblems(form: DataServiceForm): FormProblems {
    const problems: FormProblems = {};
    if (!form.title.trim()) problems.title = "A data service needs a title.";
    for (const key of ["endpointURL", "endpointDescription", "conformsTo"] as const) {
      if (form[key].trim() && !IRI.test(form[key].trim())) problems[key] = "Must be a URL.";
    }
    return problems;
  },

  /** The English texts of a language map, as a list. */
  english(value: unknown): string[] {
    if (value && typeof value === "object" && !Array.isArray(value) && !("@value" in value)) {
      const map = value as Record<string, unknown>;
      return texts(map.en ?? map["@none"] ?? []);
    }
    return texts(value);
  },

  /** `base` without what the agent sets, under the catalog context. */
  writable(base: DcatDocument, type: string): DcatDocument {
    const { issued: _issued, modified: _modified, ...rest } = base;
    return { ...rest, "@context": CATALOG_CONTEXT, "@type": rest["@type"] ?? type };
  },

  /** A language map with `value` as its English text, keeping the other languages. */
  withEnglish(base: unknown, value: string | string[]) {
    const kept = Array.isArray(value) ? many(value) : one(value);
    const map =
      base && typeof base === "object" && !Array.isArray(base) && !("@value" in base)
        ? { ...(base as Record<string, unknown>) }
        : {};
    delete map["@none"];
    if (kept) map.en = kept;
    else delete map.en;
    return Object.keys(map).length > 0 ? (map as Record<string, string | string[]>) : undefined;
  },

  /** The list in `base` with its first node replaced, or dropped when `node` is undefined. */
  replaceFirst(base: unknown, node: Node | undefined) {
    const rest = (Array.isArray(base) ? base : base == null ? [] : [base]).slice(1);
    const list = node ? [node, ...rest] : rest;
    return list.length > 0 ? list : undefined;
  },

  /** The list of IRIs in `base` with its first one replaced by `value`. */
  replaceFirstValue(base: unknown, value: string) {
    const rest = texts(base).slice(1);
    const list = value.trim() ? [value.trim(), ...rest] : rest;
    return list.length > 0 ? list : undefined;
  },

  contact(form: DatasetForm, base?: Node): Node | undefined {
    const name = form.contactName.trim();
    const email = form.contactEmail.trim();
    if (!name && !email) return undefined;
    const { fn: _fn, hasEmail: _email, ...rest } = base ?? {};
    return {
      "@type": "Kind",
      ...rest,
      fn: name || undefined,
      hasEmail: email ? `mailto:${email}` : undefined,
    };
  },

  temporal(form: DatasetForm, base?: Node): Node | undefined {
    if (!form.temporalStart && !form.temporalEnd) return undefined;
    const date = (value: string) => (value ? { "@value": value, "@type": XSD_DATE } : undefined);
    const { startDate: _start, endDate: _end, ...rest } = base ?? {};
    return {
      "@type": "PeriodOfTime",
      ...rest,
      startDate: date(form.temporalStart),
      endDate: date(form.temporalEnd),
    };
  },
};
