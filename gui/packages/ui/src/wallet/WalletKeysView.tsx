import React, { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Loader2, Plus, Trash2 } from "lucide-react";
import { toast } from "sonner";
import { customInstance } from "../lib/http";
import { PageSection } from "../layout/PageSection";
import { DataTable } from "../table/DataTable";
import { Identifier } from "../identity/Identifier";
import { FormatDate } from "../ui/format-date";
import { Skeleton } from "../ui/skeleton";
import { Badge } from "../ui/badge";
import { Button } from "../ui/button";
import { Input } from "../ui/input";
import { Label } from "../ui/label";
import { Textarea } from "../ui/textarea";
import { ConfirmDialog } from "../ui/confirm-dialog";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "../ui/dialog";

export interface WalletKeyItem {
  id: string;
  alias?: string | null;
  kty: string;
  crv?: string | null;
  created_at?: string | null;
}

interface DidKeysRef {
  alias: string;
  did: string;
  keys: { internal: string }[];
}

interface WalletInfoKeysResponse {
  status: number;
  data: { dids: DidKeysRef[] };
}

interface WalletKeysListResponse {
  status: number;
  data: WalletKeyItem[];
}

export const WALLET_KEYS_QUERY_KEY = ["/wallet/keys"] as const;

/** Private keys held by the wallet, the DIDs that use each one, and PEM import. */
export function WalletKeysView() {
  const queryClient = useQueryClient();
  const [deleting, setDeleting] = useState<WalletKeyItem | null>(null);

  const {
    data: response,
    isLoading,
    error,
  } = useQuery({
    queryKey: WALLET_KEYS_QUERY_KEY,
    queryFn: () => customInstance<WalletKeysListResponse>("/wallet/keys", { method: "GET" }),
  });

  const { data: info } = useQuery({
    queryKey: ["wallet-info-custom"],
    queryFn: () => customInstance<WalletInfoKeysResponse>("/wallet/info", { method: "GET" }),
  });

  const deleteKey = useMutation({
    mutationFn: ({ id }: { id: string }) =>
      customInstance(`/wallet/key/${encodeURIComponent(id)}`, { method: "DELETE" }),
  });

  const refresh = async () => {
    await queryClient.invalidateQueries({ queryKey: WALLET_KEYS_QUERY_KEY });
    await queryClient.invalidateQueries({ queryKey: ["wallet-keys-list"] });
  };

  const dids = info?.status === 200 ? info.data.dids : [];
  const usedBy = (key: WalletKeyItem) =>
    dids.filter((d) => d.keys.some((k) => k.internal === key.id));

  if (isLoading) {
    return (
      <PageSection title="Keys">
        <Skeleton className="h-64 w-full" />
      </PageSection>
    );
  }

  if (error || response?.status !== 200) {
    return <p className="text-sm text-danger-600">Error loading keys.</p>;
  }

  return (
    <PageSection title="Keys" action={<ImportKeyDialog onImported={refresh} />}>
      <DataTable
        searchable={false}
        data={response.data}
        keyExtractor={(k) => k.id}
        emptyMessage="No keys in this wallet"
        emptyDescription="Import a PEM private key to create DIDs with it."
        columns={[
          {
            header: "Alias",
            accessorKey: "alias",
            cell: (k) => <span className="font-medium">{k.alias || "—"}</span>,
          },
          { header: "Key ID", cell: (k) => <Identifier value={k.id} /> },
          { header: "Type", cell: (k) => <Badge variant="info">{k.kty}</Badge> },
          {
            header: "Curve",
            cell: (k) => <span className="font-mono text-xs">{k.crv ?? "—"}</span>,
          },
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
          { header: "Created", cell: (k) => <FormatDate date={k.created_at ?? null} /> },
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
}

const ImportKeyDialog = ({ onImported }: { onImported: () => Promise<void> }) => {
  const [open, setOpen] = useState(false);
  const [pem, setPem] = useState("");
  const [alias, setAlias] = useState("");

  const register = useMutation({
    mutationFn: (data: { pem: string; alias: string | null }) =>
      customInstance<{ status: number }>("/wallet/key", { method: "POST", data }),
  });

  const looksLikePem = /-----BEGIN [A-Z ]+-----[\s\S]+-----END [A-Z ]+-----/.test(pem.trim());

  const submit = async () => {
    try {
      const res = await register.mutateAsync({ pem, alias: alias || null });
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
          <DialogDescription>
            Paste a PEM private key. Type and curve are read from it.
          </DialogDescription>
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
