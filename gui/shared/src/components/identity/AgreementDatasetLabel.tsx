import React from "react";
import { useGetAgreementByIdS } from "shared/src/data/orval/negotiations/negotiations";
import { DatasetLabel } from "./DatasetLabel";
import { Identifier } from "@eunomia/ui";

/** The dataset an agreement covers; transfers only carry the agreement id. */
export function AgreementDatasetLabel({
  agreementId,
  compact = false,
}: {
  agreementId?: string | null;
  compact?: boolean;
}) {
  const { data } = useGetAgreementByIdS(agreementId ?? "", {
    query: { enabled: !!agreementId, retry: false, staleTime: 5 * 60_000 },
  });
  const target = data?.status === 200 ? data.data.target : undefined;
  if (target) return <DatasetLabel id={target} compact={compact} />;
  return <Identifier value={agreementId} />;
}
