import { createFileRoute } from "@tanstack/react-router";
// import { PubSubContext } from "shared/src/context/PubSubContext.tsx";
import {
  useGetSubscriptionById,
  useGetNotificationsBySubscriptionId,
} from "shared/src/data/orval/subscriptions/subscriptions";
import { GeneralErrorComponent } from "@/components/GeneralErrorComponent";
import { formatUrn } from "shared/lib/utils";
import { DataTable } from "shared/src/components/DataTable";
import { FormatDate } from "shared/src/components/ui/format-date";
import { Badge } from "shared/src/components/ui/badge";
import { PageLayout } from "shared/components/layout/PageLayout";
import { PageHeader } from "shared/components/layout/PageHeader";
import { PageSection } from "shared/components/layout/PageSection";
import { Skeleton } from "shared/components/ui/skeleton";

const RouteComponent = () => {
  // TODO: PubSubContext was deleted. Restore subscriptionId source.
  const subscriptionId = ""; // useContext(PubSubContext)!;

  const { data: subscription, isLoading: isSubscriptionLoading } = useGetSubscriptionById(
    subscriptionId!,
  );
  const { data: notifications, isLoading: isNotificationsLoading } =
    useGetNotificationsBySubscriptionId(subscriptionId!);

  if (isSubscriptionLoading || isNotificationsLoading) {
    return (
      <PageLayout>
        <PageHeader title="Subscription" badge={<Skeleton className="h-8 w-48" />} />
        <div>Loading...</div>
      </PageLayout>
    );
  }

  // handle error
  if (!subscription || subscription.status !== 200) {
    return <GeneralErrorComponent error={new Error("Subscription not found")} reset={() => {}} />;
  }

  if (!notifications || notifications.status !== 200) {
    return <GeneralErrorComponent error={new Error("Notifications not found")} reset={() => {}} />;
  }

  const subscriptionInfo = [
    { key: "Subscription id", value: subscription.data.id },
    { key: "Subscription callback address", value: subscription.data.callbackAddress },
    {
      key: "Subscription creation date",
      value: <FormatDate date={subscription.data.createdAt} />,
    },
  ];

  return (
    <PageLayout>
      <PageHeader title="Subscription" />

      <PageSection title="Main catalog info">
        <DataTable
          className="text-sm"
          data={subscriptionInfo}
          keyExtractor={(row) => row.key}
          hideToolbar
          columns={[
            { header: "Key", accessorKey: "key" },
            { header: "Value", sortable: false, cell: (row) => row.value },
          ]}
        />
      </PageSection>

      <PageSection title="Notifications">
        <DataTable
          className="text-sm"
          data={notifications.data}
          keyExtractor={(n) => n.id!}
          searchPlaceholder="Filter notifications by id, category, or message type..."
          emptyMessage="No notifications received for this subscription"
          defaultSortKey="createdAt"
          defaultSortDirection="desc"
          pageSize={10}
          columns={[
            {
              header: "Notification Id",
              accessorKey: "id",
              cell: (n) => <Badge variant="info">{formatUrn(n.id)}</Badge>,
            },
            {
              header: "Category",
              searchValue: (n) => (n.event?.category as string) ?? "",
              sortValue: (n) => (n.event?.category as string) ?? "",
              cell: (n) => (n.event?.category as string) ?? "",
            },
            {
              header: "Subcategory",
              searchValue: (n) => (n.event?.subcategory as string) ?? "",
              sortValue: (n) => (n.event?.subcategory as string) ?? "",
              cell: (n) => (n.event?.subcategory as string) ?? "",
            },
            {
              header: "Message Type",
              searchValue: (n) => (n.event?.messageType as string) ?? "",
              sortValue: (n) => (n.event?.messageType as string) ?? "",
              cell: (n) => (n.event?.messageType as string) ?? "",
            },
            {
              header: "Message operation",
              searchValue: (n) => (n.event?.messageOperation as string) ?? "",
              sortValue: (n) => (n.event?.messageOperation as string) ?? "",
              cell: (n) => (n.event?.messageOperation as string) ?? "",
            },
            {
              header: "Message timestamp",
              accessorKey: "createdAt",
              sortValue: (n) => new Date(n.createdAt!).getTime(),
              cell: (n) => <FormatDate date={n.createdAt} />,
            },
          ]}
        />
      </PageSection>
    </PageLayout>
  );
};

/**
 * Route for displaying subscription details and notifications.
 */
export const Route = createFileRoute("/subscriptions/")({
  component: RouteComponent,
  pendingComponent: () => <div>Loading...</div>,
});
