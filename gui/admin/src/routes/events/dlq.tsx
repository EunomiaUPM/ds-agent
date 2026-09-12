/*
 * Copyright (C) 2026 - Universidad Politécnica de Madrid - UPM
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program. If not, see <https://www.gnu.org/licenses/>.
 */

import { createFileRoute } from "@tanstack/react-router";
import { useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import {
  useListDeadLetters,
  useReplayDeadLetter,
  useReplayAllDeadLetters,
  useDeleteDeadLetter,
  getListDeadLettersQueryKey,
} from "shared/src/data/orval/events/events";
import {
  DeadLetterRecord,
  ListDeadLettersStatus,
  ReplayAllResponse,
} from "shared/src/data/orval/model";
import { PageSection } from "shared/src/components/layout/PageSection";
import { DataTable } from "shared/src/components/DataTable";
import { FormatDate } from "shared/src/components/ui/format-date";
import { Skeleton } from "shared/src/components/ui/skeleton";
import { Button } from "shared/src/components/ui/button";
import { Badge } from "shared/src/components/ui/badge";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from "shared/src/components/ui/dialog";
import { AlertTriangle, RefreshCw, Trash2, Eye, RotateCcw } from "lucide-react";
import { toast } from "sonner";

const DlqComponent = () => {
  const queryClient = useQueryClient();
  const [selectedStatus, setSelectedStatus] = useState<ListDeadLettersStatus | "">("");
  const [selectedDlq, setSelectedDlq] = useState<DeadLetterRecord | null>(null);

  const { data, isLoading, isError, refetch, isFetching } = useListDeadLetters({
    status: selectedStatus ? (selectedStatus as ListDeadLettersStatus) : undefined,
  });

  const deadLetters: DeadLetterRecord[] = Array.isArray(data?.data)
    ? (data.data as DeadLetterRecord[])
    : [];

  const { mutate: replaySingle, isPending: isReplayingSingle } = useReplayDeadLetter({
    mutation: {
      onSuccess: () => {
        queryClient.invalidateQueries({ queryKey: getListDeadLettersQueryKey() });
        toast.success("Event re-enqueued for delivery");
      },
      onError: (err) => {
        toast.error(`Replay failed: ${String(err)}`);
      },
    },
  });

  const { mutate: replayAll, isPending: isReplayingAll } = useReplayAllDeadLetters({
    mutation: {
      onSuccess: (res) => {
        queryClient.invalidateQueries({ queryKey: getListDeadLettersQueryKey() });
        const resData = res.data as ReplayAllResponse;
        toast.success(`Replayed ${resData.replayed_count} dead letters`);
      },
      onError: (err) => {
        toast.error(`Replay all failed: ${String(err)}`);
      },
    },
  });

  const { mutate: deleteRecord } = useDeleteDeadLetter({
    mutation: {
      onSuccess: () => {
        queryClient.invalidateQueries({ queryKey: getListDeadLettersQueryKey() });
        toast.success("Dead letter record discarded");
      },
      onError: (err) => {
        toast.error(`Discard failed: ${String(err)}`);
      },
    },
  });

  return (
    <div className="flex flex-col gap-6 w-full">
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <h2 className="text-lg font-semibold tracking-tight">Dead Letter Queue (DLQ)</h2>
          <p className="text-xs text-muted-foreground">
            Monitor and replay failed webhook deliveries that exceeded max exponential backoff retry
            attempts.
          </p>
        </div>

        <div className="flex items-center gap-2">
          <Button
            variant="outline"
            size="sm"
            onClick={() => refetch()}
            disabled={isFetching}
            className="gap-1.5"
          >
            <RefreshCw className={`h-3.5 w-3.5 ${isFetching ? "animate-spin" : ""}`} />
            Refresh
          </Button>
          <Button
            size="sm"
            onClick={() => replayAll()}
            disabled={isReplayingAll || deadLetters.length === 0}
            className="gap-1.5"
          >
            <RotateCcw className="h-3.5 w-3.5" />
            {isReplayingAll ? "Replaying All..." : "Replay All"}
          </Button>
        </div>
      </div>

      {/* Filter Chips */}
      <div className="flex items-center gap-2">
        <Button
          variant={selectedStatus === "" ? "default" : "outline"}
          size="sm"
          className="text-xs h-7"
          onClick={() => setSelectedStatus("")}
        >
          All
        </Button>
        <Button
          variant={selectedStatus === "Unresolved" ? "default" : "outline"}
          size="sm"
          className="text-xs h-7"
          onClick={() => setSelectedStatus("Unresolved")}
        >
          Unresolved
        </Button>
        <Button
          variant={selectedStatus === "Replayed" ? "default" : "outline"}
          size="sm"
          className="text-xs h-7"
          onClick={() => setSelectedStatus("Replayed")}
        >
          Replayed
        </Button>
        <Button
          variant={selectedStatus === "Purged" ? "default" : "outline"}
          size="sm"
          className="text-xs h-7"
          onClick={() => setSelectedStatus("Purged")}
        >
          Purged
        </Button>
      </div>

      <PageSection>
        {isLoading ? (
          <div className="flex flex-col gap-3 p-4">
            <Skeleton className="h-12 w-full" />
            <Skeleton className="h-12 w-full" />
          </div>
        ) : isError ? (
          <div className="p-8 text-center text-sm text-destructive">
            Failed to load Dead Letter Queue records.
          </div>
        ) : (
          <DataTable
            className="text-sm"
            data={deadLetters}
            keyExtractor={(dlq) => dlq.id}
            searchPlaceholder="Filter dead letters by topic, status, or callback..."
            emptyMessage="No dead letter messages found. All deliveries healthy."
            defaultSortKey="failed_at"
            defaultSortDirection="desc"
            columns={[
              {
                header: "Topic",
                accessorKey: "topic",
                cell: (dlq) => <Badge variant="code">{dlq.topic}</Badge>,
              },
              {
                header: "Status",
                accessorKey: "status",
                cell: (dlq) => (
                  <Badge variant={dlq.status === "Unresolved" ? "destructive" : "default"}>
                    {dlq.status}
                  </Badge>
                ),
              },
              {
                header: "Error",
                accessorKey: "error_message",
                cell: (dlq) => (
                  <span className="text-danger-700 dark:text-danger-300 text-xs block max-w-[280px] truncate">
                    {dlq.error_message}
                  </span>
                ),
              },
              {
                header: "Callback",
                accessorKey: "callback_address",
                cell: (dlq) => (
                  <span className="font-mono text-xs text-muted-foreground block max-w-[240px] truncate">
                    {dlq.callback_address}
                  </span>
                ),
              },
              {
                header: "Attempts",
                accessorKey: "attempts",
                cell: (dlq) => <Badge variant="infoLighter">{dlq.attempts}</Badge>,
              },
              {
                header: "Failed at",
                accessorKey: "failed_at",
                sortValue: (dlq) => new Date(dlq.failed_at).getTime(),
                cell: (dlq) => <FormatDate date={dlq.failed_at} />,
              },
              {
                header: "Actions",
                sortable: false,
                searchable: false,
                cell: (dlq) => (
                  <div className="flex items-center gap-2">
                    <Button
                      variant="outline"
                      size="xs"
                      className="gap-1"
                      disabled={isReplayingSingle}
                      onClick={() => replaySingle({ dlqId: dlq.id })}
                    >
                      <RotateCcw className="h-3 w-3" /> Replay
                    </Button>
                    <Button
                      variant="ghost"
                      size="icon"
                      className="h-7 w-7"
                      onClick={() => setSelectedDlq(dlq)}
                    >
                      <Eye className="h-3.5 w-3.5" />
                    </Button>
                    <Button
                      variant="ghost"
                      size="icon"
                      className="h-7 w-7 text-destructive hover:text-destructive hover:bg-destructive/10"
                      onClick={() => {
                        if (confirm("Discard this dead letter record?")) {
                          deleteRecord({ dlqId: dlq.id });
                        }
                      }}
                    >
                      <Trash2 className="h-3.5 w-3.5" />
                    </Button>
                  </div>
                ),
              },
            ]}
          />
        )}
      </PageSection>

      {/* Detail Dialog */}
      <Dialog open={!!selectedDlq} onOpenChange={() => setSelectedDlq(null)}>
        <DialogContent className="max-w-2xl max-h-[85vh] flex flex-col">
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2 text-sm font-mono text-destructive">
              <AlertTriangle className="h-4 w-4" />
              Dead Letter: {selectedDlq?.id}
            </DialogTitle>
            <DialogDescription className="text-xs text-muted-foreground">
              Dead letter queue inspection for failed event delivery.
            </DialogDescription>
          </DialogHeader>
          <div className="flex-1 overflow-y-auto rounded bg-muted/40 p-4 font-mono text-xs flex flex-col gap-3">
            <div>
              <span className="text-muted-foreground font-semibold">Error Message:</span>
              <p className="text-destructive mt-1 font-sans">{selectedDlq?.error_message}</p>
            </div>
            <div>
              <span className="text-muted-foreground font-semibold">Target Callback:</span>
              <p className="text-foreground mt-1 select-all">{selectedDlq?.callback_address}</p>
            </div>
            <div>
              <span className="text-muted-foreground font-semibold">Event Payload:</span>
              <pre className="whitespace-pre-wrap break-all mt-1 bg-background/60 p-2.5 rounded border border-ink/5">
                {JSON.stringify(selectedDlq?.payload, null, 2)}
              </pre>
            </div>
          </div>
          <DialogFooter>
            <Button onClick={() => setSelectedDlq(null)}>Close</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
};

export const Route = createFileRoute("/events/dlq")({
  component: DlqComponent,
});
