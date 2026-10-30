import React from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Loader2, RefreshCw, Wallet } from "lucide-react";
import { toast } from "sonner";
import { customInstance } from "../lib/http";
import { PageLayout } from "../layout/PageLayout";
import { PageHeader } from "../layout/PageHeader";
import { RouteTabs, type RouteTab } from "../layout/RouteTabs";
import { Button } from "../ui/button";

export interface WalletLayoutProps {
  /** Base route prefix for wallet tabs; defaults to "/wallet" */
  basePath?: string;
  /** Optional controlled linked state (e.g., from SSIAuthContext in DS-Agent) */
  onboarded?: boolean;
  /** Optional controlled link callback */
  onLink?: () => void | Promise<void>;
  /** Optional controlled linking spinner state */
  isLinking?: boolean;
  children?: React.ReactNode;
}

export function WalletLayout({
  basePath = "/wallet",
  onboarded: controlledOnboarded,
  onLink: controlledOnLink,
  isLinking: controlledIsLinking,
  children,
}: WalletLayoutProps) {
  const queryClient = useQueryClient();
  const isControlled = controlledOnboarded !== undefined;

  const { data: linkedRes, isLoading: checkingLink } = useQuery({
    queryKey: ["wallet-is-linked"],
    queryFn: () =>
      customInstance<{ status: number; data: boolean | { linked?: boolean } }>(
        "/wallet/is-linked",
        { method: "GET" },
      ),
    enabled: !isControlled,
  });

  const linkMutation = useMutation({
    mutationFn: () => customInstance("/wallet/link", { method: "POST" }),
    onSuccess: async () => {
      toast.success("Wallet linked");
      await queryClient.invalidateQueries({ queryKey: ["wallet-is-linked"] });
      await queryClient.invalidateQueries({ queryKey: ["wallet-info-custom"] });
    },
    onError: (err) => {
      toast.error(err instanceof Error ? err.message : "Failed to link wallet");
    },
  });

  const rawData = linkedRes?.data;
  const autoLinked =
    linkedRes?.status === 200 &&
    (typeof rawData === "boolean"
      ? rawData
      : typeof rawData === "object" && rawData !== null && "linked" in rawData
        ? Boolean(rawData.linked)
        : true);

  const ownWalletOnboarded = isControlled ? controlledOnboarded : autoLinked;
  const isLinking = isControlled ? Boolean(controlledIsLinking) : linkMutation.isPending;
  const handleLink = () => {
    if (controlledOnLink) {
      void controlledOnLink();
    } else {
      linkMutation.mutate();
    }
  };

  const cleanBase = basePath.replace(/\/$/, "");
  const tabs: RouteTab[] = [
    { label: "Info", to: `${cleanBase}/info` },
    { label: "DID", to: `${cleanBase}/did` },
    { label: "Keys", to: `${cleanBase}/keys` },
    { label: "Credentials", to: `${cleanBase}/credentials` },
    { label: "OID4VP", to: `${cleanBase}/oidc4vp` },
    { label: "OID4VCI", to: `${cleanBase}/oidc4vci` },
  ];

  return (
    <PageLayout>
      <PageHeader
        title="Wallet"
        description="Decentralized identity: DIDs, keys and verifiable credentials."
        actions={
          ownWalletOnboarded && (
            <Button onClick={handleLink} isLoading={isLinking} variant="outline" size="sm">
              {!isLinking && <RefreshCw />}
              {isLinking ? "Relinking…" : "Relink wallet"}
            </Button>
          )
        }
      />

      {!isControlled && checkingLink ? (
        <div className="flex items-center justify-center min-h-[300px]">
          <Loader2 className="h-6 w-6 animate-spin text-brand-sky" />
        </div>
      ) : !ownWalletOnboarded ? (
        <div className="flex flex-col items-center justify-center min-h-[400px] border border-dashed border-ink/10 rounded-xl bg-ink/5 p-12">
          <Wallet className="h-12 w-12 text-muted-foreground/40 mb-4" />
          <p className="text-muted-foreground font-medium text-lg">Wallet not linked</p>
          <p className="text-muted-foreground/60 text-sm mt-1 mb-6">
            Link your wallet to start managing your identity.
          </p>
          <Button onClick={handleLink} isLoading={isLinking}>
            Link wallet
          </Button>
        </div>
      ) : (
        <>
          <RouteTabs tabs={tabs} />
          {children}
        </>
      )}
    </PageLayout>
  );
}
