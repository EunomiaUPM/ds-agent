import React, { FC } from "react";
import { PolicyVariants } from "shared/src/components/ui/policy";
import { OdrlPermission, OdrlProhibition, OdrlObligation } from "shared/src/data/orval/model";
import { formatOperator } from "shared/src/components/ui/format-operator";
import { formatValue } from "shared/src/components/ui/format-value";

export interface PolicyComponentProps {
  policyItem: (OdrlPermission | OdrlProhibition | OdrlObligation)[] | null | undefined;
  variant: PolicyVariants;
}

const PolicyComponent: FC<PolicyComponentProps> = ({ policyItem, variant }) => {
  const isEmpty = policyItem?.length === 0 || policyItem == null;

  if (isEmpty) return null;

  const HeadingColor = {
    permission: "text-[#3fc28a]",
    obligation: "text-[#ffc107]",
    prohibition: "text-[#ff7878]",
  }[variant || "permission"];

  const BadgeBg = {
    permission: "bg-success-800",
    obligation: "bg-warn-900",
    prohibition: "bg-danger-900",
  }[variant || "permission"];

  const formatKey = (text: any) => {
    const clean = String(text)
      .replace(/[()[\]{},\"]/g, " ")
      .trim();
    const spaced = clean.replace(/([A-Z])/g, " $1").trim();
    return spaced.charAt(0).toUpperCase() + spaced.slice(1);
  };

  return (
    <div className="flex flex-col border-b border-white/20 last:border-0 px-3 py-1 ">
      {policyItem?.map((item, i) => (
        <div key={i} className="flex flex-col mb-6 last:mb-0">
          <div className="flex items-center gap-3 mb-4">
            <span className={`font-bold uppercase tracking-wide ${HeadingColor}`}>{variant}</span>
            <span
              className={`text-xs font-bold px-2 py-0.5 rounded text-white uppercase tracking-wider ${BadgeBg}`}
            >
              {item.action}
            </span>
          </div>

          <div className="flex flex-col">
            <span className="text-[11px] font-bold text-white/50 mb-2 uppercase tracking-wide">
              Constraints:
            </span>

            {item.constraint == null || item.constraint.length === 0 ? (
              <span className="text-sm italic text-white/50">No constraints</span>
            ) : (
              <div className="flex flex-col w-full">
                {item.constraint.map((constr: any, j: number) => (
                  <div
                    key={j}
                    className="flex items-center w-full py-1.5 border-b gap-3 border-white/10 last:border-0 text-xs"
                  >
                    <div className="w-[25%] font-semibold text-white">
                      {formatKey(constr.leftOperand)}
                    </div>
                    <div className="w-[35%] text-white/70">{formatOperator(constr.operator)}</div>
                    <div className="w-[30%] text-white/90">{formatValue(constr.rightOperand)}</div>
                  </div>
                ))}
              </div>
            )}
          </div>
        </div>
      ))}
    </div>
  );
};

export default PolicyComponent;
