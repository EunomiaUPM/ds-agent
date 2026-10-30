import { createFileRoute } from "@tanstack/react-router";
import { WalletInfoView } from "@eunomia/ui";

export const Route = createFileRoute("/wallet/info")({
  component: WalletInfoView,
});
