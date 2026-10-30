import React, { useState } from "react";
import { useMutation } from "@tanstack/react-query";
import { Loader2, ShieldCheck } from "lucide-react";
import { toast } from "sonner";
import { customInstance } from "../lib/http";
import { PageSection } from "../layout/PageSection";
import { Input } from "../ui/input";
import { Button } from "../ui/button";
import { Label } from "../ui/label";

export function WalletOidc4vpView() {
  const [uri, setUri] = useState("");

  const {
    mutateAsync: processVp,
    isPending,
    isSuccess,
    error,
  } = useMutation({
    mutationFn: (data: { uri: string }) =>
      customInstance("/wallet/oid4vp", { method: "POST", data }),
    onSuccess: () => toast.success("Presentation submitted successfully"),
    onError: (err) => {
      const msg = err instanceof Error ? err.message : "Failed to process presentation request";
      toast.error(msg);
    },
  });

  const handleProcess = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!uri) return;
    try {
      await processVp({ uri });
      setUri("");
    } catch {
      // already surfaced via onError toast
    }
  };

  return (
    <PageSection title="Present credentials (OID4VP)">
      <form
        onSubmit={handleProcess}
        className="max-w-2xl space-y-4 rounded-lg border border-ink/10 p-5"
      >
        <div className="flex items-start gap-3">
          <ShieldCheck className="mt-0.5 h-5 w-5 shrink-0 text-ink/40" />
          <p className="text-sm text-ink/70">
            Paste the OID4VP request URI a verifier gave you to present credentials from this
            wallet.
          </p>
        </div>
        <div className="space-y-1.5">
          <Label htmlFor="oidc4vp-uri">Request URI</Label>
          <Input
            id="oidc4vp-uri"
            placeholder="openid-vc://…"
            value={uri}
            onChange={(e) => setUri(e.target.value)}
            className="font-mono text-xs"
          />
        </div>
        <div className="flex items-center gap-3">
          <Button type="submit" disabled={isPending || !uri}>
            {isPending && <Loader2 className="animate-spin" />}
            Present credentials
          </Button>
          {isSuccess && (
            <span className="text-sm text-success-700 dark:text-success-300">
              Presentation sent to the verifier.
            </span>
          )}
          {!!error && (
            <span className="text-sm text-danger-700 dark:text-danger-300">
              The presentation request could not be processed.
            </span>
          )}
        </div>
      </form>
    </PageSection>
  );
}
