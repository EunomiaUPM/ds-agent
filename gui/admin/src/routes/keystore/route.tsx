import { createFileRoute, Link, Outlet, useRouterState } from "@tanstack/react-router";
import { PageLayout } from "shared/src/components/layout/PageLayout";
import { PageHeader } from "shared/src/components/layout/PageHeader";
import { cn } from "shared/src/lib/utils";
import { KeyRound } from "lucide-react";

const KeystoreLayout = () => {
  const routerState = useRouterState();
  const pathname = routerState.location.pathname;

  const tabs = [
    { label: "Parameters", to: "/keystore/parameters" },
    { label: "Secrets", to: "/keystore/secrets" },
    { label: "Config", to: "/keystore/config" },
  ];

  return (
    <PageLayout>
      <PageHeader title="Keystore"></PageHeader>

      <div className="flex gap-1 border-b border-white/10 mb-6 w-full">
        {tabs.map((tab) => {
          const isActive = pathname === tab.to || pathname.startsWith(tab.to + "/");
          return (
            <Link
              key={tab.to}
              to={tab.to}
              className={cn(
                "px-4 py-2 text-sm font-medium border-b-2 transition-all",
                isActive
                  ? "border-primary text-primary"
                  : "border-transparent text-muted-foreground hover:text-foreground",
              )}
            >
              {tab.label}
            </Link>
          );
        })}
      </div>

      <Outlet />
    </PageLayout>
  );
};

export const Route = createFileRoute("/keystore")({
  component: KeystoreLayout,
});
