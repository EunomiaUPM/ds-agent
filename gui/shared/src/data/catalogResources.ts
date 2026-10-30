// Writes to the catalog agent's management API from the GUI's editors: documents travel whole,
// edits name the version they started from (`If-Match`) and errors become readable messages.

import type { QueryClient } from "@tanstack/react-query";
import {
  createDataset,
  deleteDataset,
  draftDataset,
  getDatasetById,
  publishDataset,
  updateDataset,
} from "./orval/datasets/datasets";
import {
  createDistribution,
  deleteDistribution,
  draftDistribution,
  getDistributionById,
  publishDistribution,
  updateDistribution,
} from "./orval/distributions/distributions";
import {
  createDataService,
  deleteDataService,
  getDataServiceById,
  updateDataService,
} from "./orval/data-services/data-services";
import { deleteOdrlPolicyById } from "./orval/odrl-policies/odrl-policies";
import type { DcatDocument } from "./dcat";

/** The catalog resources the GUI edits. */
export type CatalogResourceKind = "dataset" | "distribution" | "dataService";

/** A stored document and the version it is at. */
export interface LoadedDocument {
  doc: DcatDocument;
  etag?: string;
}

type Answer = { status: number; data: unknown; headers?: Headers };

const API = {
  dataset: {
    get: getDatasetById,
    create: createDataset,
    update: updateDataset,
    remove: deleteDataset,
  },
  distribution: {
    get: getDistributionById,
    create: createDistribution,
    update: updateDistribution,
    remove: deleteDistribution,
  },
  dataService: {
    get: getDataServiceById,
    create: createDataService,
    update: updateDataService,
    remove: deleteDataService,
  },
} as const;

// Query keys of the generated client are paths; these hold catalog data.
const CATALOG_PATHS = [
  "/catalogs",
  "/datasets",
  "/distributions",
  "/data-services",
  "/odrl-policies",
  "/doctor",
  "/catalog-records",
  "/connector",
];

export const CatalogResources = {
  noun: {
    dataset: "dataset",
    distribution: "distribution",
    dataService: "data service",
  } as Record<CatalogResourceKind, string>,

  async load(kind: CatalogResourceKind, id: string): Promise<LoadedDocument> {
    const answer = (await API[kind].get(id)) as Answer;
    CatalogResources.check(answer, `load the ${CatalogResources.noun[kind]}`);
    return { doc: answer.data as DcatDocument, etag: answer.headers?.get("ETag") ?? undefined };
  },

  async create(kind: CatalogResourceKind, doc: DcatDocument): Promise<DcatDocument> {
    const answer = (await API[kind].create(doc as never)) as Answer;
    CatalogResources.check(answer, `create the ${CatalogResources.noun[kind]}`);
    return answer.data as DcatDocument;
  },

  /** Replaces the document; a 412 means someone saved it after `etag` was read. */
  async save(
    kind: CatalogResourceKind,
    id: string,
    doc: DcatDocument,
    etag?: string,
  ): Promise<DcatDocument> {
    const options = etag ? { headers: { "If-Match": etag } } : undefined;
    const answer = (await API[kind].update(id, doc as never, options)) as Answer;
    CatalogResources.check(answer, `save the ${CatalogResources.noun[kind]}`);
    return answer.data as DcatDocument;
  },

  async remove(kind: CatalogResourceKind, id: string): Promise<void> {
    const answer = (await API[kind].remove(id)) as Answer;
    CatalogResources.check(answer, `delete the ${CatalogResources.noun[kind]}`);
  },

  /** Publishes a dataset or distribution, or keeps it as a draft, out of DSP and DCAT. */
  async setPublication(
    kind: "dataset" | "distribution",
    id: string,
    draft: boolean,
  ): Promise<void> {
    const action =
      kind === "dataset"
        ? draft
          ? draftDataset
          : publishDataset
        : draft
          ? draftDistribution
          : publishDistribution;
    const answer = (await action(id)) as Answer;
    CatalogResources.check(answer, draft ? `move the ${kind} to draft` : `publish the ${kind}`);
  },

  async removePolicy(id: string): Promise<void> {
    const answer = (await deleteOdrlPolicyById(id)) as Answer;
    CatalogResources.check(answer, "delete the policy");
  },

  /** Marks every cached catalog query stale, so open screens refetch. */
  invalidate(queryClient: QueryClient) {
    return queryClient.invalidateQueries({
      predicate: (query) => {
        const key = query.queryKey[0];
        return typeof key === "string" && CATALOG_PATHS.some((path) => key.startsWith(path));
      },
    });
  },

  /** Throws the agent's message when `answer` is not a success. */
  check(answer: Answer, action: string) {
    if (answer.status >= 200 && answer.status < 300) return;
    const body = answer.data as { message?: string; details?: string | null } | string;
    const reason =
      typeof body === "object" && body
        ? [body.message, body.details].filter(Boolean).join(": ")
        : String(body ?? "");
    const hint =
      answer.status === 412
        ? "Someone saved it after you opened it; reload and try again."
        : answer.status === 409
          ? "Other resources still depend on it."
          : "";
    throw new Error(
      [`Could not ${action} (${answer.status}).`, reason, hint].filter(Boolean).join(" "),
    );
  },
};
