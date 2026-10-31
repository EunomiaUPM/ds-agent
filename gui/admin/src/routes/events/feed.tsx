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

import { createFileRoute, Link } from "@tanstack/react-router";
import { keepPreviousData, useQueryClient } from "@tanstack/react-query";
import React, { useMemo, useState } from "react";
import { ArrowUpRight, CheckCheck } from "lucide-react";
import {
  useListEventsFeed,
  useMarkAllEventsRead,
  useMarkEventRead,
} from "shared/src/data/orval/events/events";
import { EventEnvelope, ListEventsFeedParams } from "shared/src/data/orval/model";
import { useTableQueryParams } from "@eunomia/ui";
import { DataTable } from "@eunomia/ui";
import { RelativeTime } from "@eunomia/ui";
import { Button } from "@eunomia/ui";
import { Badge } from "@eunomia/ui";
import { FieldGrid } from "@eunomia/ui";
import { JsonTable } from "@eunomia/ui";
import { Identifier } from "@eunomia/ui";
import {
  Sheet,
  SheetContent,
  SheetDescription,
  SheetHeader,
  SheetTitle,
} from "@eunomia/ui";
import { EventTopics } from "shared/src/lib/eventTopics";
import { cn } from "@eunomia/ui";

export const Route = createFileRoute("/events/feed")({
  component: EventFeedPage,
});

const PERIODS: Record<string, number> = { hour: 3_600_000, day: 86_400_000, week: 604_800_000 };

function EventFeedPage() {
  const queryClient = useQueryClient();
  const [selected, setSelected] = useState<EventEnvelope | null>(null);
  const { params, apiParams, onQueryChange } = useTableQueryParams({
    defaultLimit: 20,
    defaultSort: "created_at_desc",
  });

  // Table filters become the feed's query: an area is a topic prefix, a period a lower bound.
  const queryParams = useMemo<ListEventsFeedParams>(() => {
    const { area, state, period, search: _search, ...rest } = apiParams;
    return {
      ...rest,
      ...(area && { topic: `${area}:**` }),
      ...(state && { read: state === "read" }),
      ...(period && {
        created_after: new Date(Date.now() - PERIODS[period]).toISOString(),
      }),
    };
  }, [apiParams]);

  const { data, isFetching } = useListEventsFeed(queryParams, {
    query: { placeholderData: keepPreviousData },
  });
  const page = data?.status === 200 ? data.data : undefined;

  const refresh = () =>
    queryClient.invalidateQueries({
      predicate: (q) => typeof q.queryKey[0] === "string" && q.queryKey[0].startsWith("/events"),
    });
  const { mutate: markRead } = useMarkEventRead({ mutation: { onSuccess: refresh } });
  const { mutate: markAll, isPending: markingAll } = useMarkAllEventsRead({
    mutation: { onSuccess: refresh },
  });

  const open = (event: EventEnvelope) => {
    setSelected(event);
    if (!event.read_at) markRead({ eventId: event.id });
  };

  return (
    <>
      <DataTable<EventEnvelope>
        data={page}
        serverSide
        loading={isFetching}
        queryParams={params}
        onQueryChange={onQueryChange}
        keyExtractor={(e) => e.id}
        onRowClick={open}
        searchable={false}
        emptyMessage="No events match these filters."
        toolbarActions={
          <Button variant="outline" size="sm" onClick={() => markAll()} isLoading={markingAll}>
            {!markingAll && <CheckCheck />}
            Mark all read
          </Button>
        }
        filters={[
          {
            id: "area",
            label: "Area",
            options: [
              ...EventTopics.areas.map((a) => ({ label: a.label, value: a.prefix })),
            ],
          },
          {
            id: "state",
            label: "State",
            options: [
              { label: "Unread", value: "unread" },
              { label: "Read", value: "read" },
            ],
          },
          {
            id: "period",
            label: "Period",
            options: [
              { label: "Last hour", value: "hour" },
              { label: "Last 24 hours", value: "day" },
              { label: "Last 7 days", value: "week" },
            ],
          },
        ]}
        columns={[
          {
            header: "",
            className: "w-6 pr-0",
            cell: (e) => (
              <span
                className={cn(
                  "block h-2 w-2 rounded-full",
                  e.read_at ? "bg-transparent" : "bg-brand-sky",
                )}
                title={e.read_at ? "Read" : "Unread"}
              />
            ),
          },
          {
            header: "Event",
            cell: (e) => (
              <span className="flex flex-col">
                <span className={cn("text-sm", !e.read_at && "font-semibold")}>
                  {EventTopics.describe(e.topic)}
                </span>
                <span className="font-mono text-xs text-ink/50">{e.topic}</span>
              </span>
            ),
          },
          {
            header: "Area",
            cell: (e) => (
              <Badge variant="outline" size="sm">
                {EventTopics.area(e.topic)}
              </Badge>
            ),
          },
          {
            header: "Record",
            cell: (e) => <RecordLink event={e} />,
          },
          {
            header: "When",
            accessorKey: "timestamp",
            sortKey: "created_at",
            cell: (e) => <RelativeTime date={e.timestamp} className="text-sm text-ink/70" />,
          },
        ]}
      />

      <Sheet open={!!selected} onOpenChange={(o) => !o && setSelected(null)}>
        <SheetContent className="w-full overflow-y-auto sm:max-w-xl">
          {selected && (
            <>
              <SheetHeader className="mb-4">
                <SheetTitle>{EventTopics.describe(selected.topic)}</SheetTitle>
                <SheetDescription className="font-mono text-xs">{selected.topic}</SheetDescription>
              </SheetHeader>
              <FieldGrid
                columns={2}
                className="mb-5"
                fields={[
                  { label: "Published", value: <RelativeTime date={selected.timestamp} /> },
                  { label: "Source", value: selected.source_crate },
                  { label: "Record", value: <RecordLink event={selected} /> },
                  { label: "Visibility", value: selected.visibility },
                  { label: "Event", value: <Identifier value={selected.id} />, span: 2 },
                  {
                    label: "Correlation",
                    value: <Identifier value={selected.correlation_id} />,
                    span: 2,
                    hidden: !selected.correlation_id,
                  },
                ]}
              />
              <h3 className="mb-2 text-xs font-semibold uppercase tracking-wide text-ink/60">
                Payload
              </h3>
              <JsonTable value={selected.payload} />
            </>
          )}
        </SheetContent>
      </Sheet>
    </>
  );
}

/** Link to the page of the record an event is about, if there is one. */
function RecordLink({ event }: { event: EventEnvelope }) {
  const link = EventTopics.link(event);
  if (!link) return <span className="text-ink/35">—</span>;
  const id = Object.values(link.params)[0];
  return (
    <Link
      to={link.to}
      params={link.params}
      onClick={(e: React.MouseEvent) => e.stopPropagation()}
      className="inline-flex items-center gap-1 text-brand-sky hover:underline"
    >
      <Identifier value={id} copyable={false} />
      <ArrowUpRight className="h-3.5 w-3.5" />
    </Link>
  );
}
