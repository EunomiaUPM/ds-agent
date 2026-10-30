import React, { useMemo, useState } from "react";
import {
  DndContext,
  DragOverlay,
  KeyboardSensor,
  PointerSensor,
  pointerWithin,
  rectIntersection,
  useDndContext,
  useDraggable,
  useDroppable,
  useSensor,
  useSensors,
  type CollisionDetection,
  type DragEndEvent,
  type DragStartEvent,
} from "@dnd-kit/core";
import {
  Ban,
  Braces,
  CheckCircle2,
  ClipboardList,
  GitMerge,
  GripVertical,
  Library,
  Link2,
  Plus,
  Puzzle,
  Trash2,
} from "lucide-react";
import { toast } from "sonner";
import { cn } from "shared/src/lib/utils";
import { Button } from "shared/src/components/ui/button";
import { Badge } from "shared/src/components/ui/badge";
import { Input } from "shared/src/components/ui/input";
import { SegmentedControl } from "shared/src/components/ui/segmented";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "shared/src/components/ui/select";
import { ControlPointChips } from "../ControlPointChips";
import { TemplateParameterFields } from "../TemplateParameterFields";
import { leftOperands, odrlActions, operators } from "shared/src/odrl_actions";
import { formatOperator } from "shared/src/components/ui/format-operator";
import { PolicyTemplateApi } from "shared/src/data/policyTemplateApi";
import type { PolicyTemplateDto, PolicyTemplateKind } from "shared/src/data/orval/model";
import {
  LISTS,
  PolicyTreeOps,
  type AtomicNode,
  type BuilderMode,
  type ConstraintNode,
  type Dragged,
  type ListKey,
  type LogicalNode,
  type LogicalOp,
  type PolicyTree,
  type RefNode,
  type RuleItem,
  type RuleNode,
  type Zone,
} from "./policyTree";

const LIST_META: Record<
  ListKey,
  { label: string; one: string; Icon: React.ComponentType<{ className?: string }>; tone: string }
> = {
  permission: {
    label: "Permissions",
    one: "Permission",
    Icon: CheckCircle2,
    tone: "text-success-600 dark:text-success-400",
  },
  prohibition: {
    label: "Prohibitions",
    one: "Prohibition",
    Icon: Ban,
    tone: "text-danger-600 dark:text-danger-400",
  },
  obligation: {
    label: "Obligations",
    one: "Obligation",
    Icon: ClipboardList,
    tone: "text-warn-600 dark:text-warn-400",
  },
};

const LOGICAL_LABEL: Record<LogicalOp, string> = {
  and: "All of",
  or: "Any of",
  xone: "Exactly one of",
};

const PALETTE_KINDS: { value: PolicyTemplateKind; label: string; hint: string }[] = [
  { value: "policy", label: "Policies", hint: "Whole policies, included as they are" },
  { value: "rule", label: "Rules", hint: "One permission, prohibition or obligation" },
  { value: "constraint", label: "Constraints", hint: "Conditions that go inside rules" },
];

type Library = Map<string, PolicyTemplateDto>;

/** Zones nest, so the smallest zone under the pointer wins; keyboard drags fall back to overlap. */
const innermost: CollisionDetection = (args) => {
  const hits = pointerWithin(args);
  if (hits.length === 0) return rectIntersection(args);
  const area = (id: string | number) => {
    const r = args.droppableRects.get(id);
    return r ? r.width * r.height : Infinity;
  };
  return [...hits].sort((a, b) => area(a.id) - area(b.id));
};

interface BuilderContext {
  mode: BuilderMode;
  tree: PolicyTree;
  library: Library;
  set: (tree: PolicyTree) => void;
}
const Ctx = React.createContext<BuilderContext | null>(null);
const useBuilder = () => React.useContext(Ctx)!;

export interface PolicyBuilderProps {
  mode: BuilderMode;
  tree: PolicyTree;
  onChange: (tree: PolicyTree) => void;
  /** Latest version of every template the user sees. */
  templates: PolicyTemplateDto[];
  /** Template being edited, left out of the palette so it cannot use itself. */
  self?: string;
}

/** Drag library pieces and blank blocks onto the canvas, nest them and fill in their values. */
export function PolicyBuilder({ mode, tree, onChange, templates, self }: PolicyBuilderProps) {
  const library: Library = useMemo(
    () => new Map(templates.map((t) => [PolicyTemplateApi.reference(t), t])),
    [templates],
  );
  const [dragging, setDragging] = useState<{ label: string } | null>(null);
  const sensors = useSensors(
    useSensor(PointerSensor, { activationConstraint: { distance: 4 } }),
    useSensor(KeyboardSensor),
  );

  const drop = (dragged: Dragged, zone: Zone) => {
    let base = tree;
    let node: RuleNode | ConstraintNode | undefined;
    if (dragged.source === "library") {
      const template = library.get(dragged.reference);
      if (template) node = PolicyTreeOps.ref(template);
    } else if (dragged.source === "block") {
      node = PolicyTreeOps.block(dragged.block);
    } else {
      const found = PolicyTreeOps.find(tree, dragged.id);
      if (
        !found ||
        (zone.kind === "constraints" && zone.owner && PolicyTreeOps.contains(found, zone.owner))
      )
        return;
      [base, node] = PolicyTreeOps.remove(tree, dragged.id);
    }
    if (!node) return;
    const refused = PolicyTreeOps.refusal(zone, node);
    if (refused) return toast.error(refused);
    onChange(PolicyTreeOps.insert(base, zone, node));
  };

  const onDragStart = (e: DragStartEvent) =>
    setDragging({ label: String(e.active.data.current?.label ?? "") });
  const onDragEnd = (e: DragEndEvent) => {
    setDragging(null);
    const dragged = e.active.data.current?.dragged as Dragged | undefined;
    const zone = e.over?.data.current?.zone as Zone | undefined;
    if (dragged && zone) drop(dragged, zone);
  };

  return (
    <Ctx.Provider value={{ mode, tree, library, set: onChange }}>
      <DndContext
        sensors={sensors}
        collisionDetection={innermost}
        onDragStart={onDragStart}
        onDragEnd={onDragEnd}
        onDragCancel={() => setDragging(null)}
      >
        <div className="grid min-h-0 grid-cols-1 gap-4 lg:grid-cols-[17rem_minmax(0,1fr)]">
          <PolicyBuilder.Palette templates={templates.filter((t) => t.id !== self)} onAdd={drop} />
          <PolicyBuilder.Canvas />
        </div>
        <DragOverlay dropAnimation={null}>
          {dragging && (
            <div className="rounded-lg border border-brand-sky/50 bg-background px-3 py-2 text-sm shadow-lg">
              {dragging.label}
            </div>
          )}
        </DragOverlay>
      </DndContext>
    </Ctx.Provider>
  );
}

/** The blank blocks and the library, each item draggable or added with its + button. */
PolicyBuilder.Palette = function Palette({
  templates,
  onAdd,
}: {
  templates: PolicyTemplateDto[];
  onAdd: (dragged: Dragged, zone: Zone) => void;
}) {
  const { mode, tree } = useBuilder();
  const [kind, setKind] = useState<PolicyTemplateKind>(
    mode === "constraint" ? "constraint" : "policy",
  );
  const [search, setSearch] = useState("");
  const shown = templates.filter(
    (t) =>
      PolicyTreeOps.kind(t) === kind &&
      (!search.trim() ||
        `${PolicyTemplateApi.title(t)} ${t.id}`
          .toLowerCase()
          .includes(search.trim().toLowerCase())),
  );
  const firstRule = LISTS.flatMap((l) => tree[l]).find((r): r is RuleNode => r.type === "rule");
  // Where the + button puts a condition: the root in constraint mode, else the first written rule.
  const conditionZone: Zone | undefined =
    mode === "constraint"
      ? { kind: "constraints" }
      : firstRule && { kind: "constraints", owner: firstRule.id };
  const quickZone = (dragged: Dragged): Zone | undefined => {
    if (dragged.source === "block")
      return dragged.block === "rule" ? { kind: "lane", list: "permission" } : conditionZone;
    if (dragged.source !== "library") return undefined;
    const t = templates.find((x) => PolicyTemplateApi.reference(x) === dragged.reference);
    const k = PolicyTreeOps.kind(t);
    if (k === "policy") return { kind: "include" };
    if (k === "rule") return { kind: "lane", list: PolicyTreeOps.ruleList(t) ?? "permission" };
    return conditionZone;
  };
  const add = (dragged: Dragged) => {
    const zone = quickZone(dragged);
    if (zone) onAdd(dragged, zone);
    else toast.info("Add a rule first, or drag the condition into one.");
  };

  const blocks: {
    block: "rule" | "atomic" | LogicalOp;
    label: string;
    hint: string;
    Icon: typeof Plus;
  }[] = [
    ...(mode === "constraint"
      ? []
      : [
          {
            block: "rule" as const,
            label: "Rule",
            hint: "Write an action and its conditions",
            Icon: CheckCircle2,
          },
        ]),
    { block: "atomic", label: "Condition", hint: "leftOperand, operator, value", Icon: Braces },
    { block: "and", label: "All of", hint: "Every condition inside holds", Icon: GitMerge },
    { block: "or", label: "Any of", hint: "At least one condition inside holds", Icon: GitMerge },
  ];

  return (
    <aside className="flex min-h-0 flex-col gap-4 overflow-y-auto">
      <section className="space-y-2">
        <span className="flex items-center gap-2 text-xs font-medium uppercase tracking-wide text-ink/50">
          <Puzzle className="h-3.5 w-3.5" /> Blocks
        </span>
        <div className="grid grid-cols-2 gap-1.5">
          {blocks.map((b) => (
            <PolicyBuilder.PaletteItem
              key={b.block}
              id={`block:${b.block}`}
              dragged={{ source: "block", block: b.block }}
              label={b.label}
              onAdd={add}
            >
              <span className="flex items-center gap-1.5 text-sm font-medium">
                <b.Icon className="h-3.5 w-3.5 text-ink/50" /> {b.label}
              </span>
              <span className="mt-0.5 block text-[11px] leading-snug text-ink/50">{b.hint}</span>
            </PolicyBuilder.PaletteItem>
          ))}
        </div>
      </section>

      <section className="flex min-h-0 flex-col gap-2">
        <span className="flex items-center gap-2 text-xs font-medium uppercase tracking-wide text-ink/50">
          <Library className="h-3.5 w-3.5" /> Library
        </span>
        <SegmentedControl
          value={kind}
          onChange={setKind}
          options={PALETTE_KINDS}
          ariaLabel="Kind of piece"
        />
        <Input
          placeholder="Search…"
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          onClear={() => setSearch("")}
          className="h-8 text-xs"
        />
        <div className="space-y-1.5">
          {shown.length === 0 && (
            <p className="py-3 text-center text-xs text-ink/50">Nothing of this kind.</p>
          )}
          {shown.map((t) => {
            const reference = PolicyTemplateApi.reference(t);
            return (
              <PolicyBuilder.PaletteItem
                key={reference}
                id={`lib:${reference}`}
                dragged={{ source: "library", reference }}
                label={PolicyTemplateApi.title(t)}
                onAdd={add}
              >
                <span className="block text-sm font-medium">{PolicyTemplateApi.title(t)}</span>
                {t.description && (
                  <span className="mt-0.5 block text-[11px] leading-snug text-ink/55">
                    {PolicyTemplateApi.text(t.description)}
                  </span>
                )}
                <ControlPointChips points={t.controlPoints} className="mt-1.5" />
              </PolicyBuilder.PaletteItem>
            );
          })}
        </div>
      </section>
    </aside>
  );
};

/** A draggable palette entry with a + button for those who would rather click. */
PolicyBuilder.PaletteItem = function PaletteItem({
  id,
  dragged,
  label,
  onAdd,
  children,
}: {
  id: string;
  dragged: Dragged;
  label: string;
  onAdd: (dragged: Dragged) => void;
  children: React.ReactNode;
}) {
  const { attributes, listeners, setNodeRef, isDragging } = useDraggable({
    id,
    data: { dragged, label },
  });
  return (
    <div
      ref={setNodeRef}
      {...attributes}
      {...listeners}
      className={cn(
        "group relative cursor-grab rounded-lg border border-ink/10 bg-background p-2.5 pr-8 transition-colors hover:border-brand-sky/50 active:cursor-grabbing",
        isDragging && "opacity-40",
      )}
    >
      {children}
      <button
        type="button"
        aria-label={`Add ${label}`}
        onPointerDown={(e) => e.stopPropagation()}
        onClick={() => onAdd(dragged)}
        className="absolute right-1.5 top-1.5 rounded p-1 text-ink/40 hover:bg-ink/[0.06] hover:text-ink"
      >
        <Plus className="h-3.5 w-3.5" />
      </button>
    </div>
  );
};

/** The canvas of the mode: included policies and the three rule lists, or a single condition. */
PolicyBuilder.Canvas = function Canvas() {
  const { mode, tree } = useBuilder();
  if (mode === "constraint") {
    return (
      <PolicyBuilder.Zone
        zone={{ kind: "constraints" }}
        empty="Drop the condition this template is: a constraint piece, a condition or a group."
      >
        {tree.constraints.map((c) => (
          <PolicyBuilder.ConstraintCard key={c.id} node={c} />
        ))}
      </PolicyBuilder.Zone>
    );
  }
  return (
    <div className="space-y-4">
      {mode !== "rule" && (
        <PolicyBuilder.Section title="Included policies" hint="Their rules are added as they are">
          <PolicyBuilder.Zone
            zone={{ kind: "include" }}
            empty="Drop whole policies from the library."
          >
            {tree.include.map((r) => (
              <PolicyBuilder.RefCard key={r.id} node={r} />
            ))}
          </PolicyBuilder.Zone>
        </PolicyBuilder.Section>
      )}
      {LISTS.map((list) => {
        const meta = LIST_META[list];
        return (
          <PolicyBuilder.Section
            key={list}
            title={
              <span className="flex items-center gap-1.5">
                <meta.Icon className={cn("h-4 w-4", meta.tone)} /> {meta.label}
              </span>
            }
            hint={
              list === "permission" ? "Each one is an alternative way to be allowed" : undefined
            }
          >
            <PolicyBuilder.Zone
              zone={{ kind: "lane", list }}
              empty={`Drop a Rule block or a ${meta.one.toLowerCase()} from the library.`}
            >
              {tree[list].map((r) => (
                <PolicyBuilder.RuleItemCard key={r.id} node={r} list={list} />
              ))}
            </PolicyBuilder.Zone>
            {mode === "offer" && (
              <PolicyBuilder.Zone
                zone={{ kind: "joins", list }}
                compact
                empty={`Constraint pieces dropped here join every ${meta.one.toLowerCase()}, included ones too.`}
              >
                {tree.joins[list].map((r) => (
                  <PolicyBuilder.RefCard key={r.id} node={r} />
                ))}
              </PolicyBuilder.Zone>
            )}
          </PolicyBuilder.Section>
        );
      })}
    </div>
  );
};

PolicyBuilder.Section = function Section({
  title,
  hint,
  children,
}: {
  title: React.ReactNode;
  hint?: string;
  children: React.ReactNode;
}) {
  return (
    <section className="space-y-1.5">
      <div className="flex items-baseline gap-2">
        <span className="text-sm font-medium">{title}</span>
        {hint && <span className="text-xs text-ink/45">{hint}</span>}
      </div>
      {children}
    </section>
  );
};

/** Where things are dropped; it lights up green when what is dragged fits, red when it does not. */
PolicyBuilder.Zone = function Zone({
  zone,
  empty,
  compact,
  children,
}: {
  zone: Zone;
  empty: string;
  compact?: boolean;
  children: React.ReactNode;
}) {
  const { tree, library } = useBuilder();
  const id = `zone:${zone.kind}:${"list" in zone ? zone.list : ""}:${"owner" in zone ? (zone.owner ?? "root") : ""}`;
  const { setNodeRef, isOver } = useDroppable({ id, data: { zone } });
  const { active } = useDndContext();
  const dragged = active?.data.current?.dragged as Dragged | undefined;
  const probe =
    dragged?.source === "library"
      ? library.get(dragged.reference) && PolicyTreeOps.ref(library.get(dragged.reference)!)
      : dragged?.source === "block"
        ? PolicyTreeOps.block(dragged.block)
        : dragged?.source === "node"
          ? PolicyTreeOps.find(tree, dragged.id)
          : undefined;
  const fits = probe && !PolicyTreeOps.refusal(zone, probe);
  const count = React.Children.count(children);

  return (
    <div
      ref={setNodeRef}
      className={cn(
        "space-y-2 rounded-lg border border-dashed p-2 transition-colors",
        compact ? "border-ink/10" : "border-ink/15",
        active && fits && "border-brand-sky/50 bg-brand-sky/[0.04]",
        isOver && fits && "border-success-500 bg-success-500/[0.06]",
        isOver && probe && !fits && "border-danger-500/60 bg-danger-500/[0.05]",
      )}
    >
      {children}
      {count === 0 && (
        <p className={cn("text-center text-xs text-ink/45", compact ? "py-1.5" : "py-4")}>
          {empty}
        </p>
      )}
    </div>
  );
};

/** A handle to drag a card to another zone. */
PolicyBuilder.useCardDrag = (node: RuleNode | ConstraintNode, label: string) => {
  const { attributes, listeners, setNodeRef, isDragging } = useDraggable({
    id: `node:${node.id}`,
    data: { dragged: { source: "node", id: node.id } satisfies Dragged, label },
  });
  const handle = (
    <button
      type="button"
      aria-label={`Move ${label}`}
      className="cursor-grab rounded p-0.5 text-ink/35 hover:text-ink/70 active:cursor-grabbing"
      {...attributes}
      {...listeners}
    >
      <GripVertical className="h-4 w-4" />
    </button>
  );
  return {
    ref: setNodeRef,
    style: undefined as React.CSSProperties | undefined,
    className: cn(isDragging && "opacity-40"),
    handle,
  };
};

PolicyBuilder.RemoveButton = function RemoveButton({ id, label }: { id: string; label: string }) {
  const { tree, set } = useBuilder();
  return (
    <Button
      size="icon_sm"
      variant="ghost"
      aria-label={`Remove ${label}`}
      onClick={() => set(PolicyTreeOps.remove(tree, id)[0])}
    >
      <Trash2 />
    </Button>
  );
};

PolicyBuilder.RuleItemCard = function RuleItemCard({
  node,
  list,
}: {
  node: RuleItem;
  list: ListKey;
}) {
  return node.type === "ref" ? (
    <PolicyBuilder.RefCard node={node} />
  ) : (
    <PolicyBuilder.RuleCard node={node} list={list} />
  );
};

/** A rule written here: its action and a zone for its conditions. */
PolicyBuilder.RuleCard = function RuleCard({ node, list }: { node: RuleNode; list: ListKey }) {
  const { tree, set } = useBuilder();
  const meta = LIST_META[list];
  const drag = PolicyBuilder.useCardDrag(node, `${meta.one}: ${node.action || "…"}`);
  return (
    <div
      ref={drag.ref}
      style={drag.style}
      className={cn("rounded-lg border border-ink/10 bg-background p-2.5", drag.className)}
    >
      <div className="flex items-center gap-2">
        {drag.handle}
        <meta.Icon className={cn("h-4 w-4 shrink-0", meta.tone)} />
        <span className="text-sm font-medium">{meta.one}</span>
        <Select
          value={node.action}
          onValueChange={(action) =>
            set(PolicyTreeOps.update<RuleNode>(tree, node.id, (r) => ({ ...r, action })))
          }
        >
          <SelectTrigger className="h-7 w-40 text-xs">
            <SelectValue placeholder="Action" />
          </SelectTrigger>
          <SelectContent>
            {odrlActions.map((a) => (
              <SelectItem key={a} value={a}>
                {a}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
        <span className="ml-auto">
          <PolicyBuilder.RemoveButton id={node.id} label="rule" />
        </span>
      </div>
      <div className="mt-2 pl-7">
        <PolicyBuilder.Zone
          zone={{ kind: "constraints", owner: node.id }}
          compact
          empty="Drop conditions here."
        >
          {node.constraints.map((c) => (
            <PolicyBuilder.ConstraintCard key={c.id} node={c} />
          ))}
        </PolicyBuilder.Zone>
      </div>
    </div>
  );
};

PolicyBuilder.ConstraintCard = function ConstraintCard({ node }: { node: ConstraintNode }) {
  if (node.type === "ref") return <PolicyBuilder.RefCard node={node} />;
  if (node.type === "logical") return <PolicyBuilder.LogicalCard node={node} />;
  return <PolicyBuilder.AtomicCard node={node} />;
};

/** A group of conditions combined with and, or or exactly one. */
PolicyBuilder.LogicalCard = function LogicalCard({ node }: { node: LogicalNode }) {
  const { tree, set } = useBuilder();
  const drag = PolicyBuilder.useCardDrag(node, LOGICAL_LABEL[node.op]);
  return (
    <div
      ref={drag.ref}
      style={drag.style}
      className={cn("rounded-lg border border-ink/10 bg-ink/[0.02] p-2", drag.className)}
    >
      <div className="flex items-center gap-2">
        {drag.handle}
        <GitMerge className="h-4 w-4 text-ink/50" />
        <Select
          value={node.op}
          onValueChange={(op) =>
            set(
              PolicyTreeOps.update<LogicalNode>(tree, node.id, (l) => ({
                ...l,
                op: op as LogicalOp,
              })),
            )
          }
        >
          <SelectTrigger className="h-7 w-36 text-xs">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            {(Object.keys(LOGICAL_LABEL) as LogicalOp[]).map((op) => (
              <SelectItem key={op} value={op}>
                {LOGICAL_LABEL[op]}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
        <span className="ml-auto">
          <PolicyBuilder.RemoveButton id={node.id} label="group" />
        </span>
      </div>
      <div className="mt-2 pl-6">
        <PolicyBuilder.Zone
          zone={{ kind: "constraints", owner: node.id }}
          compact
          empty="Drop conditions here."
        >
          {node.children.map((c) => (
            <PolicyBuilder.ConstraintCard key={c.id} node={c} />
          ))}
        </PolicyBuilder.Zone>
      </div>
    </div>
  );
};

/** A condition written by hand; a value like `$region` becomes a parameter of the template. */
PolicyBuilder.AtomicCard = function AtomicCard({ node }: { node: AtomicNode }) {
  const { tree, set } = useBuilder();
  const drag = PolicyBuilder.useCardDrag(node, node.leftOperand || "Condition");
  const change = (c: Partial<AtomicNode>) =>
    set(PolicyTreeOps.update<AtomicNode>(tree, node.id, (a) => ({ ...a, ...c })));
  return (
    <div
      ref={drag.ref}
      style={drag.style}
      className={cn(
        "flex items-center gap-1.5 rounded-lg border border-ink/10 bg-background p-1.5",
        drag.className,
      )}
    >
      {drag.handle}
      <Input
        list="policy-builder-operands"
        className="h-7 w-40 text-xs"
        placeholder="leftOperand"
        value={node.leftOperand}
        onChange={(e) => change({ leftOperand: e.target.value })}
      />
      <datalist id="policy-builder-operands">
        {leftOperands.map((o) => (
          <option key={o} value={o} />
        ))}
      </datalist>
      <Select value={node.operator} onValueChange={(operator) => change({ operator })}>
        <SelectTrigger className="h-7 w-32 text-xs">
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          {operators.map((o) => (
            <SelectItem key={o} value={o}>
              {formatOperator(o)}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
      <Input
        className="h-7 min-w-0 flex-1 text-xs"
        placeholder="value or $parameter"
        value={node.rightOperand}
        onChange={(e) => change({ rightOperand: e.target.value })}
      />
      <PolicyBuilder.RemoveButton id={node.id} label="condition" />
    </div>
  );
};

/** A library piece: its values, and in a template, which of them are asked when it is used. */
PolicyBuilder.RefCard = function RefCard({ node }: { node: RefNode }) {
  const { mode, tree, set } = useBuilder();
  const t = node.template;
  const title = t ? PolicyTemplateApi.title(t) : node.reference;
  const drag = PolicyBuilder.useCardDrag(node, title);
  const change = (c: Partial<RefNode>) =>
    set(PolicyTreeOps.update<RefNode>(tree, node.id, (r) => ({ ...r, ...c })));
  const params = Object.entries(t?.parameters ?? {}).filter(([, d]) => !d.ui?.hidden);
  const authoring = mode !== "offer";

  return (
    <div
      ref={drag.ref}
      style={drag.style}
      className={cn(
        "rounded-lg border border-brand-sky/25 bg-brand-sky/[0.03] p-2.5",
        drag.className,
      )}
    >
      <div className="flex items-center gap-2">
        {drag.handle}
        <Puzzle className="h-4 w-4 shrink-0 text-brand-sky" />
        <span className="min-w-0 truncate text-sm font-medium">{title}</span>
        <Badge variant="outline" size="xs">
          {PolicyTreeOps.kind(t)}
        </Badge>
        <span className="hidden truncate font-mono text-[11px] text-ink/40 sm:inline">
          {node.reference}
        </span>
        <span className="ml-auto">
          <PolicyBuilder.RemoveButton id={node.id} label={title} />
        </span>
      </div>
      {!t && <p className="mt-2 text-xs text-danger-700">Not in the library you see.</p>}
      {params.length > 0 && (
        <div className="mt-2 grid grid-cols-1 gap-2.5 pl-7 sm:grid-cols-2">
          {params.map(([name, def]) => {
            const exposedAs = node.exposed[name];
            const asked = exposedAs !== undefined;
            return (
              <div key={name} className="flex flex-col gap-1">
                <span className="flex items-center gap-1.5 text-xs font-medium">
                  {PolicyTemplateApi.text(def.ui?.label) ?? name.slice(1)}
                  <span className="font-mono text-[10px] text-ink/40">{def.dataType}</span>
                  {authoring && (
                    <button
                      type="button"
                      title={asked ? "Fixed here instead" : "Ask for it when this template is used"}
                      onClick={() => {
                        const exposed = { ...node.exposed };
                        if (asked) delete exposed[name];
                        else exposed[name] = name.slice(1);
                        change({ exposed });
                      }}
                      className={cn(
                        "ml-auto flex items-center gap-1 rounded px-1.5 py-px text-[10px]",
                        asked
                          ? "bg-brand-sky/15 text-brand-sky"
                          : "text-ink/45 hover:bg-ink/[0.05]",
                      )}
                    >
                      <Link2 className="h-3 w-3" /> {asked ? "asked" : "ask"}
                    </button>
                  )}
                </span>
                {asked ? (
                  <Input
                    className="h-8 font-mono text-xs"
                    value={exposedAs}
                    onChange={(e) =>
                      change({
                        exposed: { ...node.exposed, [name]: e.target.value.replace(/^\$/, "") },
                      })
                    }
                    title="Name of the parameter of this template"
                  />
                ) : (
                  <TemplateParameterFields.Field
                    def={def}
                    value={node.values[name]}
                    onChange={(v) => change({ values: { ...node.values, [name]: v } })}
                  />
                )}
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
};
