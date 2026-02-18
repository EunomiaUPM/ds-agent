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
  tp: TransferProcessDto | null;
  dp: DataplaneTransferDto | null;
  info: DataplaneInfoResponse | null;
  logs: DataplaneTransferLogDto[];
  events: TransferEventDto[];
}

export function useTransferProcessDetail(transferProcessId: string): TransferProcessDetail {
  const { data: transferProcess, isLoading } =
    useGetTransferProcessById(transferProcessId);

  const { data: dataplaneTransfer } = useGetDataplaneTransferByProcessId(
    transferProcessId,
    { query: { retry: false } },
  );

  const dataplaneProcessId =
    dataplaneTransfer?.data && "id" in dataplaneTransfer.data
      ? (dataplaneTransfer.data as DataplaneTransferDto).id
      : undefined;

  const { data: dataplaneLogs } = useGetDataplaneTransferLogs(
    dataplaneProcessId!,
    { query: { enabled: !!dataplaneProcessId } },
  );

  const { data: transferEvents } = useGetTransferEventsByDataplaneProcessId(
    dataplaneProcessId!,
    { query: { enabled: !!dataplaneProcessId } },
  );

  const { data: dataplaneInfo } = useGetDataplaneTransferInfo(
    dataplaneProcessId!,
    { query: { enabled: !!dataplaneProcessId } },
  );

  const tp = transferProcess?.status === 200 ? transferProcess.data : null;

  const dp =
    dataplaneTransfer?.data && "id" in dataplaneTransfer.data
      ? (dataplaneTransfer.data as DataplaneTransferDto)
      : null;

  const info =
    dataplaneInfo?.data && !("code" in dataplaneInfo.data)
      ? (dataplaneInfo.data as DataplaneInfoResponse)
      : null;

  const logs: DataplaneTransferLogDto[] =
    dataplaneLogs?.data && Array.isArray(dataplaneLogs.data)
      ? dataplaneLogs.data
      : [];

  const events: TransferEventDto[] =
    transferEvents?.data && Array.isArray(transferEvents.data)
      ? transferEvents.data
      : [];

  return { isLoading, tp, dp, info, logs, events };
}
