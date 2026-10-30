import React from "react";
import { ArrowRight } from "lucide-react";
import dayjs from "dayjs";
import { Badge } from "@eunomia/ui";
import { RelativeTime } from "@eunomia/ui";
import { DataplaneTransferLogDto } from "shared/src/data/orval/model";
import { dataplaneStateVariant } from "./utils/dataplaneState";

/** Every state the dataplane process went through, oldest first. */
export function DataplaneLogsTab({ logs }: { logs: DataplaneTransferLogDto[] }) {
  if (logs.length === 0) {
    return (
      <div className="rounded-lg border border-dashed border-ink/15 py-10 text-center text-sm text-ink/50">
        The dataplane has not logged any state change.
      </div>
    );
  }

  return (
    <div className="overflow-hidden rounded-lg border border-ink/10">
      <table className="w-full text-sm">
        <thead className="bg-ink/[0.03] text-left text-xs font-medium text-ink/55">
          <tr>
            <th className="w-10 px-3 py-2.5 font-medium">#</th>
            <th className="px-3 py-2.5 font-medium">State change</th>
            <th className="px-3 py-2.5 font-medium">Trigger</th>
            <th className="px-3 py-2.5 font-medium">Reason</th>
            <th className="px-3 py-2.5 text-right font-medium">When</th>
          </tr>
        </thead>
        <tbody>
          {logs.map((log, i) => (
            <tr key={log.id ?? i} className="border-t border-ink/[0.07]">
              <td className="px-3 py-2.5 tabular-nums text-ink/40">{i + 1}</td>
              <td className="px-3 py-2.5">
                <span className="inline-flex items-center gap-1.5">
                  {log.previous_state ? (
                    <span className="text-xs text-ink/70">{log.previous_state}</span>
                  ) : (
                    <span className="text-xs italic text-ink/45">Initial</span>
                  )}
                  <ArrowRight className="h-3 w-3 text-ink/35" />
                  <Badge variant="status" state={dataplaneStateVariant(log.new_state)} size="sm">
                    {log.new_state}
                  </Badge>
                </span>
              </td>
              <td className="px-3 py-2.5 text-ink/70">{log.trigger}</td>
              <td className="px-3 py-2.5 text-ink/60">
                {log.reason ?? <span className="text-ink/30">—</span>}
              </td>
              <td
                className="px-3 py-2.5 text-right text-ink/60"
                title={dayjs(log.created_at).format("YYYY-MM-DD HH:mm:ss.SSS")}
              >
                <RelativeTime date={log.created_at} />
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
