import { createFileRoute } from "@tanstack/react-router";
// import { PubSubContext } from "shared/src/context/PubSubContext.tsx";
import { useContext } from "react";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "shared/src/components/ui/table";
import dayjs from "dayjs";
import { useGetSubscriptions, useGetSubscriptionById, useGetNotificationsBySubscriptionId } from "shared/src/data/orval/subscriptions/subscriptions";
import { GeneralErrorComponent } from "@/components/GeneralErrorComponent";
import { formatUrn } from "shared/lib/utils";
import { PageLayout } from "shared/components/layout/PageLayout";
import { PageHeader } from "shared/components/layout/PageHeader";
import { Skeleton } from "shared/components/ui/skeleton";

const RouteComponent = () => {
  // TODO: PubSubContext was deleted. Restore subscriptionId source.
  const subscriptionId = ""; // useContext(PubSubContext)!;

  const { data: subscription, isLoading: isSubscriptionLoading } = useGetSubscriptionById(subscriptionId!);
  const { data: notifications, isLoading: isNotificationsLoading } = useGetNotificationsBySubscriptionId(subscriptionId!);


  if (isSubscriptionLoading || isNotificationsLoading) {
    return (
      <PageLayout>
        <PageHeader
          title="Subscription"
          badge={<Skeleton className="h-8 w-48" />}
        />
        <div>Loading...</div>
      </PageLayout>
    );
  }


  // handle error
  if (!subscription || subscription.status !== 200) {
    return <GeneralErrorComponent error={new Error("Subscription not found")} reset={() => { }} />;
  }

  if (!notifications || notifications.status !== 200) {
    return <GeneralErrorComponent error={new Error("Notifications not found")} reset={() => { }} />;
  }

  return (
    <div className="space-y-4">
      <h1 className="text-xl font-bold">Subscription</h1>
      <div>Subscription with id : {subscription.data.id}</div>
      <div>
        <h2>Main Catalog info: </h2>
        <Table className="text-sm">
          <TableHeader>
            <TableRow>
              <TableHead>Key</TableHead>
              <TableHead>Value</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            <TableRow>
              <TableCell>Subscription callback address</TableCell>
              <TableCell>{subscription.data.callbackAddress}</TableCell>
            </TableRow>
            <TableRow>
              <TableCell>Subscription creation date</TableCell>
              <TableCell>{dayjs(subscription.data.createdAt).format("DD/MM/YYYY - HH:mm")}</TableCell>
            </TableRow>
          </TableBody>
        </Table>
      </div>

      <div>
        <h2>Notifications</h2>
        <Table className="text-sm">
          <TableHeader>
            <TableRow>
              <TableHead>Notification Id</TableHead>
              <TableHead>Category</TableHead>
              <TableHead>Subcategory</TableHead>
              <TableHead>Message Type</TableHead>
              <TableHead>Message operation</TableHead>
              <TableHead>Message content</TableHead>
              <TableHead>Message timestamp</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {notifications?.data.map((notification) => (
              <TableRow key={formatUrn(notification.id)}>
                <TableCell>{formatUrn(notification.id)}</TableCell>
                <TableCell>{notification.event ? (notification.event.category as string) : ""}</TableCell>
                <TableCell>{notification.event ? (notification.event.subcategory as string) : ""}</TableCell>
                <TableCell>{notification.event ? (notification.event.messageType as string) : ""}</TableCell>
                <TableCell>{notification.event ? (notification.event.messageOperation as string) : ""}</TableCell>
                <TableCell>{dayjs(notification.createdAt).format("DD/MM/YYYY - HH:mm")}</TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </div>
    </div>
  );
};

/**
 * Route for displaying subscription details and notifications.
 */
export const Route = createFileRoute("/subscriptions/")({
  component: RouteComponent,
  pendingComponent: () => <div>Loading...</div>,
});
