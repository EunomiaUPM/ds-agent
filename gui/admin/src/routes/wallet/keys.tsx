import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/wallet/keys")({
  component: RouteComponent,
});

function RouteComponent() {
  return <div>Hello "/wallet/keys"!</div>;
}
