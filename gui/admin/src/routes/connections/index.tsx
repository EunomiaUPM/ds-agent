import { createFileRoute, redirect } from "@tanstack/react-router";

export const Route = createFileRoute("/connections/")({
  beforeLoad: () => {
    throw redirect({ to: "/connections/sent" });
  },
});
