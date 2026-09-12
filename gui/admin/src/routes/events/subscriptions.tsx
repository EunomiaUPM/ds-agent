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
  useListEventSubscriptions,
  useCreateEventSubscription,
  useDeleteEventSubscription,
  getListEventSubscriptionsQueryKey,
} from "shared/src/data/orval/events/events";
import { EventSubscription } from "shared/src/data/orval/model";
import { PageSection } from "shared/src/components/layout/PageSection";
import { DataTable } from "shared/src/components/DataTable";
import { FormatDate } from "shared/src/components/ui/format-date";
import { Skeleton } from "shared/src/components/ui/skeleton";
import { Button } from "shared/src/components/ui/button";
import { Badge } from "shared/src/components/ui/badge";
import { Input } from "shared/src/components/ui/input";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from "shared/src/components/ui/dialog";
import { Plus, Trash2, ShieldCheck } from "lucide-react";
import { toast } from "sonner";

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
    { label: "transfers:*", pattern: "transfers:*" },
    { label: "transfers:bla", pattern: "transfers:bla" },
    { label: "transfers:**", pattern: "transfers:**" },
    { label: "catalog:*", pattern: "catalog:*" },
    { label: "* (All)", pattern: "*" },
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
              placeholder="e.g. transfers:*, transfers:bla, or catalog:*"
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
  const [patternFilter, setPatternFilter] = useState("");

  const { data, isLoading, isError } = useListEventSubscriptions();
  const subscriptions: EventSubscription[] = Array.isArray(data?.data)
    ? (data.data as EventSubscription[])
    : [];

  const filteredSubscriptions = patternFilter
    ? subscriptions.filter((sub) =>
        sub.topic_pattern.toLowerCase().includes(patternFilter.toLowerCase()),
      )
    : subscriptions;

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
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-lg font-semibold tracking-tight">External Webhook Subscriptions</h2>
          <p className="text-xs text-muted-foreground">
            Configure external listeners that receive HTTP POST notifications with exponential
            backoff and HMAC-SHA256 signatures.
          </p>
        </div>
        <Button onClick={() => setCreateOpen(true)} className="gap-2">
          <Plus className="h-4 w-4" /> New Subscription
        </Button>
      </div>

      {/* Quick Pattern Filter Chips */}
      <div className="flex items-center gap-2">
        <Button
          variant={patternFilter === "" ? "default" : "outline"}
          size="sm"
          className="text-xs h-7"
          onClick={() => setPatternFilter("")}
        >
          All
        </Button>
        {["transfers:*", "transfers:bla", "transfers:**", "catalog:*"].map((tag) => (
          <Button
            key={tag}
            variant={patternFilter === tag ? "default" : "outline"}
            size="sm"
            className="text-xs h-7 font-mono"
            onClick={() => setPatternFilter(patternFilter === tag ? "" : tag)}
          >
            {tag}
          </Button>
        ))}
      </div>

      <PageSection>
        {isLoading ? (
          <div className="flex flex-col gap-3 p-4">
            <Skeleton className="h-12 w-full" />
            <Skeleton className="h-12 w-full" />
          </div>
        ) : isError ? (
          <div className="p-8 text-center text-sm text-destructive">
            Failed to load event subscriptions.
          </div>
        ) : (
          <DataTable
            className="text-sm"
            data={filteredSubscriptions}
            keyExtractor={(sub) => sub.id}
            searchPlaceholder="Filter subscriptions by topic or callback URL..."
            emptyMessage="No webhook subscriptions configured yet."
            defaultSortKey="created_at"
            defaultSortDirection="desc"
            columns={[
              {
                header: "Topic pattern",
                accessorKey: "topic_pattern",
                cell: (sub) => <Badge variant="code">{sub.topic_pattern}</Badge>,
              },
              {
                header: "Status",
                accessorKey: "active",
                cell: (sub) => (
                  <Badge variant="status" state={sub.active ? "ACTIVE" : "PAUSE"}>
                    {sub.active ? "Active" : "Inactive"}
                  </Badge>
                ),
              },
              {
                header: "Callback",
                accessorKey: "callback_address",
                cell: (sub) => (
                  <span className="font-mono text-xs block max-w-[280px] truncate">
                    {sub.callback_address}
                  </span>
                ),
              },
              {
                header: "Signature",
                sortValue: (sub) => Boolean(sub.secret),
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
                cell: (sub) => <Badge variant="infoLighter">{sub.retry_limit ?? 5}</Badge>,
              },
              {
                header: "Created at",
                accessorKey: "created_at",
                sortValue: (sub) => new Date(sub.created_at).getTime(),
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
