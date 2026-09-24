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
import { keepPreviousData, useQueryClient } from "@tanstack/react-query";
import { useState, useMemo } from "react";
import {
  useListEventsFeed,
  usePublishEvent,
  getListEventsFeedQueryKey,
} from "shared/src/data/orval/events/events";
import { EventEnvelope, ListEventsFeedParams } from "shared/src/data/orval/model";
import { useEventStream } from "shared/src/hooks/useEventStream";
import { useTableQueryParams } from "shared/src/hooks/useTableQueryParams";
import { PageSection } from "shared/src/components/layout/PageSection";
import { DataTable } from "shared/src/components/DataTable";
import { FormatDate } from "shared/src/components/ui/format-date";
import { Button } from "shared/src/components/ui/button";
import { Badge } from "shared/src/components/ui/badge";
import { Input } from "shared/src/components/ui/input";
import { Textarea } from "shared/src/components/ui/textarea";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from "shared/src/components/ui/dialog";
import { Radio, Send, RefreshCw, Eye, Trash2, Filter } from "lucide-react";
import { toast } from "sonner";

interface PublishDialogProps {
  open: boolean;
  onClose: () => void;
}

const publishPresets = [
  {
    label: "transfers:bla",
    topic: "transfers:bla",
    source: "gui-admin",
    payload: JSON.stringify(
      {
        transfer_id: "urn:uuid:11111111-1111-1111-1111-111111111111",
        state: "COMPLETED",
        message: "Transfer completed successfully",
      },
      null,
      2,
    ),
  },
  {
    label: "transfers:created",
    topic: "transfers:created",
    source: "transfer-agent",
    payload: JSON.stringify(
      {
        transfer_id: "urn:uuid:22222222-2222-2222-2222-222222222222",
        dataset_id: "urn:uuid:33333333-3333-3333-3333-333333333333",
        state: "REQUESTED",
      },
      null,
      2,
    ),
  },
  {
    label: "catalog:dataset:published",
    topic: "catalog:dataset:published",
    source: "catalog-service",
    payload: JSON.stringify(
      {
        dataset_id: "urn:uuid:44444444-4444-4444-4444-444444444444",
        title: "European Energy Mobility Dataset",
        version: "1.0.0",
      },
      null,
      2,
    ),
  },
];

const PublishDialog = ({ open, onClose }: PublishDialogProps) => {
  const queryClient = useQueryClient();
  const [topic, setTopic] = useState("transfers:bla");
  const [sourceCrate, setSourceCrate] = useState("gui-admin");
  const [payloadStr, setPayloadStr] = useState(
    publishPresets[0].payload,
  );

  const { mutate: publish, isPending } = usePublishEvent({
    mutation: {
      onSuccess: () => {
        queryClient.invalidateQueries({ queryKey: getListEventsFeedQueryKey() });
        toast.success("Event published to bus");
        onClose();
      },
      onError: (err) => {
        toast.error(`Publish failed: ${String(err)}`);
      },
    },
  });

  const handleSend = () => {
    if (!topic.trim()) {
      toast.error("Topic is required");
      return;
    }
    let payload: Record<string, unknown>;
    try {
      payload = JSON.parse(payloadStr);
    } catch {
      toast.error("Payload must be valid JSON");
      return;
    }

    publish({
      data: {
        topic: topic.trim(),
        source_crate: sourceCrate.trim() || undefined,
        payload,
      },
    });
  };

  return (
    <Dialog open={open} onOpenChange={onClose}>
      <DialogContent className="max-w-lg">
        <DialogHeader>
          <DialogTitle>Publish Event to Bus</DialogTitle>
          <DialogDescription className="text-xs text-muted-foreground">
            Emit an event envelope into the event bus for subscriber testing and live streaming.
          </DialogDescription>
        </DialogHeader>
        <div className="flex flex-col gap-4 py-2">
          <div className="flex flex-col gap-1.5">
            <label className="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
              Topic
            </label>
            <Input
              value={topic}
              onChange={(e) => setTopic(e.target.value)}
              placeholder="e.g. transfers:bla or transfers:created"
            />
            <div className="flex flex-wrap items-center gap-1.5 mt-1">
              <span className="text-[11px] text-muted-foreground">Presets:</span>
              {publishPresets.map((p) => (
                <Button
                  key={p.topic}
                  type="button"
                  variant={topic === p.topic ? "default" : "outline"}
                  size="xs"
                  className="text-[11px] h-5 px-2 font-mono"
                  onClick={() => {
                    setTopic(p.topic);
                    setSourceCrate(p.source);
                    setPayloadStr(p.payload);
                  }}
                >
                  {p.label}
                </Button>
              ))}
            </div>
          </div>
          <div className="flex flex-col gap-1.5">
            <label className="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
              Source
            </label>
            <Input
              value={sourceCrate}
              onChange={(e) => setSourceCrate(e.target.value)}
              placeholder="gui-admin"
            />
          </div>
          <div className="flex flex-col gap-1.5">
            <label className="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
              JSON Payload
            </label>
            <Textarea
              value={payloadStr}
              onChange={(e) => setPayloadStr(e.target.value)}
              rows={6}
              className="font-mono text-xs"
            />
          </div>
        </div>
        <DialogFooter>
          <Button variant="outline" onClick={onClose} disabled={isPending}>
            Cancel
          </Button>
          <Button onClick={handleSend} disabled={isPending} className="gap-2">
            <Send className="h-3.5 w-3.5" />
            {isPending ? "Publishing..." : "Publish"}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};

const FeedComponent = () => {
  const [publishOpen, setPublishOpen] = useState(false);
  const [selectedEvent, setSelectedEvent] = useState<EventEnvelope | null>(null);

  const { params: queryParams, onQueryChange } = useTableQueryParams({
    defaultLimit: 25,
    defaultSort: "created_at_desc",
    defaultFilters: { topic: "all" },
  });

  const activeTopic =
    queryParams.filters.topic && queryParams.filters.topic !== "all"
      ? queryParams.filters.topic
      : queryParams.search && !queryParams.search.includes(" ")
        ? queryParams.search
        : undefined;

  // Live SSE stream
  const {
    events: liveEvents,
    isConnected,
    clearEvents,
  } = useEventStream({
    topic: activeTopic,
    maxBuffer: 200,
  });

  // Historical page, filtered, sorted and paged by the events service
  const {
    data: histData,
    refetch,
    isFetching,
  } = useListEventsFeed(
    {
      topic: activeTopic,
      limit: queryParams.limit,
      page: queryParams.cursor ? undefined : queryParams.page,
      cursor: queryParams.cursor,
      sort: queryParams.sort as ListEventsFeedParams["sort"],
    },
    {
      query: {
        placeholderData: keepPreviousData,
      },
    },
  );

  const historicalEvents: EventEnvelope[] = Array.isArray(histData?.data)
    ? (histData.data as EventEnvelope[])
    : [];

  // On page 1 combine live stream and historical events; on subsequent pages show historical pages
  const displayEvents = useMemo(() => {
    let list: EventEnvelope[];
    if (queryParams.page > 1) {
      list = historicalEvents;
    } else {
      const map = new Map<string, EventEnvelope>();
      for (const ev of liveEvents) {
        map.set(ev.id, ev);
      }
      for (const ev of historicalEvents) {
        if (!map.has(ev.id)) {
          map.set(ev.id, ev);
        }
      }
      list = Array.from(map.values());
    }

    const ascending = queryParams.sort.endsWith("_asc");
    return [...list].sort((a, b) => {
      const diff = new Date(a.timestamp).getTime() - new Date(b.timestamp).getTime();
      return ascending ? diff : -diff;
    });
  }, [liveEvents, historicalEvents, queryParams.page, queryParams.sort]);

  return (
    <div className="flex flex-col gap-6 w-full">
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <div className="flex items-center gap-3">
            <h2 className="text-lg font-semibold tracking-tight">Event Stream & Audit Feed</h2>
            <div className="flex items-center gap-1.5 rounded-full border border-ink/10 bg-background/50 px-2.5 py-0.5 text-xs">
              <span
                className={`h-2 w-2 rounded-full ${isConnected ? "bg-emerald-500 animate-pulse" : "bg-zinc-500"}`}
              />
              <span className="text-muted-foreground uppercase text-xs tracking-wider font-mono">
                SSE {isConnected ? "Live" : "Offline"}
              </span>
            </div>
          </div>
          <p className="text-xs text-muted-foreground mt-1">
            Real-time event stream ingested by the BFF gateway with HMAC signature validation and
            exponential retry.
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
            variant="outline"
            size="sm"
            onClick={clearEvents}
            className="gap-1.5 text-muted-foreground"
          >
            <Trash2 className="h-3.5 w-3.5" />
            Clear Live
          </Button>
          <Button size="sm" onClick={() => setPublishOpen(true)} className="gap-1.5">
            <Radio className="h-3.5 w-3.5" />
            Publish Event
          </Button>
        </div>
      </div>

      <div className="flex flex-wrap items-center gap-2">
        <div className="relative flex-1 min-w-[240px] max-w-sm">
          <Filter className="absolute left-2.5 top-2.5 h-4 w-4 text-muted-foreground" />
          <Input
            value={queryParams.search ?? ""}
            onChange={(e) => onQueryChange({ search: e.target.value, page: 1 })}
            placeholder="Filter by topic pattern (e.g. transfers:*, transfers:bla)..."
            className="pl-9 text-xs"
          />
        </div>
        <div className="flex items-center gap-1.5">
          <Button
            variant={
              !queryParams.search &&
              (!queryParams.filters.topic || queryParams.filters.topic === "all")
                ? "default"
                : "outline"
            }
            size="sm"
            className="text-xs h-8"
            onClick={() =>
              onQueryChange({
                search: "",
                filters: { ...queryParams.filters, topic: "all" },
                page: 1,
              })
            }
          >
            All
          </Button>
          {["transfers:*", "transfers:bla", "transfers:**", "catalog:*"].map((pat) => (
            <Button
              key={pat}
              variant={
                queryParams.search === pat || queryParams.filters.topic === pat
                  ? "default"
                  : "outline"
              }
              size="sm"
              className="text-xs h-8 font-mono"
              onClick={() => {
                const isSelected =
                  queryParams.search === pat || queryParams.filters.topic === pat;
                onQueryChange({
                  search: isSelected ? "" : pat,
                  filters: { ...queryParams.filters, topic: isSelected ? "all" : pat },
                  page: 1,
                });
              }}
            >
              {pat}
            </Button>
          ))}
          {(queryParams.search ||
            (queryParams.filters.topic && queryParams.filters.topic !== "all")) && (
            <Button
              variant="ghost"
              size="sm"
              onClick={() =>
                onQueryChange({
                  search: "",
                  filters: { ...queryParams.filters, topic: "all" },
                  page: 1,
                })
              }
              className="text-xs text-muted-foreground"
            >
              Clear
            </Button>
          )}
        </div>
      </div>

      <PageSection>
        <DataTable
          className="text-sm"
          data={displayEvents}
          serverSide={true}
          loading={isFetching}
          queryParams={queryParams}
          onQueryChange={onQueryChange}
          keyExtractor={(ev) => ev.id}
          searchPlaceholder="Filter events by topic, source, or payload..."
          emptyMessage='No events recorded yet. Click "Publish Event" to test.'
          defaultSortKey="created_at"
          defaultSortDirection="desc"
          pageSize={25}
          pageSizeOptions={[10, 25, 50, 100]}
          columns={[
            {
              header: "Topic",
              sortable: false,
              cell: (ev) => <Badge variant="code">{ev.topic}</Badge>,
            },
            {
              header: "Source",
              sortable: false,
              cell: (ev) => <Badge variant="infoLighter">{ev.source_crate}</Badge>,
            },
            {
              header: "Tenant",
              sortable: false,
              cell: (ev) => <span className="font-mono text-xs">{ev.tenant_id}</span>,
            },
            {
              header: "Payload",
              searchValue: (ev) => JSON.stringify(ev.payload),
              sortable: false,
              cell: (ev) => (
                <span className="text-xs text-foreground/80 block max-w-[420px] truncate">
                  {JSON.stringify(ev.payload)}
                </span>
              ),
            },
            {
              header: "Timestamp",
              sortKey: "created_at",
              cell: (ev) => <FormatDate date={ev.timestamp} format="DD/MM/YYYY - HH:mm:ss" />,
            },
            {
              header: "Actions",
              sortable: false,
              searchable: false,
              cell: (ev) => (
                <Button
                  variant="ghost"
                  size="icon"
                  className="h-7 w-7"
                  onClick={() => setSelectedEvent(ev)}
                >
                  <Eye className="h-3.5 w-3.5" />
                </Button>
              ),
            },
          ]}
        />
      </PageSection>

      {/* Detail Dialog */}
      <Dialog open={!!selectedEvent} onOpenChange={() => setSelectedEvent(null)}>
        <DialogContent className="max-w-2xl max-h-[85vh] flex flex-col">
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2 text-sm font-mono">
              <span className="text-primary">{selectedEvent?.topic}</span>
            </DialogTitle>
            <DialogDescription className="text-xs text-muted-foreground">
              Event envelope metadata, payload, and schema details.
            </DialogDescription>
          </DialogHeader>
          <div className="flex-1 overflow-y-auto rounded bg-muted/40 p-4 font-mono text-xs">
            <pre className="whitespace-pre-wrap break-all">
              {JSON.stringify(selectedEvent, null, 2)}
            </pre>
          </div>
          <DialogFooter>
            <Button onClick={() => setSelectedEvent(null)}>Close</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <PublishDialog open={publishOpen} onClose={() => setPublishOpen(false)} />
    </div>
  );
};

export const Route = createFileRoute("/events/feed")({
  component: FeedComponent,
});
