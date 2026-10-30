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
      customInstance("/wallet/oid4vci", { method: "POST", data }),
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
    <PageSection title="Receive a credential (OID4VCI)">
      <form onSubmit={handleProcess} className="max-w-2xl space-y-4 rounded-lg border border-ink/10 p-5">
        <div className="flex items-start gap-3">
          <FileText className="mt-0.5 h-5 w-5 shrink-0 text-ink/40" />
          <p className="text-sm text-ink/70">Paste the credential offer URI an issuer gave you to add the credential to this wallet.</p>
        </div>
        <div className="space-y-1.5">
          <Label htmlFor="oidc4vci-uri">Credential offer URI</Label>
          <Input
            id="oidc4vci-uri"
            placeholder="openid-credential-offer://…"
            value={uri}
            onChange={(e) => setUri(e.target.value)}
            className="font-mono text-xs"
          />
        </div>
        <div className="flex items-center gap-3">
          <Button type="submit" disabled={isPending || !uri}>
            {isPending && <Loader2 className="animate-spin" />}
            Request credential
          </Button>
          {isSuccess && <span className="text-sm text-success-700 dark:text-success-300">Credential added to the wallet.</span>}
          {!!error && <span className="text-sm text-danger-700 dark:text-danger-300">The credential offer could not be processed.</span>}
        </div>
      </form>
    </PageSection>
  );
};

export const Route = createFileRoute("/wallet/oidc4vci")({
  component: Oidc4vciPage,
});
