import { createFileRoute, Link, Outlet, useRouterState } from "@tanstack/react-router";
import { PageLayout } from "shared/src/components/layout/PageLayout";
import { PageHeader } from "shared/src/components/layout/PageHeader";
import { cn } from "shared/src/lib/utils";

const tabs = [
  { label: "Sent", to: "/connections/sent" },
  { label: "Received", to: "/connections/received" },
];

const ConnectionsLayout = () => {
  const routerState = useRouterState();
  const pathname = routerState.location.pathname;

  return (
    <PageLayout>
      <PageHeader title="My Connections" />

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

export const Route = createFileRoute("/connections")({
  component: ConnectionsLayout,
});
