import { createFileRoute } from "@tanstack/react-router";
import { useQuery } from "@tanstack/react-query";
import { customInstance } from "shared/src/data/orval-mutator";
import { PageSection } from "shared/src/components/layout/PageSection";
import { Skeleton } from "shared/src/components/ui/skeleton";
import { ShieldAlert } from "lucide-react";

interface VcsResponse {
  status: number;
  data: any[];
}

const WalletCredentials = () => {
  const { data: response, isLoading, error } = useQuery({
    queryKey: ["wallet-vcs-custom"],
    queryFn: () => customInstance<VcsResponse>("/wallet/vcs", { method: "GET" }),
  });

  const credentials = response?.status === 200 ? response.data : [];

  if (isLoading) {
    return (
      <PageSection title="Verifiable Credentials">
        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
          <Skeleton className="h-48 w-full" />
          <Skeleton className="h-48 w-full" />
        </div>
      </PageSection>
    );
  }

  if (error) {
    return <div className="text-destructive font-mono text-xs text-center py-10">Error loading credentials</div>;
  }

  return (
    <PageSection title="Verifiable Credentials">
      {credentials.length === 0 ? (
        <div className="flex flex-col items-center justify-center py-20 text-muted-foreground border border-dashed border-white/5 rounded-xl bg-white/2">
           <ShieldAlert className="h-10 w-10 opacity-20 mb-3" />
           <p className="text-sm">No credentials found in this wallet.</p>
        </div>
      ) : (
        <div className="grid grid-cols-1 gap-6 pb-10">
          {credentials.map((vc, idx) => (
            <div key={idx} className="bg-white/5 border border-white/10 rounded-xl p-6 shadow-xl transition-all hover:border-primary/20 hover:bg-white/[0.07]">
              <pre className="text-xs font-mono text-muted-foreground/90 whitespace-pre-wrap break-all leading-normal">
                {JSON.stringify(vc, null, 2)}
              </pre>
            </div>
          ))}
        </div>
      )}
    </PageSection>
  );
};

export const Route = createFileRoute("/wallet/credentials")({
  component: WalletCredentials,
});
