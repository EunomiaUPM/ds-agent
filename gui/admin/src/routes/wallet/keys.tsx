import { createFileRoute } from "@tanstack/react-router";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { customInstance } from "shared/src/data/orval-mutator";
import { PageSection } from "shared/src/components/layout/PageSection";
import { Skeleton } from "shared/src/components/ui/skeleton";
import { Button } from "shared/src/components/ui/button";
import { Input } from "shared/src/components/ui/input";
import { Label } from "shared/src/components/ui/label";
import { Badge } from "shared/src/components/ui/badge";
import { DataTable } from "shared/src/components/DataTable";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "shared/src/components/ui/dialog";
import { useState } from "react";
import { toast } from "sonner";
import { Key, Loader2, Plus, Trash2 } from "lucide-react";

interface KeyModel {
  id: string;
  alias: string;
  kty: any;
  crv: any;
  created_at: string;
}

interface KeysResponse {
  status: number;
  data: KeyModel[];
}

const KEYS_QUERY_KEY = ["wallet-keys-list"];

function reportError(err: unknown, fallback: string) {
  const msg = err instanceof Error ? err.message : fallback;
  toast.error(msg);
}

const WalletKeysPage = () => {
  const queryClient = useQueryClient();
  const { data: response, isLoading, error } = useQuery({
    queryKey: KEYS_QUERY_KEY,
    queryFn: () => customInstance<KeysResponse>("/wallet/keys", { method: "GET" }),
  });

  const invalidate = () => queryClient.invalidateQueries({ queryKey: KEYS_QUERY_KEY });

  const deleteKey = useMutation({
    mutationFn: (id: string) =>
      customInstance(`/wallet/key/${encodeURIComponent(id)}`, { method: "DELETE" }),
    onSuccess: async () => {
      toast.success("Key deleted");
      await invalidate();
    },
    onError: (err) => reportError(err, "Failed to delete key"),
  });

  if (isLoading) {
    return (
      <PageSection title="Keys">
        <Skeleton className="h-64 w-full rounded-2xl" />
      </PageSection>
    );
  }

  if (error || response?.status !== 200) {
    return <div className="text-destructive font-mono text-xs">Error loading keys</div>;
  }

  const keys = response.data;

  return (
    <div className="space-y-8 pb-20">
      <PageSection title="Keys" action={<NewKeyDialog onCreated={invalidate} />}>
        {keys.length === 0 ? (
          <div className="flex flex-col items-center justify-center py-20 text-muted-foreground border border-dashed border-white/10 rounded-2xl bg-white/2">
            <Key className="h-12 w-12 opacity-10 mb-4" />
            <p className="text-sm font-medium">No keys stored yet.</p>
            <p className="text-xs opacity-60">Import a PEM-encoded private key to get started.</p>
          </div>
        ) : (
          <DataTable
            data={keys}
            keyExtractor={(k) => k.id}
            columns={[
              {
                header: "Alias",
                accessorKey: "alias",
                cell: (k) => (
                  <span className="font-semibold text-primary/80">{k.alias || "—"}</span>
                ),
              },
              {
                header: "ID",
                accessorKey: "id",
                cell: (k) => (
                  <span className="font-mono text-[10px] text-muted-foreground break-all">
                    {k.id}
                  </span>
                ),
              },
              {
                header: "Type",
                cell: (k) => (
                  <Badge variant="info" className="font-mono">
                    {kindToString(k.kty)}
                  </Badge>
                ),
              },
              {
                header: "Curve",
                cell: (k) => (
                  <span className="font-mono text-xs text-muted-foreground">
                    {kindToString(k.crv) || "—"}
                  </span>
                ),
              },
              {
                header: "Created",
                accessorKey: "created_at",
                cell: (k) => (
                  <span className="text-xs text-muted-foreground">
                    {k.created_at ? new Date(k.created_at).toLocaleDateString() : "—"}
                  </span>
                ),
              },
              {
                header: "Actions",
                cell: (k) => (
                  <Button
                    size="sm"
                    variant="ghost"
                    disabled={deleteKey.isPending}
                    onClick={() => deleteKey.mutate(k.id)}
                  >
                    {deleteKey.isPending && deleteKey.variables === k.id ? (
                      <Loader2 className="h-3 w-3 animate-spin" />
                    ) : (
                      <Trash2 className="h-3 w-3 text-destructive" />
                    )}
                  </Button>
                ),
              },
            ]}
          />
        )}
      </PageSection>
    </div>
  );
};

/**
 * Kty/Crv may be serialized as either a bare string ("RSA"), an object with `Other(String)`,
 * or null. Normalize for display.
 */
function kindToString(value: any): string {
  if (value == null) return "";
  if (typeof value === "string") return value;
  if (typeof value === "object") {
    if ("Other" in value) return String(value.Other);
    const keys = Object.keys(value);
    return keys[0] ?? "";
  }
  return String(value);
}

const NewKeyDialog = ({ onCreated }: { onCreated: () => Promise<void> }) => {
  const [open, setOpen] = useState(false);
  const [pem, setPem] = useState("");
  const [alias, setAlias] = useState("");

  const create = useMutation({
    mutationFn: () =>
      customInstance("/wallet/key", {
        method: "POST",
        data: { pem, alias: alias || null },
      }),
    onSuccess: async () => {
      toast.success("Key imported");
      setOpen(false);
      setPem("");
      setAlias("");
      await onCreated();
    },
    onError: (err) => reportError(err, "Failed to import key"),
  });

  const canSubmit = pem.trim().includes("BEGIN") && pem.trim().includes("END");

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>
        <Button size="sm">
          <Plus className="h-3 w-3" />
          <span className="ml-1">Import Key</span>
        </Button>
      </DialogTrigger>
      <DialogContent className="max-w-2xl">
        <DialogHeader>
          <DialogTitle>Import a private key</DialogTitle>
          <DialogDescription>
            Paste a PEM-encoded private key. The wallet derives kty/crv from the PEM headers.
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-4">
          <div className="space-y-1">
            <Label className="text-xs">Alias (optional)</Label>
            <Input
              placeholder="my-signing-key"
              value={alias}
              onChange={(e) => setAlias(e.target.value)}
            />
          </div>
          <div className="space-y-1">
            <Label className="text-xs">PEM</Label>
            <textarea
              className="w-full font-mono text-[11px] bg-black/30 border border-white/10 rounded p-3 min-h-[200px] text-foreground"
              placeholder={"-----BEGIN PRIVATE KEY-----\n...\n-----END PRIVATE KEY-----"}
              value={pem}
              onChange={(e) => setPem(e.target.value)}
            />
            {!canSubmit && pem.trim().length > 0 && (
              <p className="text-[10px] text-amber-500">
                PEM must include BEGIN/END markers.
              </p>
            )}
          </div>
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={() => setOpen(false)}>
            Cancel
          </Button>
          <Button disabled={!canSubmit || create.isPending} onClick={() => create.mutate()}>
            {create.isPending ? (
              <Loader2 className="h-3 w-3 animate-spin mr-1" />
            ) : (
              <Plus className="h-3 w-3 mr-1" />
            )}
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
