import { createFileRoute } from "@tanstack/react-router";
import { WalletOidc4vciView } from "@eunomia/ui";

export const Route = createFileRoute("/wallet/oidc4vci")({
  component: WalletOidc4vciView,
});
