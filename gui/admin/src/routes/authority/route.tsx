import { createFileRoute, Outlet } from "@tanstack/react-router";

/**
 * Authority route layout.
 */
const AuthorityRoute = () => {
  return <Outlet />;
};

export const Route = createFileRoute("/authority")({
  component: AuthorityRoute,
});
