// Reading the catalog's JSON-LD: management API documents (eunomia catalog context) and DSP
// catalogs from peers, where non-DSP properties come as `dct:` / `dcat:` compact IRIs.

/** Context of the catalog agent's management API documents. */
export const CATALOG_CONTEXT = "https://w3id.org/eunomia/v1/catalog-context.jsonld" as const;

type Literal = string | { "@value"?: unknown; "@language"?: string; "@id"?: string };

/** `value` as a list, whatever JSON-LD shape it came in. */
export const asArray = <T>(value: T | T[] | undefined | null): T[] =>
  value == null ? [] : Array.isArray(value) ? value : [value];

/** Every text of a plain value, a language map or JSON-LD value objects. */
export const texts = (value: unknown): string[] => {
  if (value == null) return [];
  if (typeof value === "string") return [value];
  if (Array.isArray(value)) return value.flatMap(texts);
  if (typeof value === "object") {
    const object = value as Record<string, unknown>;
    if ("@value" in object) return [String(object["@value"])];
    if ("@id" in object) return [String(object["@id"])];
    return Object.values(object).flatMap(texts);
  }
  return [String(value)];
};

/** One text, in `lang` when a language map has it. */
export const text = (value: unknown, lang = "en"): string => {
  if (value && typeof value === "object" && !Array.isArray(value) && !("@value" in value)) {
    const map = value as Record<string, unknown>;
    const preferred = map[lang] ?? map["@none"];
    if (preferred !== undefined) return texts(preferred)[0] ?? "";
  }
  if (Array.isArray(value)) {
    const tagged = (value as Literal[]).find(
      (v) => typeof v === "object" && v["@language"] === lang,
    );
    if (tagged) return texts(tagged)[0] ?? "";
  }
  return texts(value)[0] ?? "";
};

/** The items of a JSON-LD `@graph` answer. */
export const graphItems = <T>(graph?: { "@graph"?: T[] } | null): T[] => graph?.["@graph"] ?? [];

/**
 * A property of a DSP node: its DSP term or the compact DCAT IRI (`dct:title`), as DSP 2025-1
 * writes properties its context has no term for.
 */
export const dspValue = (node: unknown, ...keys: string[]): unknown => {
  if (!node || typeof node !== "object") return undefined;
  const object = node as Record<string, unknown>;
  for (const key of keys) {
    if (object[key] !== undefined) return object[key];
  }
  return undefined;
};

/** The title of a DSP node. */
export const dspTitle = (node: unknown): string =>
  text(dspValue(node, "dct:title", "title", "http://purl.org/dc/terms/title"));

/** The description of a DSP node. */
export const dspDescription = (node: unknown): string =>
  text(dspValue(node, "dct:description", "description", "http://purl.org/dc/terms/description"));

/** The issue date of a DSP node. */
export const dspIssued = (node: unknown): string =>
  text(dspValue(node, "dct:issued", "issued", "http://purl.org/dc/terms/issued"));

/** What every catalog screen shows of a resource. */
export interface ResourceView {
  id: string;
  title: string;
  description: string;
  issued: string;
  modified: string;
}

export interface DatasetView extends ResourceView {
  catalogId: string;
  keywords: string[];
  themes: string[];
}

export interface DataServiceView extends ResourceView {
  catalogId: string;
  endpointURL: string;
  endpointDescription: string;
}

export interface DistributionView extends ResourceView {
  datasetId: string;
  format: string;
  accessService: string;
  accessURL: string;
  mediaType: string;
}

export interface CatalogView extends ResourceView {
  participantId: string;
  homepage: string;
}

/** A management API document, as the generated client types it or not. */
export type CatalogDoc = Record<string, unknown> | undefined | null;
type Doc = CatalogDoc;

const value = (doc: Doc, key: string) => (doc ? doc[key] : undefined);

/** Management API documents as the views the screens read. */
export const CatalogViews = {
  resource(doc: Doc): ResourceView {
    return {
      id: String(value(doc, "@id") ?? ""),
      title: text(value(doc, "title")),
      description: text(value(doc, "description")),
      issued: text(value(doc, "issued")),
      modified: text(value(doc, "modified")),
    };
  },
  catalog(doc: Doc): CatalogView {
    return {
      ...CatalogViews.resource(doc),
      participantId: text(value(doc, "participantId")),
      homepage: text(value(doc, "homepage")),
    };
  },
  dataset(doc: Doc): DatasetView {
    return {
      ...CatalogViews.resource(doc),
      catalogId: text(value(doc, "inCatalog")),
      keywords: texts(value(doc, "keyword")),
      themes: texts(value(doc, "theme")),
    };
  },
  dataService(doc: Doc): DataServiceView {
    return {
      ...CatalogViews.resource(doc),
      catalogId: text(value(doc, "inCatalog")),
      endpointURL: text(value(doc, "endpointURL")),
      endpointDescription: text(value(doc, "endpointDescription")),
    };
  },
  distribution(doc: Doc): DistributionView {
    return {
      ...CatalogViews.resource(doc),
      datasetId: text(value(doc, "ofDataset")),
      format: text(value(doc, "format")),
      accessService: texts(value(doc, "accessService"))[0] ?? "",
      accessURL: texts(value(doc, "accessURL"))[0] ?? "",
      mediaType: text(value(doc, "mediaType")),
    };
  },
  /** The items of a `@graph` answer as views. */
  list<V>(graph: unknown, view: (doc: Doc) => V): V[] {
    return graphItems(graph as { "@graph"?: Doc[] }).map(view);
  },
};

const EUNOMIA = "https://w3id.org/eunomia/v1/context.jsonld" as const;

/** A DSP catalog or dataset node as a peer sent it. */
export type DspNode = Record<string, any>;

/** The catalog RPC: eunomia bodies out, the peer's DSP answer in. */
export const CatalogRpc = {
  /** Asks a peer's catalog; `noCache` asks even when it is cached. */
  catalog(peer: string, noCache = true) {
    return {
      "@context": EUNOMIA,
      "@type": "RpcCatalogRequestMessage" as const,
      associatedAgentPeer: peer,
      noCache,
    };
  },
  /** Asks one dataset of a peer. */
  dataset(peer: string, dataset: string) {
    return {
      "@context": EUNOMIA,
      "@type": "RpcDatasetRequestMessage" as const,
      associatedAgentPeer: peer,
      dataset,
    };
  },
  /** The peer's DSP answer an `RpcCatalogResult` carries. */
  response(result: unknown): DspNode | undefined {
    const response = (result as { response?: unknown } | undefined)?.response;
    return response && typeof response === "object" ? (response as DspNode) : undefined;
  },
};

/** A DSP node's members as the screens read them. */
export const DspViews = {
  resource(node: DspNode | undefined): ResourceView {
    return {
      id: String(node?.["@id"] ?? ""),
      title: dspTitle(node),
      description: dspDescription(node),
      issued: dspIssued(node),
      modified: text(dspValue(node, "dct:modified", "modified")),
    };
  },
  /** The datasets of a DSP catalog, nested catalogs left aside. */
  datasets(catalog: DspNode | undefined): DspNode[] {
    return asArray(catalog?.dataset as DspNode | DspNode[]);
  },
  /** The first data service of a DSP catalog. */
  service(catalog: DspNode | undefined): DspNode | undefined {
    return asArray(catalog?.service as DspNode | DspNode[])[0];
  },
  homepage(node: DspNode | undefined): string {
    return text(dspValue(node, "http://xmlns.com/foaf/0.1/homepage", "foaf:homepage", "homepage"));
  },
};
