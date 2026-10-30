import { createFileRoute } from "@tanstack/react-router";
import { WalletCredentialsView } from "@eunomia/ui";

const WalletCredentialsPage = () => <WalletCredentialsView showGaiaCompliance={true} />;

export const Route = createFileRoute("/wallet/credentials")({
  component: WalletCredentialsPage,
});
