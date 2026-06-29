import { createFileRoute, Outlet } from "@tanstack/react-router";

export const Route = createFileRoute("/connections/sent")({
  component: () => <Outlet />,
});
