import React from "react";
import { ShieldCheck } from "lucide-react";
import { OdrlPolicyDto } from "shared/src/data/orval/model";
import { Identifier } from "shared/src/components/identity/Identifier";
import { RelativeTime } from "shared/src/components/ui/format-date";
import { OdrlRulesView } from "./OdrlRulesView";
import { ControlPointChips } from "./ControlPointChips";

/** A catalog offer: what it says, its rules as a table, and whatever actions the page adds. */
export function PolicyCard({
  policy,
  actions,
}: {
  policy: OdrlPolicyDto;
  actions?: React.ReactNode;
}) {
  return (
    <article className="flex flex-col rounded-xl border border-ink/10 bg-ink/[0.015]">
      <header className="flex items-start gap-3 border-b border-ink/[0.07] px-4 py-3">
        <ShieldCheck className="mt-0.5 h-4 w-4 shrink-0 text-brand-sky" />
        <div className="min-w-0 flex-1">
          <p className="text-sm font-medium">{policy.description || "Untitled offer"}</p>
          <p className="mt-0.5 flex flex-wrap items-center gap-x-2 text-xs text-ink/50">
            <Identifier value={policy.id} />
            <span>·</span>
            <span>
              created <RelativeTime date={policy.createdAt} />
            </span>
            {policy.composition ? (
              <>
                <span>·</span>
                <span>
                  composed from {policy.composition.length}{" "}
                  {policy.composition.length === 1 ? "part" : "parts"}
                </span>
              </>
            ) : (
              policy.sourceTemplateId && (
                <>
                  <span>·</span>
                  <span>from template</span>
                </>
              )
            )}
          </p>
          <ControlPointChips points={policy.controlPoints} className="mt-1.5" />
        </div>
      </header>
      <div className="flex-1 p-4">
        <OdrlRulesView policy={policy.odrlOffer} />
      </div>
      {actions && (
        <footer className="flex justify-end gap-2 border-t border-ink/[0.07] px-4 py-2.5">
          {actions}
        </footer>
      )}
    </article>
  );
}
