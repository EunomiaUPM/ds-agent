import { createFileRoute } from "@tanstack/react-router";

// Auth is handled by the root component.
// This file only exists to register /login/ in the route tree.
export const Route = createFileRoute("/login/")({
  component: () => null,
});
