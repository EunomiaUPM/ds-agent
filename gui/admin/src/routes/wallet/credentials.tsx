import { createFileRoute } from "@tanstack/react-router";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { customInstance } from "shared/src/data/orval-mutator";
import { PageSection } from "shared/src/components/layout/PageSection";
import { Skeleton } from "shared/src/components/ui/skeleton";
import { Button } from "shared/src/components/ui/button";
import { Badge } from "shared/src/components/ui/badge";
import { ShieldAlert, ShieldCheck, ChevronDown, ChevronUp, FileJson, Fingerprint, Calendar, Building2, Loader2, Info, Trash2, Check } from "lucide-react";
import { useState } from "react";
import { cn } from "shared/src/lib/utils";

interface VcsResponse {
  status: number;
  data: any[];
}

const COMPLIANCE_TYPES = [
  "gx:Eori", 
  "gx:Euid", 
  "gx:LeiCode", 
  "gx:LocalRegistrationNumber", 
  "gx:TaxId", 
  "gx:VatId",
  "Eori",
  "TaxId",
  "VatId"
];

const WalletCredentials = () => {
  const queryClient = useQueryClient();
  const { data: response, isLoading, error } = useQuery({
    queryKey: ["wallet-vcs-custom"],
    queryFn: () => customInstance<VcsResponse>("/wallet/vcs", { method: "GET" }),
  });

  const [isGenerating, setIsGenerating] = useState(false);
  const [generateSuccess, setGenerateSuccess] = useState(false);

  const credentials = response?.status === 200 ? response.data : [];

  // Check if we have a registration number VC to enable Gaia-X generation
  const hasRegistrationVC = credentials.some((vc: any) => {
    const types = vc.parsedDocument?.type || [];
    return types.some((t: string) => COMPLIANCE_TYPES.includes(t));
  });

  const hasGaiaLabel = credentials.some((vc: any) => {
    const types = vc.parsedDocument?.type || [];
    return types.some((t: string) => t === "gx:LabelCredential" || t === "LabelCredential");
  });

  const handleGenerateGaia = async () => {
    setIsGenerating(true);
    setGenerateSuccess(false);
    try {
      await customInstance("/gaia/credential/generate", { method: "POST" });
      setGenerateSuccess(true);
      // Force an immediate refetch of the credentials list
      await queryClient.invalidateQueries({ queryKey: ["wallet-vcs-custom"] });
    } catch (err) {
      console.error("Error generating Gaia-X credentials:", err);
    } finally {
      setIsGenerating(false);
    }
  };

  const handleDeleteVc = async (id: string) => {
    try {
      await customInstance(`/wallet/credential/${id}`, { method: "DELETE" });
      await queryClient.invalidateQueries({ queryKey: ["wallet-vcs-custom"] });
    } catch (err) {
      console.error("Failed to delete VC:", err);
    }
  };

  if (isLoading) {
    return (
      <PageSection title="Verifiable Credentials">
        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
          <Skeleton className="h-48 w-full rounded-xl" />
          <Skeleton className="h-48 w-full rounded-xl" />
        </div>
      </PageSection>
    );
  }

  if (error) {
    return (
      <div className="flex flex-col items-center justify-center py-20 text-destructive font-mono text-xs">
        <ShieldAlert className="h-8 w-8 mb-2" />
        Error loading credentials
      </div>
    );
  }

  return (
    <div className="space-y-8 pb-20">
      {/* Gaia-X Compliance Section */}
      <PageSection title="Gaia-X Compliance">
        <div className={cn(
          "relative overflow-hidden p-6 rounded-2xl border transition-all duration-300",
          hasGaiaLabel ? "bg-green-500/10 border-green-500/20 shadow-lg" : 
          hasRegistrationVC 
            ? "bg-primary/5 border-primary/20 shadow-lg shadow-primary/5" 
            : "bg-white/2 border-white/5 opacity-80"
        )}>
          <div className="flex flex-col md:flex-row md:items-center justify-between gap-6">
            <div className="space-y-2">
              <div className="flex items-center gap-2">
                <ShieldCheck className={cn("h-5 w-5", hasGaiaLabel ? "text-green-500" : hasRegistrationVC ? "text-primary" : "text-muted-foreground")} />
                <h3 className="font-semibold text-lg">Gaia-X Framework Setup</h3>
              </div>
              <p className="text-sm text-muted-foreground max-w-xl">
                {hasGaiaLabel ? 
                  "Your wallet is already Gaia-X compliant. You possess a valid LabelCredential." :
                  "Generate the necessary Gaia-X compliant credentials based on your registration information. This will enable your participation in Gaia-X ecosystems."
                }
              </p>
              {generateSuccess && !hasGaiaLabel && (
                <div className="flex items-center gap-2 text-xs font-medium text-green-500 animate-in fade-in slide-in-from-left-2">
                   <Info className="h-3 w-3" />
                   Gaia-X credentials generated successfully. You can now request your LabelCredential.
                </div>
              )}
            </div>

            <Button 
              disabled={hasGaiaLabel || !hasRegistrationVC || isGenerating}
              onClick={handleGenerateGaia}
              className="md:min-w-[200px]"
              variant={hasGaiaLabel ? "outline" : hasRegistrationVC ? "default" : "secondary"}
            >
              {isGenerating ? (
                <>
                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                  Generating...
                </>
              ) : hasGaiaLabel ? (
                <>
                  <Check className="mr-2 h-4 w-4 text-green-500" />
                  Compliance Active
                </>
              ) : (
                "Setup Gaia-X Compliance"
              )}
            </Button>
          </div>
          
          {!hasRegistrationVC && !hasGaiaLabel && (
            <div className="mt-4 p-3 rounded-lg bg-amber-500/10 border border-amber-500/20 text-[10px] text-amber-500 flex items-center gap-2 font-mono">
              <ShieldAlert className="h-3 w-3" />
              REQUIREMENT: You need an EORI, TaxID, or VAT registration credential to enable this action.
            </div>
          )}
        </div>
      </PageSection>

      {/* Credentials List */}
      <PageSection title="Verifiable Credentials">
        {credentials.length === 0 ? (
          <div className="flex flex-col items-center justify-center py-20 text-muted-foreground border border-dashed border-white/10 rounded-2xl bg-white/2">
             <ShieldAlert className="h-12 w-12 opacity-10 mb-4" />
             <p className="text-sm font-medium">No credentials found in this wallet.</p>
             <p className="text-xs opacity-60">Your claimed credentials will appear here.</p>
          </div>
        ) : (
          <div className="grid grid-cols-1 gap-6">
            {credentials.map((vc: any, idx: number) => (
              <CredentialCard key={vc.id || idx} vc={vc} onDelete={handleDeleteVc} />
            ))}
          </div>
        )}
      </PageSection>
    </div>
  );
};

const CredentialCard = ({ vc, onDelete }: { vc: any; onDelete: (id: string) => Promise<void> }) => {
  const [isExpanded, setIsExpanded] = useState(false);
  const [isDeleting, setIsDeleting] = useState(false);
  const parsed = vc.parsedDocument || {};
  const issuerName = parsed.issuer?.name || parsed.issuer?.id || "Unknown Issuer";
  const types = (parsed.type || []).filter((t: string) => t !== "VerifiableCredential");
  const displayType = types.length > 0 ? types[types.length - 1] : "Credential";
  
  // Format dates
  const addedDate = vc.addedOn ? new Date(vc.addedOn).toLocaleDateString() : "N/A";
  const validUntil = parsed.validUntil ? new Date(parsed.validUntil).toLocaleDateString() : "Never";

  const handleDelete = async (e: React.MouseEvent) => {
    e.stopPropagation();
    setIsDeleting(true);
    await onDelete(vc.id);
    setIsDeleting(false);
  };

  return (
    <div className={cn(
      "group relative overflow-hidden bg-white/[0.03] border border-white/10 rounded-2xl transition-all duration-300",
      isExpanded ? "border-primary/40 bg-white/[0.06] ring-1 ring-primary/20" : "hover:border-white/20 hover:bg-white/[0.05]"
    )}>
      {/* Card Header Area */}
      <div 
        onClick={() => setIsExpanded(!isExpanded)}
        className="cursor-pointer p-6 flex flex-col md:flex-row md:items-center justify-between gap-4"
      >
        <div className="flex items-start gap-4">
          <div className={cn(
            "p-3 rounded-xl transition-colors duration-300",
            isExpanded ? "bg-primary/20 text-primary" : "bg-white/5 text-muted-foreground group-hover:bg-white/10"
          )}>
            <Fingerprint className="h-6 w-6" />
          </div>
          <div className="space-y-1">
            <div className="flex items-center gap-2">
              <h4 className="font-bold text-lg tracking-tight">
                {displayType}
              </h4>
              {types.length > 1 && (
                <Badge variant="info" className="text-[10px] h-4 py-0 font-mono opacity-60">
                  +{types.length - 1} more
                </Badge>
              )}
            </div>
            <div className="flex flex-wrap items-center gap-y-1 gap-x-4 text-sm text-muted-foreground/80">
              <span className="flex items-center gap-1.5">
                <Building2 className="h-3.5 w-3.5 opacity-60" />
                {issuerName}
              </span>
              <span className="flex items-center gap-1.5 font-mono text-[11px] opacity-60">
                ID: {vc.id?.substring(0, 16)}...
              </span>
            </div>
          </div>
        </div>

        <div className="flex items-center justify-between md:justify-end gap-6 border-t md:border-t-0 pt-4 md:pt-0 border-white/5">
          <div className="flex items-center gap-4 text-xs text-muted-foreground/60">
            <div className="flex flex-col items-end">
              <span className="uppercase text-[9px] font-bold tracking-widest opacity-40">Added</span>
              <span className="flex items-center gap-1 mt-0.5">
                <Calendar className="h-3 w-3" />
                {addedDate}
              </span>
            </div>
            <div className="flex flex-col items-end">
              <span className="uppercase text-[9px] font-bold tracking-widest opacity-40">Expires</span>
              <span className="flex items-center gap-1 mt-0.5">
                <ShieldCheck className="h-3 w-3" />
                {validUntil}
              </span>
            </div>
          </div>
          <div className="text-muted-foreground/40 group-hover:text-primary/60 transition-colors">
            {isExpanded ? <ChevronUp className="h-5 w-5" /> : <ChevronDown className="h-5 w-5" />}
          </div>
        </div>
      </div>

      {/* Card Content Area (JSON) */}
      <div className={cn(
        "grid transition-all duration-500 ease-in-out",
        isExpanded ? "grid-rows-[1fr] border-t border-white/10" : "grid-rows-[0fr]"
      )}>
        <div className="overflow-hidden">
          <div className="p-6 space-y-4">
            <div className="flex items-center justify-between">
              <span className="text-[10px] font-bold uppercase tracking-widest text-primary/60 flex items-center gap-2">
                <FileJson className="h-3 w-3" />
                Raw Credential Document
              </span>
              <div className="flex items-center gap-2">
                <Button 
                  variant="ghost" 
                  size="sm" 
                  className="h-7 text-[10px] font-mono hover:bg-destructive/10 hover:text-destructive"
                  onClick={handleDelete}
                  disabled={isDeleting}
                >
                  {isDeleting ? <Loader2 className="h-3 w-3 mr-1 animate-spin" /> : <Trash2 className="h-3 w-3 mr-1" />}
                  Delete
                </Button>
                <Button 
                  variant="ghost" 
                  size="sm" 
                  className="h-7 text-[10px] font-mono hover:bg-primary/10 hover:text-primary"
                  onClick={(e) => {
                    e.stopPropagation();
                    navigator.clipboard.writeText(JSON.stringify(vc, null, 2));
                  }}
                >
                  Copy JSON
                </Button>
              </div>
            </div>
            <div className="relative rounded-xl bg-black/40 border border-white/5 p-4 font-mono text-[11px] leading-relaxed text-muted-foreground/90 overflow-x-auto shadow-inner">
              <pre className="whitespace-pre-wrap break-all">{JSON.stringify(vc, null, 2)}</pre>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

export const Route = createFileRoute("/wallet/credentials")({
  component: WalletCredentials,
});
