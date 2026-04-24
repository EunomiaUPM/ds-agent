import { createFileRoute, Link, Outlet, useRouterState } from "@tanstack/react-router";
import { useContext } from "react";
import { SSIAuthContext } from "shared/src/context/SSIAuthContext";
import { PageLayout } from "shared/src/components/layout/PageLayout";
import { PageHeader } from "shared/src/components/layout/PageHeader";
import { Button } from "shared/src/components/ui/button";
import { cn } from "shared/src/lib/utils";
import { Wallet } from "lucide-react";

/**
 * Wallet route layout.
 * Provides the main navigation tabs for the wallet sub-sections.
 */
const WalletLayout = () => {
  const { ownWalletOnboarded, onboardInWallet, isLoading } = useContext(SSIAuthContext);
  const routerState = useRouterState();
  const pathname = routerState.location.pathname;

  const tabs = [
    { label: "DID", to: "/wallet/did" },
    { label: "Info", to: "/wallet/info" },
    { label: "Credentials", to: "/wallet/credentials" },
    { label: "OIDC4VP", to: "/wallet/oidc4vp" },
    { label: "OIDC4VCI", to: "/wallet/oidc4vci" },
  ];

  return (
    <PageLayout>
      <PageHeader title="Wallet">
        <div className="flex items-center gap-3">
          <Button
            onClick={onboardInWallet}
            isLoading={isLoading.onboard}
            variant={ownWalletOnboarded ? "outline" : "default"}
            size="sm"
            className="flex gap-2"
          >
            <Wallet className="h-4 w-4" />
            {isLoading.onboard
              ? "Linking..."
              : ownWalletOnboarded
              ? "Re-Link Wallet"
              : "Link Wallet"}
          </Button>
        </div>
      </PageHeader>

      {!ownWalletOnboarded ? (
        <div className="flex flex-col items-center justify-center min-h-[400px] border border-dashed border-white/10 rounded-xl bg-white/5 p-12">
           <Wallet className="h-12 w-12 text-muted-foreground/40 mb-4" />
           <p className="text-muted-foreground font-medium text-lg">Wallet not linked</p>
           <p className="text-muted-foreground/60 text-sm mt-1 mb-6">Link your wallet to start managing your identity.</p>
           <Button onClick={onboardInWallet} isLoading={isLoading.onboard}>
              Link Wallet Now
           </Button>
        </div>
      ) : (
        <>
          <div className="flex gap-1 border-b border-white/10 mb-6 w-full">
            {tabs.map((tab) => {
              const isActive = pathname === tab.to || pathname.startsWith(tab.to + "/");
              return (
                <Link
                  key={tab.to}
                  to={tab.to}
                  className={cn(
                    "px-4 py-2 text-sm font-medium border-b-2 transition-all",
                    isActive
                      ? "border-primary text-primary"
                      : "border-transparent text-muted-foreground hover:text-foreground"
                  )}
                >
                  {tab.label}
                </Link>
              );
            })}
          </div>
          <Outlet />
        </>
      )}
    </PageLayout>
  );
};

export const Route = createFileRoute("/wallet")({
  component: WalletLayout,
});
