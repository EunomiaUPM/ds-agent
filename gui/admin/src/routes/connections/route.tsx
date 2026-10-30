import { createFileRoute, Outlet } from "@tanstack/react-router";
import { RouteTabs } from "@eunomia/ui";
import { PageLayout } from "@eunomia/ui";
import { PageHeader } from "@eunomia/ui";

const tabs = [
  { label: "Sent", to: "/connections/sent" },
  { label: "Received", to: "/connections/received" },
];

const ConnectionsLayout = () => {
  return (
    <PageLayout>
      <PageHeader title="My Connections" />

      <RouteTabs tabs={tabs} />
      <Outlet />
    </PageLayout>
  );
};

export const Route = createFileRoute("/connections")({
  component: ConnectionsLayout,
});
