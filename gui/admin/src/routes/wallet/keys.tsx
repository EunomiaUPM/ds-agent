import { createFileRoute } from "@tanstack/react-router";
import { WalletKeysView } from "@eunomia/ui";

export const Route = createFileRoute("/wallet/keys")({
  component: WalletKeysView,
});
