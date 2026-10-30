// The policy being built on the canvas: rules, conditions and library pieces nested by position,
// the operations the drag and drop needs, and the conversion to and from template content.

import { PolicyTemplateApi, type ParameterValues } from "shared/src/data/policyTemplateApi";
import type {
  PolicyPart,
  PolicyTemplateDto,
  PolicyTemplateKind,
  PolicyTemplateParameterDefinition,
} from "shared/src/data/orval/model";

export type ListKey = "permission" | "prohibition" | "obligation";
export const LISTS: ListKey[] = ["permission", "prohibition", "obligation"];
export type LogicalOp = "and" | "or" | "xone";
export const LOGICAL_OPS: LogicalOp[] = ["and", "or", "xone"];

/** A library piece placed on the canvas, with its values; `exposed` maps a parameter of the
 * piece to a parameter of the template being written. */
export interface RefNode {
  id: string;
  type: "ref";
  reference: string;
  template?: PolicyTemplateDto;
  values: ParameterValues;
  exposed: Record<string, string>;
}
export interface AtomicNode {
  id: string;
  type: "atomic";
  leftOperand: string;
  operator: string;
  rightOperand: string;
}
export interface LogicalNode {
  id: string;
  type: "logical";
  op: LogicalOp;
  children: ConstraintNode[];
}
export type ConstraintNode = RefNode | AtomicNode | LogicalNode;
export interface RuleNode {
  id: string;
  type: "rule";
  action: string;
  constraints: ConstraintNode[];
}
export type RuleItem = RuleNode | RefNode;

/** Everything on the canvas. In offer mode, `joins` holds constraint pieces added to every rule of
 * a list; in constraint mode only `constraints` is used. */
export interface PolicyTree {
  include: RefNode[];
  permission: RuleItem[];
  prohibition: RuleItem[];
  obligation: RuleItem[];
  joins: Record<ListKey, RefNode[]>;
  constraints: ConstraintNode[];
}

/** Where something can be dropped. */
export type Zone =
  | { kind: "include" }
  | { kind: "lane"; list: ListKey }
  | { kind: "joins"; list: ListKey }
  | { kind: "constraints"; owner?: string };

/** What is being dragged: a library piece, a blank block or something already on the canvas. */
export type Dragged =
  | { source: "library"; reference: string }
  | { source: "block"; block: "rule" | "atomic" | LogicalOp }
  | { source: "node"; id: string };

/** What the canvas is for: an offer, or a template of a kind. */
export type BuilderMode = "offer" | PolicyTemplateKind;

type Library = Map<string, PolicyTemplateDto>;
type Json = Record<string, unknown>;

let counter = 0;
const nextId = () => `n${++counter}`;

export const PolicyTreeOps = {
  empty(): PolicyTree {
    return {
      include: [],
      permission: [],
      prohibition: [],
      obligation: [],
      joins: { permission: [], prohibition: [], obligation: [] },
      constraints: [],
    };
  },

  /** The kind of a library piece. */
  kind(template?: PolicyTemplateDto): PolicyTemplateKind {
    return template?.kind ?? "policy";
  },

  /** The rule list a rule piece belongs to. */
  ruleList(template?: PolicyTemplateDto): ListKey | undefined {
    const content = (template?.content ?? {}) as Json;
    return LISTS.find((l) => l in content);
  },

  ref(template: PolicyTemplateDto): RefNode {
    return {
      id: nextId(),
      type: "ref",
      reference: PolicyTemplateApi.reference(template),
      template,
      values: PolicyTemplateApi.defaults(template.parameters),
      exposed: {},
    };
  },

  block(block: "rule" | "atomic" | LogicalOp): RuleNode | ConstraintNode {
    if (block === "rule") return { id: nextId(), type: "rule", action: "use", constraints: [] };
    if (block === "atomic")
      return { id: nextId(), type: "atomic", leftOperand: "", operator: "eq", rightOperand: "" };
    return { id: nextId(), type: "logical", op: block, children: [] };
  },

  /** Why `node` cannot go in `zone`, or nothing when it can. */
  refusal(zone: Zone, node: RuleNode | ConstraintNode): string | undefined {
    const kind = node.type === "ref" ? PolicyTreeOps.kind(node.template) : undefined;
    switch (zone.kind) {
      case "include":
        return kind === "policy" ? undefined : "Only whole policies are included here.";
      case "joins":
        return kind === "constraint" ? undefined : "Only constraint pieces join every rule.";
      case "lane":
        if (node.type === "rule") return undefined;
        if (kind === "rule")
          return PolicyTreeOps.ruleList(node.type === "ref" ? node.template : undefined) ===
            zone.list
            ? undefined
            : `That rule is not a ${zone.list}.`;
        if (kind === "policy") return "Drop whole policies on Included policies.";
        return "Conditions go inside a rule.";
      case "constraints":
        if (node.type === "rule")
          return "Rules go in a permission, prohibition or obligation list.";
        if (node.type === "ref" && kind !== "constraint") return "Only constraint pieces go here.";
        return undefined;
    }
  },

  /** The node with that id, wherever it is. */
  find(tree: PolicyTree, id: string): RuleNode | ConstraintNode | undefined {
    return PolicyTreeOps.remove(tree, id)[1];
  },

  /** Whether `id` is the node or sits inside it. */
  contains(node: RuleNode | ConstraintNode, id: string): boolean {
    if (node.id === id) return true;
    const children =
      node.type === "rule" ? node.constraints : node.type === "logical" ? node.children : [];
    return children.some((c) => PolicyTreeOps.contains(c, id));
  },

  /** A copy of the tree without the node, and the node; nothing removed when it is not there. */
  remove(tree: PolicyTree, id: string): [PolicyTree, RuleNode | ConstraintNode | undefined] {
    let found: RuleNode | ConstraintNode | undefined;
    const fromConstraints = (list: ConstraintNode[]): ConstraintNode[] =>
      list
        .filter((c) => (c.id === id ? ((found = c), false) : true))
        .map((c) => (c.type === "logical" ? { ...c, children: fromConstraints(c.children) } : c));
    const fromRules = (list: RuleItem[]): RuleItem[] =>
      list
        .filter((r) => (r.id === id ? ((found = r), false) : true))
        .map((r) =>
          r.type === "rule" ? { ...r, constraints: fromConstraints(r.constraints) } : r,
        );
    const next: PolicyTree = {
      include: tree.include.filter((r) => (r.id === id ? ((found = r), false) : true)),
      permission: fromRules(tree.permission),
      prohibition: fromRules(tree.prohibition),
      obligation: fromRules(tree.obligation),
      joins: {
        permission: tree.joins.permission.filter((r) =>
          r.id === id ? ((found = r), false) : true,
        ),
        prohibition: tree.joins.prohibition.filter((r) =>
          r.id === id ? ((found = r), false) : true,
        ),
        obligation: tree.joins.obligation.filter((r) =>
          r.id === id ? ((found = r), false) : true,
        ),
      },
      constraints: fromConstraints(tree.constraints),
    };
    return [next, found];
  },

  /** A copy of the tree with the node added at the end of the zone. */
  insert(tree: PolicyTree, zone: Zone, node: RuleNode | ConstraintNode): PolicyTree {
    switch (zone.kind) {
      case "include":
        return { ...tree, include: [...tree.include, node as RefNode] };
      case "joins":
        return {
          ...tree,
          joins: { ...tree.joins, [zone.list]: [...tree.joins[zone.list], node as RefNode] },
        };
      case "lane":
        return { ...tree, [zone.list]: [...tree[zone.list], node as RuleItem] };
      case "constraints": {
        const c = node as ConstraintNode;
        if (!zone.owner) return { ...tree, constraints: [...tree.constraints, c] };
        return PolicyTreeOps.update(tree, zone.owner, (owner) =>
          owner.type === "rule"
            ? { ...owner, constraints: [...owner.constraints, c] }
            : owner.type === "logical"
              ? { ...owner, children: [...owner.children, c] }
              : owner,
        );
      }
    }
  },

  /** A copy of the tree with the node `id` replaced by what `change` makes of it. */
  update<T extends RuleNode | ConstraintNode>(
    tree: PolicyTree,
    id: string,
    change: (node: T) => RuleNode | ConstraintNode,
  ): PolicyTree {
    const inConstraints = (list: ConstraintNode[]): ConstraintNode[] =>
      list.map((c) => {
        const own = c.id === id ? (change(c as T) as ConstraintNode) : c;
        return own.type === "logical" ? { ...own, children: inConstraints(own.children) } : own;
      });
    const inRules = (list: RuleItem[]): RuleItem[] =>
      list.map((r) => {
        const own = r.id === id ? (change(r as T) as RuleItem) : r;
        return own.type === "rule" ? { ...own, constraints: inConstraints(own.constraints) } : own;
      });
    const inRefs = (list: RefNode[]): RefNode[] =>
      list.map((r) => (r.id === id ? (change(r as T) as RefNode) : r));
    return {
      include: inRefs(tree.include),
      permission: inRules(tree.permission),
      prohibition: inRules(tree.prohibition),
      obligation: inRules(tree.obligation),
      joins: {
        permission: inRefs(tree.joins.permission),
        prohibition: inRefs(tree.joins.prohibition),
        obligation: inRefs(tree.joins.obligation),
      },
      constraints: inConstraints(tree.constraints),
    };
  },

  /** Every library piece on the canvas. */
  refs(tree: PolicyTree): RefNode[] {
    const out: RefNode[] = [];
    const fromConstraints = (list: ConstraintNode[]) =>
      list.forEach((c) =>
        c.type === "ref" ? out.push(c) : c.type === "logical" && fromConstraints(c.children),
      );
    out.push(...tree.include);
    for (const l of LISTS) {
      tree[l].forEach((r) => (r.type === "ref" ? out.push(r) : fromConstraints(r.constraints)));
      out.push(...tree.joins[l]);
    }
    fromConstraints(tree.constraints);
    return out;
  },

  /** What still needs filling: empty rules, conditions and parameter values. */
  problems(tree: PolicyTree, mode: BuilderMode): string[] {
    const out: string[] = [];
    const rules = LISTS.flatMap((l) => tree[l]);
    if (mode === "constraint" && tree.constraints.length !== 1)
      out.push("A constraint template holds exactly one condition.");
    if (mode === "rule" && (rules.length !== 1 || rules[0].type !== "rule"))
      out.push("A rule template holds exactly one rule you write.");
    if ((mode === "policy" || mode === "offer") && rules.length + tree.include.length === 0)
      out.push("Drop a rule or a policy on the canvas.");
    if (rules.some((r) => r.type === "rule" && !r.action)) out.push("Every rule needs an action.");
    const walk = (list: ConstraintNode[]) =>
      list.forEach((c) => {
        if (c.type === "atomic" && (!c.leftOperand || !c.operator || !c.rightOperand))
          out.push("Complete or remove the empty conditions.");
        if (c.type === "logical") {
          if (c.children.length === 0) out.push(`An empty "${c.op}" group needs conditions.`);
          walk(c.children);
        }
      });
    rules.forEach((r) => r.type === "rule" && walk(r.constraints));
    walk(tree.constraints);
    for (const r of PolicyTreeOps.refs(tree)) {
      if (!r.template) {
        out.push(`${r.reference} is not in the library you see.`);
        continue;
      }
      const pending = PolicyTemplateApi.missing(r.template.parameters, r.values).filter(
        (p) => !(p in r.exposed),
      );
      if (pending.length)
        out.push(
          `${PolicyTemplateApi.title(r.template)}: fill in ${pending.map((p) => p.slice(1)).join(", ")}.`,
        );
    }
    return [...new Set(out)];
  },

  /** Template content: references with their values, exposed ones bound to `$name`. */
  content(tree: PolicyTree, mode: BuilderMode): unknown {
    const ref = (r: RefNode): Json => {
      const withValues: Json = { ...PolicyTemplateApi.given(r.values) };
      for (const [child, parent] of Object.entries(r.exposed)) withValues[child] = `$${parent}`;
      return Object.keys(withValues).length
        ? { "@ref": r.reference, with: withValues }
        : { "@ref": r.reference };
    };
    const constraint = (c: ConstraintNode): Json =>
      c.type === "ref"
        ? ref(c)
        : c.type === "atomic"
          ? { leftOperand: c.leftOperand, operator: c.operator, rightOperand: c.rightOperand }
          : { [c.op]: c.children.map(constraint) };
    const rule = (r: RuleItem): Json =>
      r.type === "ref"
        ? ref(r)
        : {
            action: r.action,
            ...(r.constraints.length && { constraint: r.constraints.map(constraint) }),
          };
    if (mode === "constraint") return tree.constraints[0] ? constraint(tree.constraints[0]) : {};
    if (mode === "rule") {
      const list = LISTS.find((l) => tree[l].length) ?? "permission";
      return tree[list][0] ? { [list]: rule(tree[list][0]) } : {};
    }
    const out: Json = {};
    if (tree.include.length) out["@include"] = tree.include.map(ref);
    for (const l of LISTS) if (tree[l].length) out[l] = tree[l].map(rule);
    return out;
  },

  /** An offer's parts: the canvas as one policy, then each constraint joining a list. */
  parts(tree: PolicyTree): PolicyPart[] {
    const joins = LISTS.flatMap((list) =>
      tree.joins[list].map(
        (r): PolicyPart => ({
          "@ref": r.reference,
          with: PolicyTemplateApi.given(r.values),
          into: list,
        }),
      ),
    );
    return [PolicyTreeOps.content(tree, "offer") as PolicyPart, ...joins];
  },

  /** Parameters of the template being written: each exposed parameter, defined as in its piece
   * with the current value as default, and each `$name` typed in a condition, as a string. */
  parameters(tree: PolicyTree): Record<string, PolicyTemplateParameterDefinition> {
    const out: Record<string, PolicyTemplateParameterDefinition> = {};
    const walk = (list: ConstraintNode[]) =>
      list.forEach((c) => {
        if (c.type === "atomic" && /^\$[A-Za-z][\w-]*$/.test(c.rightOperand))
          out[c.rightOperand] ??= { dataType: "string" };
        if (c.type === "logical") walk(c.children);
      });
    LISTS.forEach((l) => tree[l].forEach((r) => r.type === "rule" && walk(r.constraints)));
    walk(tree.constraints);
    for (const r of PolicyTreeOps.refs(tree)) {
      for (const [child, parent] of Object.entries(r.exposed)) {
        const def = r.template?.parameters[child];
        if (!def) continue;
        const current = r.values[child];
        out[`$${parent}`] = {
          ...def,
          ...(current !== undefined && current !== "" && { defaultValue: current }),
        };
      }
    }
    return out;
  },

  /** The tree a template's content describes, its references looked up in the library. */
  parse(content: unknown, mode: BuilderMode, library: Library): PolicyTree {
    const tree = PolicyTreeOps.empty();
    const ref = (o: Json): RefNode => {
      const reference = String(o["@ref"]);
      const template = library.get(reference);
      const values: ParameterValues = {};
      const exposed: Record<string, string> = {};
      for (const [k, v] of Object.entries((o.with ?? {}) as Json)) {
        if (typeof v === "string" && /^\$[A-Za-z]/.test(v)) exposed[k] = v.slice(1);
        else values[k] = v as ParameterValues[string];
      }
      const defaults = template ? PolicyTemplateApi.defaults(template.parameters) : {};
      return {
        id: nextId(),
        type: "ref",
        reference,
        template,
        values: { ...defaults, ...values },
        exposed,
      };
    };
    const constraint = (c: Json): ConstraintNode => {
      if ("@ref" in c) return ref(c);
      const op = LOGICAL_OPS.find((o) => Array.isArray(c[o]));
      if (op)
        return { id: nextId(), type: "logical", op, children: (c[op] as Json[]).map(constraint) };
      return {
        id: nextId(),
        type: "atomic",
        leftOperand: String(c.leftOperand ?? ""),
        operator: String(c.operator ?? "eq"),
        rightOperand:
          typeof c.rightOperand === "string"
            ? c.rightOperand
            : JSON.stringify(c.rightOperand ?? ""),
      };
    };
    const rule = (r: Json): RuleItem =>
      "@ref" in r
        ? ref(r)
        : {
            id: nextId(),
            type: "rule",
            action: String(r.action ?? ""),
            constraints: ((r.constraint ?? []) as Json[]).map(constraint),
          };
    const c = (content ?? {}) as Json;
    if (mode === "constraint") {
      if (Object.keys(c).length) tree.constraints = [constraint(c)];
      return tree;
    }
    if (mode === "rule") {
      const list = LISTS.find((l) => l in c);
      if (list) tree[list] = [rule(c[list] as Json)];
      return tree;
    }
    tree.include = ((c["@include"] ?? []) as Json[]).map(ref);
    for (const l of LISTS) tree[l] = ((c[l] ?? []) as Json[]).map(rule);
    return tree;
  },
};
