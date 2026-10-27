// Catalogs in and out of the catalog agent: a catalog in a profile and RDF syntax (preview or
// download) and DCAT imports, sent raw because Turtle or RDF/XML bodies are not JSON.

import { getApiGatewayBase } from "./orval-mutator";
import { getGetCatalogInProfileUrl } from "./orval/catalog-exchange/catalog-exchange";
import type { Finding, ImportReport, ShaclResult } from "./orval/model";

export type CatalogProfile = "dsp" | "dcat3" | "dcat-ap";
export type ImportProfile = "dcat-ap" | "dcat3";
export type RdfFormat = "jsonld" | "turtle" | "ntriples" | "rdfxml";

export const CATALOG_PROFILES: { key: CatalogProfile; label: string }[] = [
  { key: "dsp", label: "DSP 2025-1" },
  { key: "dcat3", label: "DCAT 3" },
  { key: "dcat-ap", label: "DCAT-AP 3.0.1" },
];

export const RDF_FORMATS: {
  key: RdfFormat;
  label: string;
  mediaType: string;
  extensions: string[];
}[] = [
  {
    key: "jsonld",
    label: "JSON-LD",
    mediaType: "application/ld+json",
    extensions: ["jsonld", "json"],
  },
  { key: "turtle", label: "Turtle", mediaType: "text/turtle", extensions: ["ttl"] },
  { key: "ntriples", label: "N-Triples", mediaType: "application/n-triples", extensions: ["nt"] },
  {
    key: "rdfxml",
    label: "RDF/XML",
    mediaType: "application/rdf+xml",
    extensions: ["rdf", "xml", "owl"],
  },
];

/** SHACL shape groups each import profile offers; the first ones run by default. */
export const SHAPE_GROUPS: Record<
  ImportProfile,
  { key: string; label: string; byDefault: boolean }[]
> = {
  "dcat-ap": [
    { key: "base", label: "Base", byDefault: true },
    { key: "recommended", label: "Recommendations", byDefault: true },
    { key: "deprecated", label: "Deprecated URIs", byDefault: true },
    { key: "range", label: "Ranges (needs typed objects)", byDefault: false },
    { key: "codelists", label: "Code lists (needs EU vocabularies)", byDefault: false },
  ],
  dcat3: [{ key: "base", label: "DCAT 3 rules", byDefault: true }],
};

export interface ImportOptions {
  profile: ImportProfile;
  mode: "upsert" | "create";
  dryRun: boolean;
  target?: string;
  checks?: string[];
}

/** An import's answer: the report, or the error the agent gave instead. */
export interface ImportOutcome {
  status: number;
  report?: ImportReport;
  error?: string;
}

export const CatalogExchange = {
  /** The gateway URL of an API path. */
  url(path: string): string {
    const base = (getApiGatewayBase() || "").replace(/\/$/, "");
    return `${base}${path}`;
  },

  /** The RDF format a file name's extension says, if any. */
  formatOf(fileName: string) {
    const extension = fileName.split(".").pop()?.toLowerCase() ?? "";
    return RDF_FORMATS.find((f) => f.extensions.includes(extension));
  },

  /** Saves the whole catalog in `profile` and `format` as a file. */
  async download(catalogId: string, profile: CatalogProfile, format: RdfFormat): Promise<void> {
    const response = await fetch(
      CatalogExchange.url(
        getGetCatalogInProfileUrl(catalogId, profile, { download: true, format }),
      ),
    );
    if (!response.ok) throw new Error(`Download failed (${response.status})`);
    const disposition = response.headers.get("content-disposition") ?? "";
    const name = /filename="([^"]+)"/.exec(disposition)?.[1] ?? `catalog-${profile}.${format}`;
    const link = document.createElement("a");
    link.href = URL.createObjectURL(await response.blob());
    link.download = name;
    link.click();
    URL.revokeObjectURL(link.href);
  },

  /** Sends `file` to the import route in the syntax its extension says (JSON-LD otherwise). */
  async importDocument(file: File, options: ImportOptions): Promise<ImportOutcome> {
    const params = new URLSearchParams({
      profile: options.profile,
      mode: options.mode,
      dryRun: String(options.dryRun),
    });
    if (options.target) params.set("target", options.target);
    if (options.checks?.length) params.set("checks", options.checks.join(","));
    const mediaType = CatalogExchange.formatOf(file.name)?.mediaType ?? "application/ld+json";
    const response = await fetch(CatalogExchange.url(`/catalogs/import?${params}`), {
      method: "POST",
      headers: { "Content-Type": mediaType },
      body: await file.text(),
    });
    const body = await response.json().catch(() => undefined);
    if (body && "validation" in body)
      return { status: response.status, report: body as ImportReport };
    return {
      status: response.status,
      error:
        [body?.message, body?.details].filter(Boolean).join(": ") ||
        `The agent answered ${response.status}`,
    };
  },

  /** A SHACL result as a doctor finding, to show both alike. */
  finding(result: ShaclResult): Finding {
    return {
      severity: result.severity,
      resource: result.focusNode,
      property: result.path,
      message: result.message,
    };
  },
};
