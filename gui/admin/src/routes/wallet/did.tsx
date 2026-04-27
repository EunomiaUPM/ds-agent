import { createFileRoute } from "@tanstack/react-router";
import { useGetWalletDid } from "shared/data/orval/wallet/wallet";
import { PageSection } from "shared/src/components/layout/PageSection";
import { Skeleton } from "shared/src/components/ui/skeleton";

const WalletDID = () => {
  const { data: didDoc, isLoading, error } = useGetWalletDid();

  if (isLoading) {
    return (
      <PageSection title="DID Document">
        <Skeleton className="h-64 w-full" />
      </PageSection>
    );
  }

  if (error || didDoc?.status !== 200) {
    return <div className="text-destructive font-mono text-xs">Error loading DID document</div>;
  }

  return (
    <PageSection title="DID Document">
      <div className="bg-black/20 rounded-xl border border-white/5 p-6 shadow-inner">
        <pre className="text-xs md:text-sm font-mono text-muted-foreground/80 whitespace-pre-wrap break-all leading-relaxed">
          {JSON.stringify(didDoc.data, null, 2)}
        </pre>
      </div>
    </PageSection>
  );
};

export const Route = createFileRoute("/wallet/did")({
  component: WalletDID,
});
