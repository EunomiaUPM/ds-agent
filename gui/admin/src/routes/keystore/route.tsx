import { createFileRoute, Outlet } from "@tanstack/react-router";
import { RouteTabs } from "shared/src/components/layout/RouteTabs";
import { PageLayout } from "shared/src/components/layout/PageLayout";
import { PageHeader } from "shared/src/components/layout/PageHeader";
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
