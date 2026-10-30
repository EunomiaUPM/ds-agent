import { EventEnvelope } from "../data/orval/model";

const AREAS: Record<string, string> = {
  transfers: "Transfer",
  negotiations: "Negotiation",
  catalog: "Catalog",
  connector: "Connector",
  keystore: "Keystore",
  auth: "Identity",
};
const ACTIONS: Record<string, string> = { create: "created", edit: "updated", delete: "deleted" };
// Entities that only make sense with their area: `Transfer process`, not `Process`.
const GENERIC_ENTITIES = new Set(["process", "message"]);
const ACRONYMS: Record<string, string> = { vc: "VC", vp: "VP", did: "DID", dlq: "DLQ", odrl: "ODRL" };

export interface EventLink {
  to: string;
  params: Record<string, string>;
}

/** Readable labels and in-app links for bus topics shaped `<area>:<entity>:<action>`. */
export const EventTopics = {
  /** Areas offered as feed filters, by topic prefix. */
  areas: Object.entries(AREAS).map(([prefix, label]) => ({ prefix, label })),

  /** `catalog:policy_template:create` → `Policy template created`, `transfers:process:edit` → `Transfer process updated`. */
  describe(topic: string): string {
    const [area, entity, action] = topic.split(":");
    const words = (entity ?? area).split(/[_-]/).map((w) => ACRONYMS[w] ?? w);
    if (!entity || GENERIC_ENTITIES.has(entity)) words.unshift(AREAS[area] ?? area);
    const verb = action ? (ACTIONS[action] ?? action) : "";
    const text = `${words.join(" ")} ${verb}`.trim();
    return text.charAt(0).toUpperCase() + text.slice(1);
  },

  area(topic: string): string {
    return AREAS[topic.split(":")[0]] ?? topic.split(":")[0];
  },

  /** The record the event is about, when the GUI has a page for it. */
  link(event: EventEnvelope): EventLink | undefined {
    const p = (event.payload ?? {}) as Record<string, unknown>;
    const id = typeof p.id === "string" ? p.id : undefined;
    const ref = (v: unknown) => (typeof v === "string" ? v : undefined);

    if (id?.startsWith("urn:transfer-process:"))
      return { to: "/transfer-process/$transferProcessId", params: { transferProcessId: id } };
    const transfer = ref(p.transferProcessId);
    if (transfer)
      return {
        to: "/transfer-process/$transferProcessId",
        params: { transferProcessId: transfer },
      };
    if (id?.startsWith("urn:negotiation-process:"))
      return { to: "/contract-negotiation/$cnProcess", params: { cnProcess: id } };
    if (id?.startsWith("urn:agreement:"))
      return { to: "/agreements/$agreementId", params: { agreementId: id } };
    const urn = id ?? ref(p["@id"]);
    if (urn?.startsWith("urn:catalog:"))
      return { to: "/catalog/$catalogId", params: { catalogId: urn } };
    if (urn?.startsWith("urn:schema:"))
      return { to: "/my-catalog/schemas/$schemaId", params: { schemaId: urn } };
    if (event.topic.startsWith("catalog:policy_template:") && id)
      return { to: "/my-catalog/policies/$templateId", params: { templateId: id } };
    const negotiation = ref(p.negotiationAgentProcessId);
    if (negotiation)
      return { to: "/contract-negotiation/$cnProcess", params: { cnProcess: negotiation } };
    return undefined;
  },
};
