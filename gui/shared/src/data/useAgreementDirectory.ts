import { useMemo } from "react";
import { useQuery } from "@tanstack/react-query";
import { getBatchAgreements } from "shared/src/data/orval/negotiations/negotiations";
import { AgreementDto } from "shared/src/data/orval/model";

export interface AgreementDirectory {
  isLoading: boolean;
  byId: Map<string, AgreementDto>;
}

/** The given agreements, fetched in one batch call; unknown ids are left out. */
export const useAgreementDirectory = (ids: (string | null | undefined)[]): AgreementDirectory => {
  const key = [...new Set(ids.filter((id): id is string => !!id))].sort();
  const { data, isLoading } = useQuery({
    queryKey: ["agreements", "batch", key],
    queryFn: () => getBatchAgreements({ ids: key }),
    enabled: key.length > 0,
    staleTime: 5 * 60_000,
  });

  return useMemo(() => {
    const items = data?.status === 200 ? data.data : [];
    return { isLoading, byId: new Map(items.map((a) => [a.id, a])) };
  }, [data, isLoading]);
};
