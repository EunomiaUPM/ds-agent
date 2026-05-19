import { createFileRoute, redirect } from "@tanstack/react-router";

export const Route = createFileRoute("/keystore/")({
  beforeLoad: () => {
    throw redirect({ to: "/keystore/parameters" });
  },
});
