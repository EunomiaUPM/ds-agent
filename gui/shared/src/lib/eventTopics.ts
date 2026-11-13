import type {
  EventEnvelope,
  EventLink,
  EventTopicCatalog,
  LiveInvalidations,
} from "@eunomia/ui/events";
import { getApiGatewayBase } from "../data/orval-mutator";

const AREAS: Record<string, string> = {
  transfers: "Transfer",
  negotiations: "Negotiation",
  offers: "Offer",
  agreements: "Agreement",
  protocol: "Protocol",
  catalog: "Catalog",
  dataset: "Dataset",
  dataset_series: "Dataset series",
  distribution: "Distribution",
  data_service: "Data service",
  agent: "Agent",
  catalog_record: "Catalog record",
  schema: "Schema",
  policies: "Policy",
  usage_control: "Usage control",
  dataplane: "Dataplane",
  connector_instances: "Connector instance",
  connector_templates: "Connector template",
  grants: "Grant",
  verifications: "Verification",
  issuances: "Issuance",
  interactions: "Interaction",
  participants: "Participant",
  keystore_secrets: "Keystore secret",
  keystore_parameters: "Keystore parameter",
};
const ACTIONS: Record<string, string> = {
  create: "created",
  edit: "updated",
  delete: "deleted",
  trust_updated: "trust updated",
  state_changed: "state changed",
};
const FILTER_AREAS: Array<[string, string]> = [
  ["transfers", "Transfers"],
  ["negotiations", "Negotiations"],
  ["agreements", "Agreements"],
  ["catalog", "Catalog"],
  ["dataset", "Datasets"],
  ["policies", "Policies"],
  ["usage_control", "Usage control"],
  ["dataplane", "Dataplane"],
  ["connector_instances", "Connectors"],
  ["grants", "Identity grants"],
  ["participants", "Participants"],
  ["keystore_secrets", "Keystore secrets"],
  ["keystore_parameters", "Keystore parameters"],
];
// Entities that only make sense with their area: `Transfer process`, not `Process`.
const GENERIC_ENTITIES = new Set(["process", "message", "pdp", "obligation", "data", "transfer"]);
const ACRONYMS: Record<string, string> = {
  vc: "VC",
  vp: "VP",
  did: "DID",
  dlq: "DLQ",
  odrl: "ODRL",
  pdp: "PDP",
};

/** A page of the GUI with the params of its `$segments`. */
interface Route {
  to: string;
  params: Record<string, string>;
}

/** Readable labels and in-app links for canonical dataspace event topics. */
export const EventTopics: EventTopicCatalog & { route(event: EventEnvelope): Route | undefined } = {
  areas: FILTER_AREAS.map(([prefix, label]) => ({ pattern: `${prefix}:**`, label })),

  /** `transfers:started` → `Transfer started`, `usage_control:pdp:evaluated` → `Usage control PDP evaluated`. */
  describe(topic: string): string {
    const parts = topic.split(":");
    if (parts.length === 2) {
      const [subjectKey, action] = parts;
      const subject = AREAS[subjectKey] ?? subjectKey.replace(/[_-]/g, " ");
      const verb = ACTIONS[action] ?? action.replace(/[_-]/g, " ");
      return `${subject} ${verb}`.trim();
    }
    const [area, entity, action] = parts;
    const words = (entity ?? area).split(/[_-]/).map((w) => ACRONYMS[w] ?? w);
    if (!entity || GENERIC_ENTITIES.has(entity)) words.unshift(AREAS[area] ?? area);
    const verb = action ? (ACTIONS[action] ?? action.replace(/[_-]/g, " ")) : "";
    const label = words.join(" ");
    const cap = label.charAt(0).toUpperCase() + label.slice(1);
    return `${cap} ${verb}`.trim();
  },

  area(topic: string): string {
    return AREAS[topic.split(":")[0]] ?? topic.split(":")[0];
  },

  /** The record the event is about, labelled by its id, when the GUI has a page for it. */
  link(event: EventEnvelope): EventLink | undefined {
    const route = EventTopics.route(event);
    return route && { ...route, label: Object.values(route.params)[0] };
  },

  route(event: EventEnvelope): Route | undefined {
    const p = (event.payload ?? {}) as Record<string, unknown>;
    const ext = event as unknown as {
      subject?: { kind?: string; id?: string } | null;
      correlation?: {
        flow_refs?: {
          transfer_id?: string | null;
          negotiation_id?: string | null;
          agreement_id?: string | null;
          dataset_id?: string | null;
        };
      };
    };
    const ref = (v: unknown) => (typeof v === "string" && v.length > 0 ? v : undefined);
    const refs = ext.correlation?.flow_refs;

    const transfer =
      ref(p.transfer_id) ??
      ref(refs?.transfer_id) ??
      ref(p.transferProcessId) ??
      (ext.subject?.kind === "transfer" ? ref(ext.subject.id) : undefined);
    if (transfer) {
      return { to: "/transfer-process/$transferProcessId", params: { transferProcessId: transfer } };
    }

    const negotiation =
      ref(p.negotiation_id) ??
      ref(refs?.negotiation_id) ??
      ref(p.negotiationAgentProcessId) ??
      (ext.subject?.kind === "negotiation" ? ref(ext.subject.id) : undefined);
    if (negotiation) {
      return { to: "/contract-negotiation/$cnProcess", params: { cnProcess: negotiation } };
    }

    const agreement =
      ref(p.agreement_id) ??
      ref(refs?.agreement_id) ??
      (ext.subject?.kind === "agreement" ? ref(ext.subject.id) : undefined);
    if (agreement) {
      return { to: "/agreements/$agreementId", params: { agreementId: agreement } };
    }

    const id = ref(p.id) ?? ref(p.resource_id) ?? ref(p.policy_id) ?? ref(p.process_id);
    if (id?.startsWith("urn:transfer-process:"))
      return { to: "/transfer-process/$transferProcessId", params: { transferProcessId: id } };
    if (id?.startsWith("urn:negotiation-process:"))
      return { to: "/contract-negotiation/$cnProcess", params: { cnProcess: id } };
    if (id?.startsWith("urn:agreement:"))
      return { to: "/agreements/$agreementId", params: { agreementId: id } };

    const urn = id ?? ref(p["@id"]) ?? ref(ext.subject?.id);
    if (urn?.startsWith("urn:catalog:"))
      return { to: "/catalog/$catalogId", params: { catalogId: urn } };
    if (urn?.startsWith("urn:schema:") || event.topic.startsWith("schema:")) {
      const schemaId = urn?.split("/")[0];
      if (schemaId) return { to: "/my-catalog/schemas/$schemaId", params: { schemaId } };
    }
    if (event.topic.startsWith("policies:") && id) {
      return {
        to: "/my-catalog/policies/$templateId",
        params: { templateId: id.split("@")[0] },
      };
    }
    return undefined;
  },
};

const AUTH_INVALIDATION_KEYS = [
  "/peer-connection",
  "/gate",
  "/mates",
  "/vc-request",
  "peer-connection-sent",
  "gate-received",
];

const CATALOG_INVALIDATION_KEYS = [
  "/catalogs",
  "/datasets",
  "/distributions",
  "/data-services",
  "/odrl-policies",
  "/policy-templates",
];

/** Query key prefixes (orval keys are request paths) refreshed by each topic prefix. */
export const EVENT_INVALIDATIONS: LiveInvalidations = [
  ["transfers:", ["/transfers/"]],
  ["negotiations:", ["/negotiations/"]],
  ["offers:", ["/negotiations/", "/offers"]],
  ["agreements:", ["/agreements"]],
  ["catalog:", CATALOG_INVALIDATION_KEYS],
  ["dataset:", ["/datasets"]],
  ["dataset_series:", ["/catalogs"]],
  ["distribution:", ["/distributions"]],
  ["data_service:", ["/data-services"]],
  ["agent:", ["/catalogs"]],
  ["catalog_record:", ["/catalogs"]],
  ["schema:", ["/schemas"]],
  ["policies:", ["/odrl-policies", "/policy-templates"]],
  ["usage_control:", ["/dataplane", "/connector"]],
  ["dataplane:", ["/dataplane"]],
  ["connector_instances:", ["/connector"]],
  ["connector_templates:", ["/connector"]],
  ["grants:", AUTH_INVALIDATION_KEYS],
  ["verifications:", AUTH_INVALIDATION_KEYS],
  ["issuances:", AUTH_INVALIDATION_KEYS],
  ["interactions:", AUTH_INVALIDATION_KEYS],
  ["participants:", AUTH_INVALIDATION_KEYS],
  ["keystore_secrets:", ["/keystore"]],
  ["keystore_parameters:", ["/keystore"]],
];

/** `ws(s)://…/admin/api/live` from the configured gateway, or this page's origin. */
export const liveEventsUrl = (): string => {
  const base = (getApiGatewayBase() || `${window.location.origin}/admin/api`).replace(/\/$/, "");
  const api = base.endsWith("/admin/api") ? base : `${base}/admin/api`;
  return `${api.replace(/^http/, "ws")}/live`;
};
