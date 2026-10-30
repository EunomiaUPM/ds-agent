// Calls to the catalog agent's policy template and composition API that throw the agent's message
// on failure, and the helpers the policy screens share: versions, texts and parameter values.

import type { QueryClient } from "@tanstack/react-query";
import {
  composeOdrlPolicy,
  previewOdrlPolicyComposition,
} from "./orval/odrl-policies/odrl-policies";
import {
  createPolicyTemplate,
  deletePolicyTemplateByIdAndVersion,
  previewPolicyTemplate,
} from "./orval/policy-templates/policy-templates";
import type {
  ControlPoint,
  LocalizedText,
  NewPolicyCompositionDto,
  NewPolicyTemplateDto,
  OdrlPolicyDto,
  ParameterValue,
  PolicyPart,
  PolicyPreviewDto,
  PolicyTemplateDto,
  PolicyTemplateParameterDefinition,
  TemplatePreviewDto,
} from "./orval/model";
import { CatalogResources } from "./catalogResources";

type Answer = { status: number; data: unknown };

/** The versions of one template, newest first. */
export interface TemplateGroup {
  id: string;
  latest: PolicyTemplateDto;
  versions: PolicyTemplateDto[];
}

/** Values by parameter name; a missing one has no value yet. */
export type ParameterValues = Record<string, ParameterValue | undefined>;

export const CONTROL_POINTS: { value: ControlPoint; label: string; hint: string }[] = [
  { value: "catalog", label: "Catalog", hint: "Who sees the offer in the catalog" },
  { value: "negotiation", label: "Negotiation", hint: "Who may open and agree a contract" },
  { value: "transfer", label: "Transfer", hint: "When a transfer may be requested or started" },
  { value: "dataplane", label: "Dataplane", hint: "Each data request while a transfer runs" },
  { value: "validity", label: "Validity", hint: "How long the agreement lasts" },
  { value: "duties", label: "Duties", hint: "What the consumer must or must not do afterwards" },
];

export const PolicyTemplateApi = {
  async create(input: NewPolicyTemplateDto): Promise<PolicyTemplateDto> {
    const answer = (await createPolicyTemplate(input)) as Answer;
    CatalogResources.check(answer, "save the template");
    return answer.data as PolicyTemplateDto;
  },

  async remove(id: string, version: string): Promise<void> {
    const answer = (await deletePolicyTemplateByIdAndVersion(id, version)) as Answer;
    CatalogResources.check(answer, "delete the template version");
  },

  async preview(request: TemplatePreviewDto): Promise<PolicyPreviewDto> {
    const answer = (await previewPolicyTemplate(request)) as Answer;
    CatalogResources.check(answer, "preview the template");
    return answer.data as PolicyPreviewDto;
  },

  async previewParts(parts: PolicyPart[]): Promise<PolicyPreviewDto> {
    const answer = (await previewOdrlPolicyComposition({ parts })) as Answer;
    CatalogResources.check(answer, "preview the policy");
    return answer.data as PolicyPreviewDto;
  },

  async compose(input: NewPolicyCompositionDto): Promise<OdrlPolicyDto> {
    const answer = (await composeOdrlPolicy(input)) as Answer;
    CatalogResources.check(answer, "add the policy");
    return answer.data as OdrlPolicyDto;
  },

  /** Refetches every template list, version and usage the screens hold. */
  invalidate(queryClient: QueryClient) {
    return queryClient.invalidateQueries({
      predicate: (q) =>
        typeof q.queryKey[0] === "string" &&
        q.queryKey[0].startsWith("/v1/catalog-agent/policy-templates"),
    });
  },

  /** `id@version` of every template the content references directly, in document order. */
  references(content: unknown): string[] {
    const found: string[] = [];
    const walk = (v: unknown) => {
      if (Array.isArray(v)) v.forEach(walk);
      else if (v && typeof v === "object") {
        const ref = (v as Record<string, unknown>)["@ref"];
        if (typeof ref === "string" && !found.includes(ref)) found.push(ref);
        Object.values(v).forEach(walk);
      }
    };
    walk(content);
    return found;
  },

  /** `id@version`, the way references name a template. */
  reference(template: PolicyTemplateDto): string {
    return `${template.id}@${template.version}`;
  },

  /** Versions grouped by template, newest first; the groups keep the order of their latest. */
  group(templates: PolicyTemplateDto[]): TemplateGroup[] {
    const groups = new Map<string, PolicyTemplateDto[]>();
    for (const t of templates) groups.set(t.id, [...(groups.get(t.id) ?? []), t]);
    return [...groups.entries()].map(([id, versions]) => {
      const sorted = [...versions].sort((a, b) => b.date.localeCompare(a.date));
      return { id, latest: sorted[0], versions: sorted };
    });
  },

  /** A localized text in English when there is one, else its first form. */
  text(value?: LocalizedText | null): string | undefined {
    if (value == null) return undefined;
    if (typeof value === "string") return value;
    const list = value as { "@value": string; "@language": string }[];
    return (list.find((t) => t["@language"] === "en") ?? list[0])?.["@value"];
  },

  /** The name of a template to show: its title, else its id. */
  title(template: PolicyTemplateDto): string {
    return PolicyTemplateApi.text(template.title) ?? template.id;
  },

  /** The parameters' defaults, the starting values of a form. */
  defaults(parameters: Record<string, PolicyTemplateParameterDefinition>): ParameterValues {
    return Object.fromEntries(
      Object.entries(parameters).map(([name, def]) => [name, def.defaultValue]),
    );
  },

  /** Parameters that still need a value. */
  missing(
    parameters: Record<string, PolicyTemplateParameterDefinition>,
    values: ParameterValues,
  ): string[] {
    return Object.keys(parameters).filter((name) => {
      const v = values[name];
      return v === undefined || v === "" || (Array.isArray(v) && v.length === 0);
    });
  },

  /** Only the values that are set, as the API takes them. */
  given(values: ParameterValues): Record<string, ParameterValue> {
    return Object.fromEntries(
      Object.entries(values).filter(
        (entry): entry is [string, ParameterValue] => entry[1] !== undefined && entry[1] !== "",
      ),
    );
  },
};
