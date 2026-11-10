import { createFileRoute } from "@tanstack/react-router";
import { WalletCredentials } from "@/components/credentials/WalletCredentials";

const WalletCredentialsPage = () => <WalletCredentials showGaiaCompliance={true} />;

export const Route = createFileRoute("/wallet/credentials")({
  component: WalletCredentialsPage,
});
