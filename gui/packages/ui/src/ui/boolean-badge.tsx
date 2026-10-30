import React from "react";
import { Badge } from "./badge";

export interface BooleanBadgeProps {
  value: boolean | string | null | undefined;
  trueLabel?: string;
  falseLabel?: string;
}

export function BooleanBadge({
  value,
  trueLabel = "Yes",
  falseLabel = "No",
}: BooleanBadgeProps) {
  const isTrue = value === true || value === "true" || value === "Yes";
  return (
    <Badge variant={isTrue ? "success" : "danger"} dot>
      {isTrue ? trueLabel : falseLabel}
    </Badge>
  );
}

export default BooleanBadge;
