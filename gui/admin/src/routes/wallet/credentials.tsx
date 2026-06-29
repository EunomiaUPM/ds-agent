import { createFileRoute, Link } from "@tanstack/react-router";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { customInstance } from "shared/src/data/orval-mutator";
import { toast } from "sonner";
import { PageSection } from "shared/src/components/layout/PageSection";
import { Skeleton } from "shared/src/components/ui/skeleton";
import { Button } from "shared/src/components/ui/button";
import { Badge } from "shared/src/components/ui/badge";
import {
  ShieldAlert,
  ShieldCheck,
  ChevronDown,
  ChevronUp,
  FileJson,
  Fingerprint,
  Calendar,
  Building2,
  Loader2,
  Trash2,
  ArrowRight,
  Check,
  Sparkles,
} from "lucide-react";
import { useState } from "react";
import { cn } from "shared/src/lib/utils";

interface VcsResponse {
  status: number;
  data: any[];
}

const REGISTRATION_TYPES = [
  "gx:Eori",
  "gx:Euid",
  "gx:LeiCode",
  "gx:LocalRegistrationNumber",
  "gx:TaxId",
  "gx:VatId",
  "Eori",
  "TaxId",
  "VatId",
];
const LEGAL_PERSON_TYPES = ["gx:LegalPerson", "LegalPerson"];
const TC_TYPES = ["gx:TermsAndConditions", "TermsAndConditions"];
const LABEL_TYPES = ["gx:LabelCredential", "LabelCredential"];

const RECOMMENDED_REGISTRATION_VC = "gx_VatId_jwt_vc_json";
const RECOMMENDED_COMPLIANCE_VC = "gx_LabelCredential_jwt_vc_json";

function vcHasAny(vcs: any[], targets: string[]): boolean {
  return vcs.some((vc) => {
    const types: string[] = vc.parsed_document?.type || [];
    return types.some((t) => targets.includes(t));
  });
}

const WalletCredentials = () => {
  const queryClient = useQueryClient();
  const {
    data: response,
    isLoading,
    error,
  } = useQuery({
    queryKey: ["wallet-vcs-custom"],
    queryFn: () => customInstance<VcsResponse>("/wallet/vcs", { method: "GET" }),
  });

  const [isGenerating, setIsGenerating] = useState(false);

  const credentials = response?.status === 200 ? response.data : [];

  const hasRegistration = vcHasAny(credentials, REGISTRATION_TYPES);
  const hasLegalPerson = vcHasAny(credentials, LEGAL_PERSON_TYPES);
  const hasTermsAndConditions = vcHasAny(credentials, TC_TYPES);
  const hasLabel = vcHasAny(credentials, LABEL_TYPES);

  // Pipeline step:
  // 1) need registration VC from a legal authority
  // 2) auto-generate self-attested LegalPerson + TermsAndConditions
  // 3) request the compliance credential from a clearing house
  // 4) compliant (LabelCredential held)
  const step: 1 | 2 | 3 | 4 = hasLabel
    ? 4
    : hasRegistration && hasLegalPerson && hasTermsAndConditions
      ? 3
      : hasRegistration
        ? 2
        : 1;

  const handleGenerateGaia = async () => {
    setIsGenerating(true);
    try {
      await customInstance("/gaia/generate", { method: "POST" });
      toast.success("Gaia-X self-issued credentials generated");
      await queryClient.invalidateQueries({ queryKey: ["wallet-vcs-custom"] });
    } catch (err) {
      const msg = err instanceof Error ? err.message : "Failed to generate Gaia-X credentials";
      toast.error(msg);
    } finally {
      setIsGenerating(false);
    }
  };

  const handleDeleteVc = async (id: string) => {
    try {
      await customInstance(`/wallet/credential/${id}`, { method: "DELETE" });
      toast.success("Credential deleted");
      await queryClient.invalidateQueries({ queryKey: ["wallet-vcs-custom"] });
    } catch (err) {
      const msg = err instanceof Error ? err.message : "Failed to delete credential";
      toast.error(msg);
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
        <ComplianceStepCard
          step={step}
          isGenerating={isGenerating}
          onGenerate={handleGenerateGaia}
          flags={{ hasRegistration, hasLegalPerson, hasTermsAndConditions, hasLabel }}
        />
        <StepTrail step={step} />
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
  const parsed = vc.parsed_document || {};
  const issuerName = parsed.issuer?.name || parsed.issuer?.id || "Unknown Issuer";
  const types = (parsed.type || []).filter((t: string) => t !== "VerifiableCredential");
  const displayType = types.length > 0 ? types[types.length - 1] : "Credential";

  // Format dates. `added_on` and `valid_until` come from the entity; `validUntil` may
  // also live inside the parsed VC document depending on the issuer.
  const addedRaw = vc.added_on ?? vc.addedOn;
  const addedDate = addedRaw ? new Date(addedRaw).toLocaleDateString() : "N/A";
  const validRaw = vc.valid_until ?? parsed.validUntil ?? parsed.expirationDate;
  const validUntil = validRaw ? new Date(validRaw).toLocaleDateString() : "Never";

  const handleDelete = async (e: React.MouseEvent) => {
    e.stopPropagation();
    setIsDeleting(true);
    await onDelete(vc.id);
    setIsDeleting(false);
  };

  return (
    <div
      className={cn(
        "group relative overflow-hidden bg-white/[0.03] border border-white/10 rounded-2xl transition-all duration-300",
        isExpanded
          ? "border-primary/40 bg-white/[0.06] ring-1 ring-primary/20"
          : "hover:border-white/20 hover:bg-white/[0.05]",
      )}
    >
      {/* Card Header Area */}
      <div
        onClick={() => setIsExpanded(!isExpanded)}
        className="cursor-pointer p-6 flex flex-col md:flex-row md:items-center justify-between gap-4"
      >
        <div className="flex items-start gap-4">
          <div
            className={cn(
              "p-3 rounded-xl transition-colors duration-300",
              isExpanded
                ? "bg-primary/20 text-primary"
                : "bg-white/5 text-muted-foreground group-hover:bg-white/10",
            )}
          >
            <Fingerprint className="h-6 w-6" />
          </div>
          <div className="space-y-1">
            <div className="flex items-center gap-2">
              <h4 className="font-bold text-lg tracking-tight">{displayType}</h4>
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
              <span className="uppercase text-[9px] font-bold tracking-widest opacity-40">
                Added
              </span>
              <span className="flex items-center gap-1 mt-0.5">
                <Calendar className="h-3 w-3" />
                {addedDate}
              </span>
            </div>
            <div className="flex flex-col items-end">
              <span className="uppercase text-[9px] font-bold tracking-widest opacity-40">
                Expires
              </span>
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
      <div
        className={cn(
          "grid transition-all duration-500 ease-in-out",
          isExpanded ? "grid-rows-[1fr] border-t border-white/10" : "grid-rows-[0fr]",
        )}
      >
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
                  {isDeleting ? (
                    <Loader2 className="h-3 w-3 mr-1 animate-spin" />
                  ) : (
                    <Trash2 className="h-3 w-3 mr-1" />
                  )}
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

interface ComplianceFlags {
  hasRegistration: boolean;
  hasLegalPerson: boolean;
  hasTermsAndConditions: boolean;
  hasLabel: boolean;
}

const ComplianceStepCard = ({
  step,
  isGenerating,
  onGenerate,
  flags,
}: {
  step: 1 | 2 | 3 | 4;
  isGenerating: boolean;
  onGenerate: () => void;
  flags: ComplianceFlags;
}) => {
  const wrapper = cn(
    "relative overflow-hidden p-6 rounded-2xl border transition-all duration-300",
    step === 4
      ? "bg-green-500/10 border-green-500/20 shadow-lg"
      : step === 1
        ? "bg-white/2 border-white/5 opacity-90"
        : "bg-primary/5 border-primary/20 shadow-lg shadow-primary/5",
  );

  return (
    <div className={wrapper}>
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-6">
        <div className="space-y-2">
          <div className="flex items-center gap-2">
            <ShieldCheck
              className={cn(
                "h-5 w-5",
                step === 4 ? "text-green-500" : step === 1 ? "text-muted-foreground" : "text-primary",
              )}
            />
            <h3 className="font-semibold text-lg">{stepTitle(step)}</h3>
            <Badge variant="info" className="font-mono text-[10px]">
              Step {step} of 4
            </Badge>
          </div>
          <p className="text-sm text-muted-foreground max-w-xl">{stepDescription(step)}</p>
        </div>

        <StepCta step={step} isGenerating={isGenerating} onGenerate={onGenerate} />
      </div>

      {step === 1 && (
        <Hint
          tone="amber"
          text="REQUIREMENT: pick a credential type like VatId, TaxId or LeiCode from a legal authority."
        />
      )}
      {step === 2 && !flags.hasLegalPerson && !flags.hasTermsAndConditions && (
        <Hint
          tone="primary"
          text="The wallet will self-issue gx:LegalPerson and gx:TermsAndConditions on top of your registration credential."
        />
      )}
      {step === 3 && (
        <Hint
          tone="primary"
          text="Ask a clearing house to issue a LabelCredential signing your self-attested data."
        />
      )}
    </div>
  );
};

const StepCta = ({
  step,
  isGenerating,
  onGenerate,
}: {
  step: 1 | 2 | 3 | 4;
  isGenerating: boolean;
  onGenerate: () => void;
}) => {
  if (step === 1) {
    return (
      <Link to="/authority/new" search={{ recommended: RECOMMENDED_REGISTRATION_VC } as any}>
        <Button className="md:min-w-[220px]">
          Request VC
          <ArrowRight className="ml-2 h-4 w-4" />
        </Button>
      </Link>
    );
  }
  if (step === 2) {
    return (
      <Button
        disabled={isGenerating}
        onClick={onGenerate}
        className="md:min-w-[220px]"
      >
        {isGenerating ? (
          <>
            <Loader2 className="mr-2 h-4 w-4 animate-spin" />
            Generating...
          </>
        ) : (
          <>
            <Sparkles className="mr-2 h-4 w-4" />
            Auto-generate VCs
          </>
        )}
      </Button>
    );
  }
  if (step === 3) {
    return (
      <Link to="/authority/new" search={{ recommended: RECOMMENDED_COMPLIANCE_VC } as any}>
        <Button className="md:min-w-[220px]">
          Request Compliance
          <ArrowRight className="ml-2 h-4 w-4" />
        </Button>
      </Link>
    );
  }
  return (
    <Button variant="outline" disabled className="md:min-w-[220px]">
      <Check className="mr-2 h-4 w-4 text-green-500" />
      Compliance Active
    </Button>
  );
};

function stepTitle(step: 1 | 2 | 3 | 4) {
  switch (step) {
    case 1:
      return "Get your legal identity credential";
    case 2:
      return "Self-issue Gaia-X base credentials";
    case 3:
      return "Request the compliance credential";
    case 4:
      return "You are Gaia-X compliant";
  }
}

function stepDescription(step: 1 | 2 | 3 | 4) {
  switch (step) {
    case 1:
      return "Pick a registration credential (VatId, TaxId, LEI, EORI, ...) from a trusted authority. This anchors your legal identity.";
    case 2:
      return "Generate the two self-issued credentials (LegalPerson and TermsAndConditions) that Gaia-X requires on top of your registration data.";
    case 3:
      return "With registration + self-issued credentials in hand, a clearing house can now sign your LabelCredential.";
    case 4:
      return "Your wallet holds a valid LabelCredential. You can participate in Gaia-X workflows.";
  }
}

const Hint = ({ tone, text }: { tone: "amber" | "primary"; text: string }) => {
  const cls =
    tone === "amber"
      ? "bg-amber-500/10 border-amber-500/20 text-amber-500"
      : "bg-primary/10 border-primary/20 text-primary";
  return (
    <div className={cn("mt-4 p-3 rounded-lg text-[10px] flex items-center gap-2 font-mono border", cls)}>
      <ShieldAlert className="h-3 w-3" />
      {text}
    </div>
  );
};

const StepTrail = ({ step }: { step: 1 | 2 | 3 | 4 }) => {
  const items: { n: 1 | 2 | 3 | 4; label: string }[] = [
    { n: 1, label: "Registration" },
    { n: 2, label: "Self-issued" },
    { n: 3, label: "Compliance" },
    { n: 4, label: "Compliant" },
  ];
  return (
    <div className="mt-4 flex items-center gap-2">
      {items.map((it, idx) => {
        const done = it.n < step;
        const current = it.n === step;
        return (
          <div key={it.n} className="flex items-center gap-2">
            <div
              className={cn(
                "h-6 w-6 rounded-full flex items-center justify-center text-[10px] font-bold border",
                done
                  ? "bg-green-500/20 border-green-500/40 text-green-500"
                  : current
                    ? "bg-primary/20 border-primary/40 text-primary"
                    : "bg-white/5 border-white/10 text-muted-foreground",
              )}
            >
              {done ? <Check className="h-3 w-3" /> : it.n}
            </div>
            <span
              className={cn(
                "text-[10px] uppercase tracking-widest",
                current ? "text-primary font-bold" : "text-muted-foreground",
              )}
            >
              {it.label}
            </span>
            {idx < items.length - 1 && (
              <div className="h-px w-6 bg-white/10" aria-hidden />
            )}
          </div>
        );
      })}
    </div>
  );
};

export const Route = createFileRoute("/wallet/credentials")({
  component: WalletCredentials,
});
