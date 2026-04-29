import { createFileRoute, Outlet } from "@tanstack/react-router";

/**
 * Providers route layout.
 */
const ProvidersRoute = () => {
  return <Outlet />;
};

export const Route = createFileRoute("/providers")({
  component: ProvidersRoute,
});
