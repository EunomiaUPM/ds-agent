import React from "react";
import { TabsContent } from "shared/src/components/ui/tabs";
import { FormatDate } from "shared/src/components/ui/format-date";
import { Badge } from "shared/components/ui/badge";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "shared/components/ui/table";
import { DataplaneTransferLogDto } from "shared/src/data/orval/model";
import { dataplaneStateVariant } from "./utils/dataplaneState";

export function DataplaneLogsTab({ logs }: { logs: DataplaneTransferLogDto[] }) {
  return (
    <TabsContent value="dataplane-logs" className="w-full">
      {logs.length > 0 ? (
        <div className="rounded-md border overflow-hidden mt-4">
          <Table>
            <TableHeader className="bg-muted/50 sticky top-0">
              <TableRow>
                <TableHead className="w-[160px]">Timestamp</TableHead>
                <TableHead className="w-[160px]">Trigger</TableHead>
                <TableHead className="w-[300px]">State Transition</TableHead>
                <TableHead>Reason</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {logs.map((log, i) => (
                <TableRow key={log.id ?? i}>
                  <TableCell className="text-xs text-muted-foreground whitespace-nowrap">
                    <FormatDate date={log.created_at} />
                  </TableCell>
                  <TableCell>
                    <span className="font-mono text-xs bg-muted/70 border border-white/10 px-1.5 py-0.5 rounded whitespace-nowrap">
                      {log.trigger ?? "—"}
                    </span>
                  </TableCell>
                  <TableCell>
                    <div className="flex items-center gap-2">
                      {log.previous_state ? (
                        <Badge
                          variant="status"
                          state={dataplaneStateVariant(log.previous_state)}
                          size="sm"
                        >
                          {log.previous_state}
                        </Badge>
                      ) : (
                        <span className="text-muted-foreground text-xs italic">initial</span>
                      )}
                      <span className="text-muted-foreground text-xs">→</span>
                      <Badge
                        variant="status"
                        state={dataplaneStateVariant(log.new_state)}
                        size="sm"
                      >
                        {log.new_state ?? "—"}
                      </Badge>
                    </div>
                  </TableCell>
                  <TableCell className="text-xs text-muted-foreground">
                    {log.reason ? (
                      <span>{log.reason}</span>
                    ) : (
                      <span className="italic opacity-50">—</span>
                    )}
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </div>
      ) : (
        <div className="text-muted-foreground p-8 text-center border rounded-md border-dashed mt-4">
          No dataplane logs available.
        </div>
      )}
    </TabsContent>
  );
}
