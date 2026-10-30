import { createFileRoute } from "@tanstack/react-router";
import { WalletOidc4vpView } from "@eunomia/ui";

export const Route = createFileRoute("/wallet/oidc4vp")({
  component: WalletOidc4vpView,
});
