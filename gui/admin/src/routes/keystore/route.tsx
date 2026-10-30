import { createFileRoute, Outlet } from "@tanstack/react-router";
import { RouteTabs } from "@eunomia/ui";
import { PageLayout } from "@eunomia/ui";
import { PageHeader } from "@eunomia/ui";
import { KeyRound } from "lucide-react";

const KeystoreLayout = () => {
  const tabs = [
    { label: "Parameters", to: "/keystore/parameters" },
    { label: "Secrets", to: "/keystore/secrets" },
    { label: "Config", to: "/keystore/config" },
  ];

  return (
    <PageLayout>
      <PageHeader
        title="Keystore"
        description="Parameters and secrets the agents read at runtime, and the configuration they run with."
      />

      <RouteTabs tabs={tabs} />

      <Outlet />
    </PageLayout>
  );
};

export const Route = createFileRoute("/keystore")({
  component: KeystoreLayout,
});
