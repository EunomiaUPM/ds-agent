import { createFileRoute } from "@tanstack/react-router";
import { useQuery } from "@tanstack/react-query";
import { customInstance } from "shared/src/data/orval-mutator";
import { PageSection } from "shared/src/components/layout/PageSection";
import { DataTable } from "shared/src/components/DataTable";
import { Skeleton } from "shared/src/components/ui/skeleton";
import { Badge } from "shared/src/components/ui/badge";
import { InfoList } from "shared/src/components/ui/info-list";
import { ChevronRight, ChevronDown, FileJson, Copy, Check } from "lucide-react";
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
  };
}

const WalletInfoPage = () => {
  const {
    data: response,
    isLoading,
    error,
  } = useQuery({
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
              cell: (d) => <span className="font-semibold text-primary/80">{d.alias}</span>,
            },
            {
              header: "DID",
              accessorKey: "did",
              cell: (d) => (
                <span className="font-mono text-[10px] text-muted-foreground break-all">
                  {d.did}
                </span>
              ),
            },
            {
              header: "Default",
              accessorKey: "default",
              cell: (d) => (
                <Badge variant={d.default ? "default" : "info"}>
                  {d.default ? "PRIMARY" : "SECONDARY"}
                </Badge>
              ),
            },
            {
              header: "Created",
              accessorKey: "createdOn",
            },
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
  const [copied, setCopied] = useState(false);

  let formattedDoc = did.document;
  try {
    const parsed = typeof did.document === "string" ? JSON.parse(did.document) : did.document;
    formattedDoc = JSON.stringify(parsed, null, 2);
  } catch (e) {
    // leave as is if not JSON
  }

  const handleCopy = (e: React.MouseEvent) => {
    e.stopPropagation();
    navigator.clipboard.writeText(formattedDoc);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className="group border border-white/10 rounded-xl overflow-hidden bg-white/[0.02] transition-all hover:bg-white/[0.04]">
      <button
        onClick={() => setIsOpen(!isOpen)}
        className="w-full flex items-center justify-between p-4 text-left transition-colors"
      >
        <div className="flex items-center gap-3">
          <div
            className={cn(
              "transition-transform duration-200",
              isOpen ? "rotate-90 text-primary" : "text-muted-foreground",
            )}
          >
            <ChevronRight className="h-5 w-5" />
          </div>
          <span className="text-sm font-semibold">{did.alias}</span>
          <span className="text-xs text-muted-foreground/60 font-mono truncate max-w-[200px] md:max-w-md">
            {did.did}
          </span>
        </div>
      </button>
      {isOpen && (
        <div className="p-4 pt-0">
          <div className="bg-black/40 rounded-xl border border-white/5 overflow-hidden">
            <div className="flex items-center justify-between px-4 py-2 border-b border-white/5 bg-white/[0.02]">
              <span className="text-[10px] font-bold uppercase tracking-widest text-primary/60 flex items-center gap-2">
                <FileJson className="h-3 w-3" />
                JSON Document
              </span>
              <button
                onClick={handleCopy}
                className="flex items-center gap-1.5 text-[10px] font-mono text-muted-foreground hover:text-primary transition-colors"
              >
                {copied ? (
                  <Check className="h-3 w-3 text-green-500" />
                ) : (
                  <Copy className="h-3 w-3" />
                )}
                {copied ? "Copied!" : "Copy"}
              </button>
            </div>
            <div className="p-4 overflow-x-auto">
              <pre className="font-mono text-[11px] text-muted-foreground/90 whitespace-pre-wrap break-all leading-relaxed">
                {formattedDoc}
              </pre>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

export const Route = createFileRoute("/wallet/info")({
  component: WalletInfoPage,
});
