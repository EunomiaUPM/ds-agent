// Calls to the catalog agent's schema API that throw the agent's message on failure, and the
// helpers the schema screens share: grouping versions, reading samples, recognising URLs.

import type { QueryClient } from "@tanstack/react-query";
import {
  checkSchemaUrl,
  createSchema,
  createSchemaVersion,
  deleteSchema,
  deleteSchemaVersion,
  getSchemaVersion,
  inferSchema,
  validateSchema,
} from "./orval/schemas/schemas";
import type {
  NewSchemaDto,
  NewSchemaVersionDto,
  SchemaUrlCheckView,
  SchemaValidationView,
  SchemaVersionView,
  SchemaView,
} from "./orval/model";
import { CatalogResources } from "./catalogResources";

type Answer = { status: number; data: unknown };
type JsonSchema = Record<string, unknown>;

/** The versions of one schema, newest first. */
export interface SchemaGroup {
  id: string;
  latest: SchemaView;
  versions: SchemaView[];
}

const HOSTED = /\/public\/schemas\/([^/]+)\/v?(\d+)$/;

export const SchemaApi = {
  async infer(samples: unknown[]): Promise<JsonSchema> {
    const answer = (await inferSchema({ samples })) as Answer;
    CatalogResources.check(answer, "infer a schema");
    return answer.data as JsonSchema;
  },

  async validate(schema: JsonSchema, samples: unknown[] = []): Promise<SchemaValidationView> {
    const answer = (await validateSchema({ schema, samples })) as Answer;
    CatalogResources.check(answer, "check the schema");
    return answer.data as SchemaValidationView;
  },

  async create(input: NewSchemaDto): Promise<SchemaView> {
    const answer = (await createSchema(input)) as Answer;
    CatalogResources.check(answer, "save the schema");
    return answer.data as SchemaView;
  },

  async addVersion(id: string, input: NewSchemaVersionDto): Promise<SchemaView> {
    const answer = (await createSchemaVersion(id, input)) as Answer;
    CatalogResources.check(answer, "save the new version");
    return answer.data as SchemaView;
  },

  async version(id: string, version: number): Promise<SchemaVersionView> {
    const answer = (await getSchemaVersion(id, String(version))) as Answer;
    CatalogResources.check(answer, "load the schema");
    return answer.data as SchemaVersionView;
  },

  /** Deletes one version, or the whole schema when `version` is left out. */
  async remove(id: string, version?: number): Promise<void> {
    const answer = (
      version === undefined
        ? await deleteSchema(id)
        : await deleteSchemaVersion(id, String(version))
    ) as Answer;
    CatalogResources.check(answer, "delete the schema");
  },

  async checkUrl(url: string): Promise<SchemaUrlCheckView> {
    const answer = (await checkSchemaUrl({ url })) as Answer;
    CatalogResources.check(answer, "check the URL");
    return answer.data as SchemaUrlCheckView;
  },

  /** Versions grouped by schema, the most recently changed schema first. */
  group(views: SchemaView[]): SchemaGroup[] {
    const groups = new Map<string, SchemaView[]>();
    for (const view of views) groups.set(view.id, [...(groups.get(view.id) ?? []), view]);
    return [...groups.entries()]
      .map(([id, versions]) => {
        const sorted = [...versions].sort((a, b) => b.version - a.version);
        return { id, latest: sorted[0], versions: sorted };
      })
      .sort((a, b) => b.latest.createdAt.localeCompare(a.latest.createdAt));
  },

  /** The schema id and version a URL of this connector names, if it is one. */
  hosted(url: string): { id: string; version: number } | null {
    const match = HOSTED.exec(url.trim());
    return match ? { id: decodeURIComponent(match[1]), version: Number(match[2]) } : null;
  },

  /**
   * Sample documents from pasted text: one JSON document, newline-delimited JSON, or, with
   * `records`, the items of a top-level array.
   */
  samples(text: string, records: boolean): unknown[] {
    const trimmed = text.trim();
    if (!trimmed) return [];
    try {
      const parsed = JSON.parse(trimmed);
      return records && Array.isArray(parsed) ? parsed : [parsed];
    } catch (whole) {
      const lines = trimmed.split(/\r?\n/).filter((l) => l.trim());
      if (lines.length < 2) throw whole;
      return lines.map((line, i) => {
        try {
          return JSON.parse(line);
        } catch {
          throw new Error(`Line ${i + 1} is not JSON.`);
        }
      });
    }
  },

  invalidate(queryClient: QueryClient) {
    return queryClient.invalidateQueries({
      predicate: (q) =>
        typeof q.queryKey[0] === "string" && q.queryKey[0].startsWith("/v1/catalog-agent/schemas"),
    });
  },
};
