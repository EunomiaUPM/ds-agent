// The JSON-LD contexts the agents' APIs speak, and helpers to write bodies in them.

/** Catalog resources of the catalog agent's management API. */
export const CATALOG_CONTEXT = "https://w3id.org/eunomia/v1/catalog-context.jsonld" as const;
/** Connector templates and instances. */
export const CONNECTOR_CONTEXT = "https://w3id.org/eunomia/v1/connector-context.jsonld" as const;
/** Keystore parameters, secrets and the application config. */
export const KEYSTORE_CONTEXT = "https://w3id.org/eunomia/v1/keystore-context.jsonld" as const;
/** W3C's ODRL context, the one `odrlOffer` travels in. */
export const ODRL_CONTEXT = "http://www.w3.org/ns/odrl.jsonld" as const;

export const JsonLd = {
  /** ODRL rules as the `Offer` the catalog keeps. */
  odrlOffer<T extends object>(rules: T) {
    return { ...rules, "@context": ODRL_CONTEXT, "@type": "Offer" as const };
  },
  /** `node` without its own `@context`, to embed it in a document with another one. */
  embedded<T extends object>(node: T): Omit<T, "@context"> {
    const { "@context": _context, ...rest } = node as T & { "@context"?: unknown };
    return rest;
  },
};
