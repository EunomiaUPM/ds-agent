import React from "react";
import { Ban, CheckCircle2, ClipboardList, GitMerge, Layers, Puzzle } from "lucide-react";
import { cn } from "@eunomia/ui";
import { formatOperator } from "@eunomia/ui";
import type { PolicyTemplateKind } from "shared/src/data/orval/model";

type Json = Record<string, unknown>;

const LISTS = [
  {
    key: "permission",
    label: "Permission",
    Icon: CheckCircle2,
    tone: "text-success-600 dark:text-success-400",
  },
  {
    key: "prohibition",
    label: "Prohibition",
    Icon: Ban,
    tone: "text-danger-600 dark:text-danger-400",
  },
  {
    key: "obligation",
    label: "Obligation",
    Icon: ClipboardList,
    tone: "text-warn-600 dark:text-warn-400",
  },
] as const;

const LOGICAL: Record<string, string> = {
  and: "All of",
  andSequence: "All of, in order",
  or: "Any of",
  xone: "Exactly one of",
};

export interface TemplateContentViewProps {
  kind: PolicyTemplateKind;
  content: unknown;
  /** How a reference `id@version` is shown; a link on pages that have one. */
  renderRef?: (reference: string) => React.ReactNode;
  className?: string;
}

/** A template's content read as structure: included policies, rules and their conditions, with
 * references to other templates and `$parameters` marked out. */
export function TemplateContentView({
  kind,
  content,
  renderRef,
  className,
}: TemplateContentViewProps) {
  const ref = renderRef ?? ((r: string) => <span className="font-mono">{r}</span>);
  const c = (content ?? {}) as Json;
  return (
    <div className={cn("space-y-3 text-sm", className)}>
      {kind === "constraint" && <TemplateContentView.Constraint value={c} renderRef={ref} />}
      {kind === "rule" &&
        LISTS.filter((l) => l.key in c).map((l) => (
          <TemplateContentView.Rule key={l.key} list={l} rule={c[l.key] as Json} renderRef={ref} />
        ))}
      {kind === "policy" && (
        <>
          {Array.isArray(c["@include"]) && (c["@include"] as Json[]).length > 0 && (
            <div className="rounded-lg border border-ink/10 p-3">
              <p className="mb-2 flex items-center gap-1.5 text-xs font-medium uppercase tracking-wide text-ink/50">
                <Layers className="h-3.5 w-3.5" /> Includes the rules of
              </p>
              <div className="flex flex-wrap gap-2">
                {(c["@include"] as Json[]).map((r, i) => (
                  <TemplateContentView.Ref key={i} value={r} renderRef={ref} />
                ))}
              </div>
            </div>
          )}
          {LISTS.flatMap((l) =>
            ((c[l.key] ?? []) as Json[]).map((rule, i) => (
              <TemplateContentView.Rule
                key={`${l.key}-${i}`}
                list={l}
                rule={rule}
                renderRef={ref}
              />
            )),
          )}
        </>
      )}
    </div>
  );
}

type RenderRef = (reference: string) => React.ReactNode;

/** A `$parameter` stands out; anything else is shown as its value. */
TemplateContentView.Value = function Value({ value }: { value: unknown }) {
  if (typeof value === "string" && /^\$[A-Za-z]/.test(value))
    return (
      <span className="rounded bg-brand-sky/10 px-1 font-mono text-[12px] text-brand-sky">
        {value}
      </span>
    );
  const text = typeof value === "string" ? value : JSON.stringify(value);
  return <span className="font-mono text-[12px] text-ink/85">{text}</span>;
};

/** A reference to another template, with the values it binds. */
TemplateContentView.Ref = function Ref({
  value,
  renderRef,
}: {
  value: Json;
  renderRef: RenderRef;
}) {
  const bindings = Object.entries((value.with ?? {}) as Json);
  return (
    <span className="inline-flex flex-wrap items-center gap-1.5 rounded-md border border-brand-sky/25 bg-brand-sky/[0.05] px-2 py-1">
      <Puzzle className="h-3.5 w-3.5 text-brand-sky" />
      <span className="text-[13px]">{renderRef(String(value["@ref"]))}</span>
      {bindings.map(([k, v]) => (
        <span key={k} className="inline-flex items-center gap-1 text-xs text-ink/60">
          <span className="font-mono">{k.replace(/^\$/, "")}</span>=
          <TemplateContentView.Value value={v} />
        </span>
      ))}
    </span>
  );
};

TemplateContentView.Constraint = function Constraint({
  value,
  renderRef,
}: {
  value: Json;
  renderRef: RenderRef;
}) {
  if ("@ref" in value) return <TemplateContentView.Ref value={value} renderRef={renderRef} />;
  const op = Object.keys(LOGICAL).find((k) => Array.isArray(value[k]));
  if (op)
    return (
      <div className="space-y-1.5">
        <span className="flex items-center gap-1.5 text-xs font-medium text-ink/60">
          <GitMerge className="h-3.5 w-3.5" /> {LOGICAL[op]}
        </span>
        <div className="space-y-1.5 border-l-2 border-ink/10 pl-3">
          {(value[op] as Json[]).map((c, i) => (
            <TemplateContentView.Constraint key={i} value={c} renderRef={renderRef} />
          ))}
        </div>
      </div>
    );
  const left = value.leftOperand ?? value["ovc:leftOperand"];
  return (
    <div className="flex flex-wrap items-center gap-x-2 gap-y-1">
      {"ovc:credentialSubjectType" in value && (
        <span className="text-xs text-ink/55">
          claim of <TemplateContentView.Value value={value["ovc:credentialSubjectType"]} />
        </span>
      )}
      <TemplateContentView.Value value={left} />
      <span className="text-xs text-ink/55">
        {typeof value.operator === "string" && !value.operator.startsWith("$") ? (
          formatOperator(value.operator)
        ) : (
          <TemplateContentView.Value value={value.operator} />
        )}
      </span>
      <TemplateContentView.Value value={value.rightOperand} />
    </div>
  );
};

TemplateContentView.Rule = function Rule({
  list,
  rule,
  renderRef,
}: {
  list: (typeof LISTS)[number];
  rule: Json;
  renderRef: RenderRef;
}) {
  const constraints = (
    Array.isArray(rule.constraint) ? rule.constraint : rule.constraint ? [rule.constraint] : []
  ) as Json[];
  return (
    <div className="rounded-lg border border-ink/10 p-3">
      <div className="flex items-center gap-2">
        <list.Icon className={cn("h-4 w-4", list.tone)} />
        <span className="font-medium">{list.label}</span>
        {"@ref" in rule ? (
          <TemplateContentView.Ref value={rule} renderRef={renderRef} />
        ) : (
          <span className="rounded bg-ink/[0.06] px-1.5 py-px font-mono text-xs">
            {String(rule.action ?? "")}
          </span>
        )}
      </div>
      {constraints.length > 0 && (
        <div className="mt-2.5 space-y-1.5 border-l-2 border-ink/10 pl-3">
          <span className="text-[11px] uppercase tracking-wide text-ink/45">when</span>
          {constraints.map((c, i) => (
            <TemplateContentView.Constraint key={i} value={c} renderRef={renderRef} />
          ))}
        </div>
      )}
    </div>
  );
};
