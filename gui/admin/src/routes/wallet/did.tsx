import { createFileRoute } from "@tanstack/react-router";
import { WalletDidView } from "@eunomia/ui";

export const Route = createFileRoute("/wallet/did")({
  component: WalletDidView,
});
