import { createFileRoute } from "@tanstack/react-router";
import { useQuery } from "@tanstack/react-query";
import { customInstance } from "shared/src/data/orval-mutator";
import { PageSection } from "shared/src/components/layout/PageSection";
import { DataTable } from "shared/src/components/DataTable";
import { Skeleton } from "shared/src/components/ui/skeleton";
import { Badge } from "shared/src/components/ui/badge";
import { InfoList } from "shared/src/components/ui/info-list";
import { ChevronRight, ChevronDown } from "lucide-react";
import { useState } from "react";
import { cn } from "shared/src/lib/utils";

interface WalletInfoResponse {
  status: number;
  data: {
    id: string;
    name: string;
    createdOn: string;
    addedOn: string;
    permission: string;
    dids: {
      did: string;
      alias: string;
      keyId: string;
      default: boolean;
      createdOn: string;
      document: string;
    }[];
  }
}

const WalletInfoPage = () => {
  const { data: response, isLoading, error } = useQuery({
    queryKey: ["wallet-info-custom"],
    queryFn: () => customInstance<WalletInfoResponse>("/wallet/info", { method: "GET" }),
  });

  const info = response?.status === 200 ? response.data : null;

  if (isLoading) {
    return (
      <div className="space-y-8">
        <Skeleton className="h-40 w-full" />
        <Skeleton className="h-64 w-full" />
      </div>
    );
  }

  if (error || !info) {
    return <div className="text-destructive font-mono text-xs">Error loading wallet info</div>;
  }

  return (
    <div className="space-y-12 pb-20">
      <PageSection title="General Information">
        <div className="bg-white/5 border border-white/10 rounded-xl p-8 backdrop-blur-sm shadow-xl">
           <InfoList 
             items={[
               { label: "Wallet ID", value: info.id },
               { label: "Name", value: info.name },
               { label: "Created On", value: info.createdOn },
               { label: "Permission Level", value: info.permission },
             ]}
           />
        </div>
      </PageSection>

      <PageSection title="Associated DIDs">
        <DataTable
          data={info.dids}
          keyExtractor={(d) => d.did}
          columns={[
            {
              header: "Alias",
              accessorKey: "alias",
              cell: (d) => <span className="font-semibold text-primary/80">{d.alias}</span>
            },
            {
              header: "DID",
              accessorKey: "did",
              cell: (d) => <span className="font-mono text-[10px] text-muted-foreground break-all">{d.did}</span>
            },
            {
              header: "Default",
              accessorKey: "default",
              cell: (d) => (
                <Badge variant={d.default ? "default" : "info"}>
                  {d.default ? "PRIMARY" : "SECONDARY"}
                </Badge>
              )
            },
            {
              header: "Created",
              accessorKey: "createdOn",
            }
          ]}
        />
      </PageSection>

      <PageSection title="DID Documents Details">
        <div className="space-y-4">
          {info.dids.map((did) => (
            <DidDocItem key={did.did} did={did} />
          ))}
        </div>
      </PageSection>
    </div>
  );
};

const DidDocItem = ({ did }: { did: any }) => {
  const [isOpen, setIsOpen] = useState(false);
  return (
    <div className="group border border-white/10 rounded-lg overflow-hidden bg-black/10 transition-all hover:bg-black/20">
      <button 
        onClick={() => setIsOpen(!isOpen)}
        className="w-full flex items-center justify-between p-4 text-left transition-colors"
      >
        <div className="flex items-center gap-3">
           <div className={cn("transition-transform duration-200", isOpen ? "rotate-90" : "")}>
             <ChevronRight className="h-4 w-4 text-muted-foreground" />
           </div>
           <span className="text-sm font-medium">{did.alias}</span>
           <span className="text-xs text-muted-foreground/60 font-mono truncate max-w-[200px] md:max-w-md">{did.did}</span>
        </div>
      </button>
      {isOpen && (
        <div className="p-4 pt-0">
          <div className="bg-black/40 rounded p-4 font-mono text-[10px] text-muted-foreground/80 overflow-x-auto border border-white/5 whitespace-pre">
            {did.document}
          </div>
        </div>
      )}
    </div>
  );
};

export const Route = createFileRoute("/wallet/info")({
  component: WalletInfoPage,
});
