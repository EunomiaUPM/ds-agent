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
import { useState } from "react";
import {
  useListEventSubscriptions,
  useCreateEventSubscription,
  useDeleteEventSubscription,
  getListEventSubscriptionsQueryKey,
} from "shared/src/data/orval/events/events";
import { EventSubscription, ListEventSubscriptionsParams } from "shared/src/data/orval/model";
import { useTableQueryParams } from "@eunomia/ui";
import { PageSection } from "@eunomia/ui";
import { DataTable } from "@eunomia/ui";
import { FormatDate } from "@eunomia/ui";
import { Skeleton } from "@eunomia/ui";
import { Button } from "@eunomia/ui";
import { Badge } from "@eunomia/ui";
import { Input } from "@eunomia/ui";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from "@eunomia/ui";
import { Plus, Trash2, ShieldCheck } from "lucide-react";
import { toast } from "sonner";
import { EventTopics } from "shared/src/lib/eventTopics";

interface CreateSubscriptionDialogProps {
  open: boolean;
  onClose: () => void;
}

const CreateSubscriptionDialog = ({ open, onClose }: CreateSubscriptionDialogProps) => {
  const queryClient = useQueryClient();
  const [topicPattern, setTopicPattern] = useState("transfers:*");
  const [callbackAddress, setCallbackAddress] = useState("https://");
  const [secret, setSecret] = useState("");
  const [retryLimit, setRetryLimit] = useState("5");

  const topicPresets = [
    ...EventTopics.areas.map((a) => ({ label: `${a.prefix}:*`, pattern: `${a.prefix}:*` })),
    { label: "* (all)", pattern: "*" },
  ];

  const { mutate: createSub, isPending } = useCreateEventSubscription({
    mutation: {
      onSuccess: () => {
        queryClient.invalidateQueries({ queryKey: getListEventSubscriptionsQueryKey() });
        toast.success("Webhook subscription created");
        onClose();
        setSecret("");
      },
      onError: (err) => {
        toast.error(`Subscription creation failed: ${String(err)}`);
      },
    },
  });

  const handleSubmit = () => {
    if (!topicPattern.trim()) {
      toast.error("Topic pattern is required");
      return;
    }
    if (!callbackAddress.trim() || callbackAddress === "https://") {
      toast.error("Valid callback URL is required");
      return;
    }

    const retries = parseInt(retryLimit, 10);

    createSub({
      data: {
        topic_pattern: topicPattern.trim(),
        callback_address: callbackAddress.trim(),
        secret: secret.trim() || undefined,
        retry_limit: isNaN(retries) ? undefined : retries,
      },
    });
  };

  return (
    <Dialog open={open} onOpenChange={onClose}>
      <DialogContent className="max-w-md">
        <DialogHeader>
          <DialogTitle>Add Webhook Subscription</DialogTitle>
          <DialogDescription className="text-xs text-muted-foreground">
            Configure an external webhook listener with automated retry policy and HMAC signing.
          </DialogDescription>
        </DialogHeader>
        <div className="flex flex-col gap-4 py-2">
          <div className="flex flex-col gap-1.5">
            <label className="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
              Topic Pattern
            </label>
            <Input
              value={topicPattern}
              onChange={(e) => setTopicPattern(e.target.value)}
              placeholder="e.g. transfers:* or catalog:dataset:create"
            />
            <div className="flex flex-wrap items-center gap-1.5 mt-1">
              <span className="text-[11px] text-muted-foreground">Presets:</span>
              {topicPresets.map((p) => (
                <Button
                  key={p.pattern}
                  type="button"
                  variant={topicPattern === p.pattern ? "default" : "outline"}
                  size="xs"
                  className="text-[11px] h-5 px-2 font-mono"
                  onClick={() => setTopicPattern(p.pattern)}
                >
                  {p.label}
                </Button>
              ))}
            </div>
            <p className="text-[11px] text-muted-foreground mt-0.5">
              Supports <code className="font-mono">:</code> and <code className="font-mono">.</code> delimiters with <code className="font-mono">*</code> (single segment) and <code className="font-mono">**</code> (multi-segment) wildcards.
            </p>
          </div>
          <div className="flex flex-col gap-1.5">
            <label className="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
              Callback URL
            </label>
            <Input
              value={callbackAddress}
              onChange={(e) => setCallbackAddress(e.target.value)}
              placeholder="https://your-service.com/webhook"
            />
          </div>
          <div className="flex flex-col gap-1.5">
            <label className="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
              HMAC-SHA256 Secret (Optional)
            </label>
            <Input
              type="password"
              value={secret}
              onChange={(e) => setSecret(e.target.value)}
              placeholder="Leave empty for unauthenticated webhook"
            />
          </div>
          <div className="flex flex-col gap-1.5">
            <label className="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
              Max Retry Limit
            </label>
            <Input
              type="number"
              value={retryLimit}
              onChange={(e) => setRetryLimit(e.target.value)}
              min="1"
              max="20"
            />
          </div>
        </div>
        <DialogFooter>
          <Button variant="outline" onClick={onClose} disabled={isPending}>
            Cancel
          </Button>
          <Button onClick={handleSubmit} disabled={isPending}>
            {isPending ? "Subscribing..." : "Create Subscription"}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};

const SubscriptionsComponent = () => {
  const queryClient = useQueryClient();
  const [createOpen, setCreateOpen] = useState(false);

  const { params: queryParams, onQueryChange } = useTableQueryParams({
    defaultLimit: 10,
    defaultSort: "created_at_desc",
    defaultFilters: { status: "all" },
  });

  const status = queryParams.filters.status;
  const { data, isLoading, isError, isFetching } = useListEventSubscriptions(
    {
      active: status === "active" ? true : status === "inactive" ? false : undefined,
      limit: queryParams.limit,
      page: queryParams.cursor ? undefined : queryParams.page,
      cursor: queryParams.cursor,
      sort: queryParams.sort as ListEventSubscriptionsParams["sort"],
    },
    { query: { placeholderData: keepPreviousData } },
  );
  const subscriptions: EventSubscription[] = Array.isArray(data?.data)
    ? (data.data as EventSubscription[])
    : [];

  const { mutate: deleteSub } = useDeleteEventSubscription({
    mutation: {
      onSuccess: () => {
        queryClient.invalidateQueries({ queryKey: getListEventSubscriptionsQueryKey() });
        toast.success("Subscription deleted");
      },
      onError: (err) => {
        toast.error(`Delete failed: ${String(err)}`);
      },
    },
  });

  return (
    <div className="flex flex-col gap-6 w-full">
      <PageSection
        title="Webhooks"
        action={
          <Button size="sm" onClick={() => setCreateOpen(true)}>
            <Plus /> New subscription
          </Button>
        }
      >
        {isLoading && !data ? (
          <div className="flex flex-col gap-3 p-4">
            <Skeleton className="h-12 w-full" />
            <Skeleton className="h-12 w-full" />
          </div>
        ) : isError && !data ? (
          <div className="p-8 text-center text-sm text-destructive">
            Failed to load event subscriptions.
          </div>
        ) : (
          <DataTable
            className="text-sm"
            data={subscriptions}
            serverSide={true}
            loading={isFetching}
            queryParams={queryParams}
            onQueryChange={onQueryChange}
            keyExtractor={(sub) => sub.id}
            searchPlaceholder="Filter subscriptions by topic or callback URL..."
            emptyMessage="No webhook subscriptions configured yet."
            defaultSortKey="created_at"
            defaultSortDirection="desc"
            pageSize={10}
            pageSizeOptions={[10, 20, 50, 100]}
            filters={[
              {
                id: "status",
                label: "Status",
                value: queryParams.filters.status ?? "all",
                options: [
                  { label: "Active", value: "active" },
                  { label: "Inactive", value: "inactive" },
                ],
              },
            ]}
            columns={[
              {
                header: "Topic pattern",
                accessorKey: "topic_pattern",
                sortable: false,
                cell: (sub) => <Badge variant="code">{sub.topic_pattern}</Badge>,
              },
              {
                header: "Status",
                accessorKey: "active",
                sortable: false,
                cell: (sub) => (
                  <Badge variant="status" state={sub.active ? "ACTIVE" : "PAUSE"}>
                    {sub.active ? "Active" : "Inactive"}
                  </Badge>
                ),
              },
              {
                header: "Callback",
                accessorKey: "callback_address",
                sortable: false,
                cell: (sub) => (
                  <span className="font-mono text-xs block max-w-[280px] truncate">
                    {sub.callback_address}
                  </span>
                ),
              },
              {
                header: "Signature",
                sortable: false,
                searchable: false,
                cell: (sub) =>
                  sub.secret ? (
                    <Badge variant="success">
                      <ShieldCheck className="h-3 w-3" /> HMAC
                    </Badge>
                  ) : (
                    <span className="text-xs text-muted-foreground">None</span>
                  ),
              },
              {
                header: "Retries",
                accessorKey: "retry_limit",
                sortable: false,
                cell: (sub) => <Badge variant="infoLighter">{sub.retry_limit ?? 5}</Badge>,
              },
              {
                header: "Created at",
                accessorKey: "created_at",
                cell: (sub) => <FormatDate date={sub.created_at} />,
              },
              {
                header: "Actions",
                sortable: false,
                searchable: false,
                cell: (sub) => (
                  <Button
                    variant="ghost"
                    size="icon"
                    className="h-7 w-7 text-destructive hover:text-destructive hover:bg-destructive/10"
                    onClick={() => {
                      if (confirm(`Delete subscription for "${sub.topic_pattern}"?`)) {
                        deleteSub({ subscriptionId: sub.id });
                      }
                    }}
                  >
                    <Trash2 className="h-3.5 w-3.5" />
                  </Button>
                ),
              },
            ]}
          />
        )}
      </PageSection>

      <CreateSubscriptionDialog open={createOpen} onClose={() => setCreateOpen(false)} />
    </div>
  );
};

export const Route = createFileRoute("/events/subscriptions")({
  component: SubscriptionsComponent,
});
