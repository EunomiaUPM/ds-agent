import { createFileRoute } from "@tanstack/react-router";
import { useGetWalletDid } from "shared/src/data/orval/wallet/wallet";
import { PageSection } from "shared/src/components/layout/PageSection";
import { Skeleton } from "shared/src/components/ui/skeleton";
import { Button } from "shared/src/components/ui/button";
import { Badge } from "shared/src/components/ui/badge";
import { useState } from "react";
import { cn } from "shared/src/lib/utils";
import {
  FileJson,
  Fingerprint,
  Key,
  Link2,
  Copy,
  Check,
  ChevronDown,
  ChevronUp,
} from "lucide-react";

const WalletDID = () => {
  const { data: response, isLoading, error } = useGetWalletDid();
  const [isJsonExpanded, setIsJsonExpanded] = useState(false);
  const [copiedId, setCopiedId] = useState(false);

  if (isLoading) {
    return (
      <PageSection title="DID Document">
        <Skeleton className="h-64 w-full rounded-2xl" />
      </PageSection>
    );
  }

  if (error || response?.status !== 200) {
    return <div className="text-destructive font-mono text-xs">Error loading DID document</div>;
  }

  const didDoc = response.data as any;

  const handleCopyId = () => {
    navigator.clipboard.writeText(didDoc.id || "");
    setCopiedId(true);
    setTimeout(() => setCopiedId(false), 2000);
  };

  return (
    <PageSection title="DID Document">
      <div className="bg-white/[0.03] border border-white/10 rounded-2xl overflow-hidden">
        {/* Header section with DID ID */}
        <div className="p-6 border-b border-white/10 flex flex-col md:flex-row md:items-start justify-between gap-6">
          <div className="flex items-start gap-4">
            <div className="p-3 rounded-xl bg-primary/20 text-primary">
              <Fingerprint className="h-8 w-8" />
            </div>
            <div className="space-y-2">
              <h3 className="font-semibold text-lg flex items-center gap-2">
                Decentralized Identifier
                <Badge variant="info" className="text-[10px] h-5 py-0">
                  Active
                </Badge>
              </h3>
              <div className="flex items-center gap-2 bg-black/30 px-3 py-2 rounded-lg border border-white/5">
                <span className="font-mono text-sm text-foreground/90 break-all">{didDoc.id}</span>
                <Button variant="ghost" size="icon" className="h-6 w-6 ml-2" onClick={handleCopyId}>
                  {copiedId ? (
                    <Check className="h-3 w-3 text-green-500" />
                  ) : (
                    <Copy className="h-3 w-3" />
                  )}
                </Button>
              </div>
            </div>
          </div>
        </div>

        {/* Content sections */}
        <div className="grid grid-cols-1 md:grid-cols-2 divide-y md:divide-y-0 md:divide-x divide-white/10">
          {/* Verification Methods (Keys) */}
          <div className="p-6 space-y-4">
            <h4 className="text-xs uppercase tracking-widest text-muted-foreground/60 font-bold flex items-center gap-2">
              <Key className="h-3.5 w-3.5" />
              Verification Methods
            </h4>
            {didDoc.verificationMethod && didDoc.verificationMethod.length > 0 ? (
              <div className="space-y-3">
                {didDoc.verificationMethod.map((vm: any, idx: number) => (
                  <div
                    key={vm.id || idx}
                    className="bg-white/5 rounded-lg p-3 border border-white/5 text-xs"
                  >
                    <div className="font-mono text-foreground/80 mb-1 truncate">{vm.id}</div>
                    <div className="flex justify-between items-center text-muted-foreground/60 mt-2">
                      <span>Type: {vm.type}</span>
                    </div>
                  </div>
                ))}
              </div>
            ) : (
              <div className="text-sm text-muted-foreground italic">
                No verification methods defined
              </div>
            )}
          </div>

          {/* Services */}
          <div className="p-6 space-y-4">
            <h4 className="text-xs uppercase tracking-widest text-muted-foreground/60 font-bold flex items-center gap-2">
              <Link2 className="h-3.5 w-3.5" />
              Services
            </h4>
            {didDoc.service && didDoc.service.length > 0 ? (
              <div className="space-y-3">
                {didDoc.service.map((svc: any, idx: number) => (
                  <div
                    key={svc.id || idx}
                    className="bg-white/5 rounded-lg p-3 border border-white/5 text-xs space-y-1"
                  >
                    <div className="font-semibold text-white/90 truncate">{svc.type}</div>
                    <div className="font-mono text-foreground/70 truncate break-all opacity-80">
                      {svc.serviceEndpoint}
                    </div>
                  </div>
                ))}
              </div>
            ) : (
              <div className="text-sm text-muted-foreground italic">No services defined</div>
            )}
          </div>
        </div>

        {/* Expandable JSON */}
        <div className="border-t border-white/10">
          <div
            onClick={() => setIsJsonExpanded(!isJsonExpanded)}
            className="cursor-pointer p-4 flex items-center justify-between hover:bg-white/5 transition-colors"
          >
            <span className="text-[10px] font-bold uppercase tracking-widest text-foreground/70 flex items-center gap-2">
              <FileJson className="h-3 w-3" />
              Raw DID Document
            </span>
            <div className="text-muted-foreground/40">
              {isJsonExpanded ? (
                <ChevronUp className="h-4 w-4" />
              ) : (
                <ChevronDown className="h-4 w-4" />
              )}
            </div>
          </div>

          <div
            className={cn(
              "grid transition-all duration-300 ease-in-out",
              isJsonExpanded ? "grid-rows-[1fr]" : "grid-rows-[0fr]",
            )}
          >
            <div className="overflow-hidden">
              <div className="p-4 bg-black/40 border-t border-white/5 shadow-inner">
                <pre className="text-[11px] font-mono text-muted-foreground/80 whitespace-pre-wrap break-all leading-relaxed">
                  {JSON.stringify(didDoc, null, 2)}
                </pre>
              </div>
            </div>
          </div>
        </div>
      </div>
    </PageSection>
  );
};

export const Route = createFileRoute("/wallet/did")({
  component: WalletDID,
});
