import { createFileRoute, Outlet } from "@tanstack/react-router";
import { useContext } from "react";
import { WalletLayout } from "@eunomia/ui";
import { SSIAuthContext } from "shared/src/context/SSIAuthContext";

const WalletRouteLayout = () => {
  const { ownWalletOnboarded, onboardInWallet, isLoading } = useContext(SSIAuthContext);

  return (
    <WalletLayout
      basePath="/wallet"
      onboarded={ownWalletOnboarded}
      onLink={onboardInWallet}
      isLinking={isLoading.onboard}
    >
      <Outlet />
    </WalletLayout>
  );
};

export const Route = createFileRoute("/wallet")({
  component: WalletRouteLayout,
});
