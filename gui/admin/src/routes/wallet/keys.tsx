import { createFileRoute } from "@tanstack/react-router";
import { useState } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { Loader2, Plus, Trash2 } from "lucide-react";
import { toast } from "sonner";
import { customInstance } from "shared/src/data/orval-mutator";
import {
  getGetWalletKeysQueryKey,
  useDeleteWalletKey,
  useGetWalletKeys,
  useRegisterWalletKey,
} from "shared/src/data/orval/wallet/wallet";
import type { WalletKey } from "shared/src/data/orval/model";
import { PageSection } from "shared/src/components/layout/PageSection";
import { DataTable } from "shared/src/components/DataTable";
import { Identifier } from "shared/src/components/identity/Identifier";
import { FormatDate } from "shared/src/components/ui/format-date";
import { Skeleton } from "shared/src/components/ui/skeleton";
import { Badge } from "shared/src/components/ui/badge";
import { Button } from "shared/src/components/ui/button";
import { Input } from "shared/src/components/ui/input";
import { Label } from "shared/src/components/ui/label";
import { Textarea } from "shared/src/components/ui/textarea";
import { ConfirmDialog } from "shared/src/components/ui/confirm-dialog";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "shared/src/components/ui/dialog";

interface DidKeys {
  alias: string;
  did: string;
  keys: { internal: string }[];
}

interface WalletInfoResponse {
  status: number;
  data: { dids: DidKeys[] };
}

/** Private keys held by the wallet, the DIDs that use each one, and PEM import. */
const WalletKeysPage = () => {
  const queryClient = useQueryClient();
  const [deleting, setDeleting] = useState<WalletKey | null>(null);
  const { data: response, isLoading, error } = useGetWalletKeys();
  // Same query key as the Info and DID tabs, so the three share the cached wallet.
  const { data: info } = useQuery({
    queryKey: ["wallet-info-custom"],
    queryFn: () => customInstance<WalletInfoResponse>("/wallet/info", { method: "GET" }),
  });
  const deleteKey = useDeleteWalletKey();

  const refresh = () => queryClient.invalidateQueries({ queryKey: getGetWalletKeysQueryKey() });
  const dids = info?.status === 200 ? info.data.dids : [];
  const usedBy = (key: WalletKey) => dids.filter((d) => d.keys.some((k) => k.internal === key.id));

  if (isLoading) return <Skeleton className="h-64 w-full rounded-xl" />;
  if (error || response?.status !== 200) {
    return <div className="text-destructive font-mono text-xs">Error loading keys</div>;
  }

  return (
    <PageSection title="Keys" action={<ImportKeyDialog onImported={refresh} />}>
      <DataTable
        searchable={false}
        data={response.data}
        keyExtractor={(k) => k.id}
        emptyMessage="No keys yet. Import a PEM private key to sign with it."
        columns={[
          {
            header: "Alias",
            accessorKey: "alias",
            cell: (k) => <span className="font-medium">{k.alias || "—"}</span>,
          },
          { header: "Key ID", cell: (k) => <Identifier value={k.id} /> },
          { header: "Type", cell: (k) => <Badge variant="info">{k.kty}</Badge> },
          { header: "Curve", cell: (k) => <span className="font-mono text-xs">{k.crv ?? "—"}</span> },
          {
            header: "Used by",
            cell: (k) => {
              const users = usedBy(k);
              return users.length === 0 ? (
                <span className="text-ink/40 text-xs">No DID</span>
              ) : (
                <div className="flex flex-wrap gap-1">
                  {users.map((d) => (
                    <Badge key={d.did} variant="outline" title={d.did}>
                      {d.alias}
                    </Badge>
                  ))}
                </div>
              );
            },
          },
          { header: "Created", cell: (k) => <FormatDate date={k.created_at} /> },
          {
            header: "",
            cell: (k) => (
              <Button
                size="sm"
                variant="ghost"
                aria-label={`Delete key ${k.alias || k.id}`}
                disabled={usedBy(k).length > 0}
                title={usedBy(k).length > 0 ? "A DID still uses this key" : "Delete key"}
                onClick={() => setDeleting(k)}
              >
                <Trash2 className="h-3.5 w-3.5 text-danger-500" />
              </Button>
            ),
          },
        ]}
      />
      <ConfirmDialog
        open={deleting !== null}
        onOpenChange={(open) => !open && setDeleting(null)}
        title={`Delete key ${deleting?.alias || deleting?.id}?`}
        description="The private key is removed from the wallet and cannot be recovered."
        onConfirm={async () => {
          await deleteKey.mutateAsync({ id: deleting!.id });
          toast.success("Key deleted");
          await refresh();
        }}
      />
    </PageSection>
  );
};

/** Imports a PEM private key; the wallet derives kty and crv from it. */
const ImportKeyDialog = ({ onImported }: { onImported: () => Promise<void> }) => {
  const [open, setOpen] = useState(false);
  const [pem, setPem] = useState("");
  const [alias, setAlias] = useState("");
  const register = useRegisterWalletKey();
  const looksLikePem = /-----BEGIN [A-Z ]+-----[\s\S]+-----END [A-Z ]+-----/.test(pem.trim());

  const submit = async () => {
    try {
      const res = await register.mutateAsync({ data: { pem, alias: alias || null } });
      if (res.status >= 300) throw new Error("The wallet rejected the key");
      toast.success("Key imported");
      setOpen(false);
      setPem("");
      setAlias("");
      await onImported();
    } catch (err) {
      toast.error(err instanceof Error ? err.message : "Failed to import key");
    }
  };

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>
        <Button size="sm">
          <Plus />
          Import key
        </Button>
      </DialogTrigger>
      <DialogContent className="max-w-2xl">
        <DialogHeader>
          <DialogTitle>Import a private key</DialogTitle>
          <DialogDescription>Paste a PEM private key. Type and curve are read from it.</DialogDescription>
        </DialogHeader>
        <div className="space-y-4">
          <div className="space-y-1.5">
            <Label htmlFor="key-alias">Alias</Label>
            <Input
              id="key-alias"
              placeholder="e.g. signing-2026"
              value={alias}
              onChange={(e) => setAlias(e.target.value)}
            />
          </div>
          <div className="space-y-1.5">
            <Label htmlFor="key-pem">PEM</Label>
            <Textarea
              id="key-pem"
              className="min-h-[200px] font-mono text-xs"
              placeholder={"-----BEGIN PRIVATE KEY-----\n…\n-----END PRIVATE KEY-----"}
              value={pem}
              onChange={(e) => setPem(e.target.value)}
            />
            {pem.trim() && !looksLikePem && (
              <p className="text-xs text-warn-700 dark:text-warn-300">
                The PEM needs its BEGIN and END lines.
              </p>
            )}
          </div>
        </div>
        <DialogFooter>
          <Button variant="outline" onClick={() => setOpen(false)}>
            Cancel
          </Button>
          <Button disabled={!looksLikePem || register.isPending} onClick={submit}>
            {register.isPending ? <Loader2 className="animate-spin" /> : <Plus />}
            Import
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};

export const Route = createFileRoute("/wallet/keys")({
  component: WalletKeysPage,
});
