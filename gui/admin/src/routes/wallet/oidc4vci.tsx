import { createFileRoute } from "@tanstack/react-router";
import { useState } from "react";
import { useMutation } from "@tanstack/react-query";
import { customInstance } from "shared/src/data/orval-mutator";
import { toast } from "sonner";
import { PageSection } from "shared/src/components/layout/PageSection";
import { Input } from "shared/src/components/ui/input";
import { Button } from "shared/src/components/ui/button";
import { Label } from "shared/src/components/ui/label";
import { FileText, Loader2 } from "lucide-react";

/**
 * Route for processing OIDC4VCI credential offers.
 */
const Oidc4vciPage = () => {
  const [uri, setUri] = useState("");

  const {
    mutateAsync: processVci,
    isPending,
    isSuccess,
    error,
  } = useMutation({
    mutationFn: (data: { uri: string }) =>
      customInstance("/wallet/oidc4vci", { method: "POST", data }),
    onSuccess: () => toast.success("Credential issued successfully"),
    onError: (err) => {
      const msg = err instanceof Error ? err.message : "Failed to process credential offer";
      toast.error(msg);
    },
  });

  const handleProcess = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!uri) return;
    try {
      await processVci({ uri });
      setUri("");
    } catch {
      // already surfaced via onError toast
    }
  };

  return (
    <div className="max-w-2xl mx-auto py-8">
      <PageSection title="Credential Issuance (OIDC4VCI)">
        <div className="bg-white/5 border border-white/10 rounded-2xl p-10 backdrop-blur-md shadow-2xl space-y-8">
          <div className="text-center space-y-2">
            <FileText className="h-12 w-12 text-primary mx-auto mb-2" />
            <p className="text-sm text-muted-foreground">
              Paste a Credential Offer URI to receive new verifiable credentials.
            </p>
          </div>

          <form onSubmit={handleProcess} className="space-y-6">
            <div className="space-y-2">
              <Label
                htmlFor="oidc4vci-uri"
                className="text-xs uppercase tracking-widest text-muted-foreground/60 font-bold"
              >
                Credential Offer URI
              </Label>
              <Input
                id="oidc4vci-uri"
                placeholder="openid-credential-offer://..."
                value={uri}
                onChange={(e) => setUri(e.target.value)}
                className="bg-black/30 border-white/10 h-12 focus:ring-primary/50 text-sm font-mono"
              />
            </div>

            <Button
              type="submit"
              disabled={isPending || !uri}
              className="w-full h-12 text-base font-bold shadow-lg shadow-primary/10 transition-all active:scale-[0.98]"
            >
              {isPending ? (
                <span className="flex items-center gap-2">
                  <Loader2 className="h-4 w-4 animate-spin" />
                  Issuing...
                </span>
              ) : (
                "Request Credential"
              )}
            </Button>
          </form>

          {isSuccess && (
            <div className="p-4 rounded-lg bg-green-500/10 border border-green-500/20 text-green-400 text-sm text-center font-medium animate-in fade-in zoom-in duration-300">
              ✓ Credential issuance completed successfully
            </div>
          )}

          {!!error && (
            <div className="p-4 rounded-lg bg-destructive/10 border border-destructive/20 text-destructive text-sm text-center font-mono">
              Error: Failed to process credential offer
            </div>
          )}
        </div>
      </PageSection>
    </div>
  );
};

export const Route = createFileRoute("/wallet/oidc4vci")({
  component: Oidc4vciPage,
});
