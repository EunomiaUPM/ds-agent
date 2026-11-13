import { useCallback, useMemo } from "react";
import { useGetTransferProcessById } from "shared/src/data/orval/transfers/transfers";
import {
  useGetDataplaneTransferByProcessId,
  useGetDataplaneTransferInfo,
} from "shared/src/data/orval/dataplane-transfers/dataplane-transfers";
import { useGetDataplaneTransferLogs } from "shared/src/data/orval/dataplane-logs/dataplane-logs";
import { useGetTransferEventsByDataplaneProcessId } from "shared/src/data/orval/transfer-events/transfer-events";
import {
  DataplaneInfoResponse,
  DataplaneTransferDto,
  DataplaneTransferLogDto,
  TransferEventDto,
  TransferProcessDto,
} from "shared/src/data/orval/model";

export interface TransferProcessDetail {
  isLoading: boolean;
  isDataplaneLoading: boolean;
  isLogsLoading: boolean;
  isEventsLoading: boolean;
  isFetching: boolean;
  tp: TransferProcessDto | null;
  dp: DataplaneTransferDto | null;
  info: DataplaneInfoResponse | null;
  logs: DataplaneTransferLogDto[];
  events: TransferEventDto[];
  refresh: () => void;
}

export function useTransferProcessDetail(transferProcessId: string): TransferProcessDetail {
  const tpQuery = useGetTransferProcessById(transferProcessId);

  const dpQuery = useGetDataplaneTransferByProcessId(transferProcessId, {
    query: { retry: false },
  });

  const dp = useMemo<DataplaneTransferDto | null>(() => {
    const raw = dpQuery.data?.data;
    return raw && "id" in raw ? (raw as DataplaneTransferDto) : null;
  }, [dpQuery.data]);

  const dataplaneProcessId = dp?.id;

  const logsQuery = useGetDataplaneTransferLogs(dataplaneProcessId ?? "", {
    query: { enabled: !!dataplaneProcessId },
  });

  const eventsQuery = useGetTransferEventsByDataplaneProcessId(dataplaneProcessId ?? "", {
    query: { enabled: !!dataplaneProcessId },
  });

  const infoQuery = useGetDataplaneTransferInfo(dataplaneProcessId ?? "", {
    query: { enabled: !!dataplaneProcessId },
  });

  const tp = tpQuery.data?.status === 200 ? tpQuery.data.data : null;

  const info = useMemo<DataplaneInfoResponse | null>(() => {
    const raw = infoQuery.data?.data;
    return raw && !("code" in raw) ? (raw as DataplaneInfoResponse) : null;
  }, [infoQuery.data]);

  const logs = useMemo<DataplaneTransferLogDto[]>(() => {
    const raw = logsQuery.data?.data;
    if (!raw || !Array.isArray(raw)) return [];
    return [...raw].sort(
      (a, b) => new Date(a.created_at).getTime() - new Date(b.created_at).getTime(),
    );
  }, [logsQuery.data]);

  const events = useMemo<TransferEventDto[]>(() => {
    const raw = eventsQuery.data?.data;
    if (!raw || !Array.isArray(raw)) return [];
    return [...raw].sort(
      (a, b) => new Date(b.created_at).getTime() - new Date(a.created_at).getTime(),
    );
  }, [eventsQuery.data]);

  const refresh = useCallback(() => {
    void tpQuery.refetch();
    void dpQuery.refetch();
    if (dataplaneProcessId) {
      void logsQuery.refetch();
      void eventsQuery.refetch();
      void infoQuery.refetch();
    }
  }, [tpQuery, dpQuery, logsQuery, eventsQuery, infoQuery, dataplaneProcessId]);

  const isFetching =
    tpQuery.isFetching ||
    dpQuery.isFetching ||
    logsQuery.isFetching ||
    eventsQuery.isFetching ||
    infoQuery.isFetching;

  return {
    isLoading: tpQuery.isLoading,
    isDataplaneLoading: dpQuery.isLoading,
    isLogsLoading: logsQuery.isLoading,
    isEventsLoading: eventsQuery.isLoading,
    isFetching,
    tp,
    dp,
    info,
    logs,
    events,
    refresh,
  };
}
