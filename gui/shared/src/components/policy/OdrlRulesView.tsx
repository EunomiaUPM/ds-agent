import React from "react";
import { Ban, CheckCircle2, ClipboardList } from "lucide-react";
import { cn } from "@eunomia/ui";
import { formatOperator } from "@eunomia/ui";

type Rule = {
  action?: unknown;
  constraint?: Constraint[] | Constraint;
  duty?: Rule[] | Rule;
};
type Constraint = { leftOperand?: unknown; operator?: unknown; rightOperand?: unknown };

const KINDS = [
  {
    key: "permission",
    label: "Permissions",
    Icon: CheckCircle2,
    tone: "text-success-600 dark:text-success-400",
  },
  {
    key: "prohibition",
    label: "Prohibitions",
    Icon: Ban,
    tone: "text-danger-600 dark:text-danger-400",
  },
  {
    key: "obligation",
    label: "Obligations",
    Icon: ClipboardList,
    tone: "text-warn-600 dark:text-warn-400",
  },
] as const;

/** The rules of an ODRL offer or agreement, one table row per rule with its constraints. */
export function OdrlRulesView({ policy, className }: { policy: unknown; className?: string }) {
  const p = (policy ?? {}) as Record<string, unknown>;
  const groups = KINDS.map((k) => ({ ...k, rules: OdrlRulesView.list<Rule>(p[k.key]) })).filter(
    (g) => g.rules.length > 0,
  );

  if (groups.length === 0) {
    return <p className={cn("text-sm italic text-ink/50", className)}>The policy has no rules.</p>;
  }

  return (
    <div className={cn("overflow-hidden rounded-lg border border-ink/10", className)}>
      <table className="w-full text-sm">
        <thead className="bg-ink/[0.03] text-left text-xs font-medium text-ink/55">
          <tr>
            <th className="w-40 px-3 py-2 font-medium">Rule</th>
            <th className="w-40 px-3 py-2 font-medium">Action</th>
            <th className="px-3 py-2 font-medium">Constraints</th>
          </tr>
        </thead>
        <tbody>
          {groups.flatMap(({ key, label, Icon, tone, rules }) =>
            rules.map((rule, i) => (
              <tr key={`${key}-${i}`} className="border-t border-ink/[0.07] align-top">
                <td className="px-3 py-2.5">
                  <span className={cn("inline-flex items-center gap-1.5 font-medium", tone)}>
                    <Icon className="h-4 w-4" />
                    {label.slice(0, -1)}
                  </span>
                </td>
                <td className="px-3 py-2.5">
                  <span className="rounded-md bg-ink/[0.07] px-2 py-0.5 font-mono text-xs">
                    {OdrlRulesView.term(rule.action)}
                  </span>
                </td>
                <td className="px-3 py-2.5">
                  <OdrlRulesView.Constraints rule={rule} />
                </td>
              </tr>
            )),
          )}
        </tbody>
      </table>
    </div>
  );
}

OdrlRulesView.list = <T,>(v: unknown): T[] => (v == null ? [] : Array.isArray(v) ? v : [v]) as T[];

/** ODRL terms come as plain strings or as `{ "@id": ... }` nodes. */
OdrlRulesView.term = (v: unknown): string => {
  if (v == null) return "—";
  if (typeof v === "object") {
    const o = v as Record<string, unknown>;
    return String(o["@id"] ?? o["@value"] ?? o.id ?? JSON.stringify(v));
  }
  return String(v);
};

OdrlRulesView.Constraints = function Constraints({ rule }: { rule: Rule }) {
  const constraints = OdrlRulesView.list<Constraint>(rule.constraint);
  const duties = OdrlRulesView.list<Rule>(rule.duty);
  if (constraints.length === 0 && duties.length === 0)
    return <span className="text-ink/40">Unconstrained</span>;

  return (
    <div className="flex flex-col gap-1.5">
      {constraints.map((c, i) => (
        <span key={i} className="inline-flex flex-wrap items-center gap-1.5 text-[0.8125rem]">
          <span className="font-mono text-xs text-ink/80">{OdrlRulesView.term(c.leftOperand)}</span>
          <span className="text-ink/50">{formatOperator(OdrlRulesView.term(c.operator))}</span>
          <span className="font-medium">{OdrlRulesView.term(c.rightOperand)}</span>
        </span>
      ))}
      {duties.map((d, i) => (
        <span key={`d${i}`} className="text-[0.8125rem] text-ink/70">
          Duty: <span className="font-mono text-xs">{OdrlRulesView.term(d.action)}</span>
        </span>
      ))}
    </div>
  );
};
