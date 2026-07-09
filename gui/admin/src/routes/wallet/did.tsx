import { createFileRoute } from "@tanstack/react-router";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { customInstance } from "shared/src/data/orval-mutator";
import { PageSection } from "shared/src/components/layout/PageSection";
import { Skeleton } from "shared/src/components/ui/skeleton";
import { Button } from "shared/src/components/ui/button";
import { Badge } from "shared/src/components/ui/badge";
import { Input } from "shared/src/components/ui/input";
import { Label } from "shared/src/components/ui/label";
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
import { cn } from "shared/src/lib/utils";
import { toast } from "sonner";
import {
  Check,
  ChevronRight,
  Copy,
  FileJson,
  Key,
  Loader2,
  Plus,
  Star,
  Trash2,
} from "lucide-react";

interface KeyRef {
  internal: string;
  fragment: string;
}

interface KeyModel {
  id: string;
  alias: string;
  kty: any;
  crv: any;
  created_at: string;
}

interface DidModel {
  id: string;
  did: string;
  alias: string;
  default: boolean;
  type: string;
  keys: KeyRef[];
  default_key: KeyRef;
  did_document: any;
  service?: any[] | null;
}

interface WalletInfo {
  dids: DidModel[];
}

interface WalletInfoResponse {
  status: number;
  data: WalletInfo;
}

interface KeysResponse {
  status: number;
  data: KeyModel[];
}

const INFO_QUERY_KEY = ["wallet-info-custom"];
const KEYS_QUERY_KEY = ["wallet-keys-list"];

function reportError(err: unknown, fallback: string) {
  const msg = err instanceof Error ? err.message : fallback;
  toast.error(msg);
}

const WalletDIDPage = () => {
  const queryClient = useQueryClient();
  const {
    data: response,
    isLoading,
    error,
  } = useQuery({
    queryKey: INFO_QUERY_KEY,
    queryFn: () => customInstance<WalletInfoResponse>("/wallet/info", { method: "GET" }),
  });

  const invalidate = () => queryClient.invalidateQueries({ queryKey: INFO_QUERY_KEY });

  const setDefaultDid = useMutation({
    mutationFn: (id: string) =>
      customInstance(`/wallet/did/${encodeURIComponent(id)}/default`, { method: "POST" }),
    onSuccess: async () => {
      toast.success("Default DID updated");
      await invalidate();
    },
    onError: (err) => reportError(err, "Failed to set default DID"),
  });

  if (isLoading) {
    return (
      <PageSection title="DIDs">
        <Skeleton className="h-64 w-full rounded-2xl" />
      </PageSection>
    );
  }

  if (error || response?.status !== 200) {
    return <div className="text-destructive font-mono text-xs">Error loading DIDs</div>;
  }

  const dids = response.data.dids;

  return (
    <div className="space-y-8 pb-20">
      <PageSection
        title="DIDs"
        action={<NewDidDialog onCreated={invalidate} />}
      >
        {dids.length === 0 ? (
          <div className="flex flex-col items-center justify-center py-20 text-muted-foreground border border-dashed border-white/10 rounded-2xl bg-white/2">
            <Key className="h-12 w-12 opacity-10 mb-4" />
            <p className="text-sm font-medium">No DIDs registered yet.</p>
          </div>
        ) : (
          <div className="space-y-4">
            {dids.map((d) => (
              <DidPanel
                key={d.did}
                did={d}
                onSetDefault={() => setDefaultDid.mutate(d.id)}
                isSettingDefault={
                  setDefaultDid.isPending && setDefaultDid.variables === d.id
                }
                onChanged={invalidate}
              />
            ))}
          </div>
        )}
      </PageSection>
    </div>
  );
};

const DidPanel = ({
  did,
  onSetDefault,
  isSettingDefault,
  onChanged,
}: {
  did: DidModel;
  onSetDefault: () => void;
  isSettingDefault: boolean;
  onChanged: () => Promise<void>;
}) => {
  const [isOpen, setIsOpen] = useState(false);
  const [newKeyId, setNewKeyId] = useState("");
  const [copied, setCopied] = useState(false);

  const isWeb = did.type === "Web";

  const addKey = useMutation({
    mutationFn: (keyId: string) =>
      customInstance(
        `/wallet/did/${encodeURIComponent(did.id)}/key/${encodeURIComponent(keyId)}`,
        { method: "POST" },
      ),
    onSuccess: async () => {
      toast.success("Key added to DID");
      setNewKeyId("");
      await onChanged();
    },
    onError: (err) => reportError(err, "Failed to add key"),
  });

  const removeKey = useMutation({
    mutationFn: (keyId: string) =>
      customInstance(
        `/wallet/did/${encodeURIComponent(did.id)}/key/${encodeURIComponent(keyId)}`,
        { method: "DELETE" },
      ),
    onSuccess: async () => {
      toast.success("Key removed from DID");
      await onChanged();
    },
    onError: (err) => reportError(err, "Failed to remove key"),
  });

  const setDefaultKey = useMutation({
    mutationFn: (keyId: string) =>
      customInstance(
        `/wallet/did/${encodeURIComponent(did.id)}/key/default/${encodeURIComponent(keyId)}`,
        { method: "POST" },
      ),
    onSuccess: async () => {
      toast.success("Default key updated");
      await onChanged();
    },
    onError: (err) => reportError(err, "Failed to set default key"),
  });

  const deleteDid = useMutation({
    mutationFn: () =>
      customInstance(`/wallet/did/${encodeURIComponent(did.id)}`, { method: "DELETE" }),
    onSuccess: async () => {
      toast.success("DID deleted");
      await onChanged();
    },
    onError: (err) => reportError(err, "Failed to delete DID"),
  });

  const formattedDoc = JSON.stringify(did.did_document, null, 2);

  const handleCopy = (e: React.MouseEvent) => {
    e.stopPropagation();
    navigator.clipboard.writeText(formattedDoc);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className="group border border-white/10 rounded-xl overflow-hidden bg-white/[0.02] transition-all hover:bg-white/[0.04]">
      <div className="w-full flex items-center justify-between p-4 gap-3">
        <button
          onClick={() => setIsOpen(!isOpen)}
          className="flex items-center gap-3 flex-1 min-w-0 text-left"
        >
          <div
            className={cn(
              "transition-transform duration-200 shrink-0",
              isOpen ? "rotate-90 text-primary" : "text-muted-foreground",
            )}
          >
            <ChevronRight className="h-5 w-5" />
          </div>
          <span className="text-sm font-semibold">{did.alias}</span>
          {did.default && <Badge variant="default">PRIMARY</Badge>}
          <Badge variant="info" className="font-mono text-[10px]">
            {did.type}
          </Badge>
          <span
            className="text-xs text-muted-foreground/60 font-mono truncate max-w-[320px]"
            title={did.did}
          >
            {did.did}
          </span>
        </button>

        <div className="flex items-center gap-2 shrink-0">
          {!did.default && (
            <Button
              size="sm"
              variant="outline"
              disabled={isSettingDefault}
              onClick={onSetDefault}
            >
              {isSettingDefault ? (
                <Loader2 className="h-3 w-3 animate-spin" />
              ) : (
                <Star className="h-3 w-3" />
              )}
              <span className="ml-1">Set default</span>
            </Button>
          )}
          <Button
            size="sm"
            variant="ghost"
            disabled={did.default || deleteDid.isPending}
            onClick={() => deleteDid.mutate()}
            title={
              did.default
                ? "Cannot delete the active default DID"
                : "Delete this DID"
            }
          >
            {deleteDid.isPending ? (
              <Loader2 className="h-3 w-3 animate-spin" />
            ) : (
              <Trash2 className="h-3 w-3 text-destructive" />
            )}
          </Button>
        </div>
      </div>

      {isOpen && (
        <div className="p-4 pt-0 space-y-6">
          {/* ===== Keys management ============================================================ */}
          <div>
            <div className="flex items-center gap-2 mb-3">
              <Key className="h-4 w-4 text-primary" />
              <h5 className="text-xs uppercase tracking-widest font-bold text-foreground/70">
                Attached Keys
              </h5>
            </div>
            <div className="space-y-2">
              {did.keys.map((k) => {
                const isDefault = k.internal === did.default_key?.internal;
                return (
                  <div
                    key={`${k.internal}-${k.fragment}`}
                    className="flex items-center justify-between gap-3 bg-white/5 rounded-lg p-3 border border-white/5"
                  >
                    <div className="min-w-0 flex-1 space-y-1">
                      <div className="font-mono text-xs text-foreground/80 truncate">
                        #{k.fragment}
                      </div>
                      <div className="font-mono text-[10px] text-muted-foreground truncate">
                        {k.internal}
                      </div>
                    </div>
                    <div className="flex items-center gap-2">
                      {isDefault ? (
                        <Badge variant="default" className="text-[10px]">
                          DEFAULT
                        </Badge>
                      ) : (
                        <Button
                          size="sm"
                          variant="ghost"
                          disabled={!isWeb || setDefaultKey.isPending}
                          onClick={() => setDefaultKey.mutate(k.internal)}
                          title={
                            isWeb
                              ? "Set as default signing key"
                              : "did:jwk has a single key — cannot change default"
                          }
                        >
                          <Star className="h-3 w-3" />
                        </Button>
                      )}
                      <Button
                        size="sm"
                        variant="ghost"
                        disabled={!isWeb || did.keys.length === 1 || removeKey.isPending}
                        onClick={() => removeKey.mutate(k.internal)}
                        title={
                          !isWeb
                            ? "did:jwk keys must be removed by deleting the DID itself"
                            : did.keys.length === 1
                              ? "Cannot remove the only key of a DID"
                              : "Remove key from DID"
                        }
                      >
                        <Trash2 className="h-3 w-3 text-destructive" />
                      </Button>
                    </div>
                  </div>
                );
              })}
            </div>

            {isWeb ? (
              <KeyPicker
                excludeIds={did.keys.map((k) => k.internal)}
                value={newKeyId}
                onChange={setNewKeyId}
                disabled={addKey.isPending}
                onSubmit={() => addKey.mutate(newKeyId.trim())}
                isSubmitting={addKey.isPending}
              />
            ) : (
              <p className="mt-3 text-[10px] text-muted-foreground italic">
                did:jwk DIDs hold a single key bound to their identifier. Create a new DID instead.
              </p>
            )}
          </div>

          {/* ===== Raw document ============================================================== */}
          <div>
            <div className="bg-black/40 rounded-xl border border-white/5 overflow-hidden">
              <div className="flex items-center justify-between px-4 py-2 border-b border-white/5 bg-white/[0.02]">
                <span className="text-[10px] font-bold uppercase tracking-widest text-primary/60 flex items-center gap-2">
                  <FileJson className="h-3 w-3" />
                  DID Document
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
        </div>
      )}
    </div>
  );
};

const KeyPicker = ({
  excludeIds,
  value,
  onChange,
  disabled,
  onSubmit,
  isSubmitting,
}: {
  excludeIds: string[];
  value: string;
  onChange: (v: string) => void;
  disabled: boolean;
  onSubmit: () => void;
  isSubmitting: boolean;
}) => {
  const { data: keysRes } = useQuery({
    queryKey: KEYS_QUERY_KEY,
    queryFn: () => customInstance<KeysResponse>("/wallet/keys", { method: "GET" }),
  });

  const allKeys = keysRes?.status === 200 ? keysRes.data : [];
  const candidates = allKeys.filter((k) => !excludeIds.includes(k.id));

  return (
    <div className="mt-4 space-y-2">
      <Label className="text-[10px] uppercase tracking-widest text-muted-foreground/70">
        Attach existing key
      </Label>
      <div className="flex items-center gap-2">
        <select
          className="flex-1 bg-black/30 border border-white/10 rounded px-3 py-2 text-xs font-mono text-foreground"
          value={value}
          onChange={(e) => onChange(e.target.value)}
        >
          <option value="">— Choose a key —</option>
          {candidates.map((k) => (
            <option key={k.id} value={k.id}>
              {k.alias ? `${k.alias} (${k.id})` : k.id}
            </option>
          ))}
        </select>
        <Button size="sm" disabled={!value || disabled} onClick={onSubmit}>
          {isSubmitting ? (
            <Loader2 className="h-3 w-3 animate-spin" />
          ) : (
            <Plus className="h-3 w-3" />
          )}
          <span className="ml-1">Add key</span>
        </Button>
      </div>
      {candidates.length === 0 && (
        <p className="text-[10px] text-muted-foreground italic">
          No spare keys available. Create one from the Keys tab.
        </p>
      )}
    </div>
  );
};

/**
 * Mirrors the backend `DidService` JSON shape exactly:
 *   { id?: "AuthorizationServer" | "CredentialIssuer" | "FederatedCatalog" | string,
 *     type: string,
 *     serviceEndpoint: string }
 */
interface DidServiceInput {
  id?: string;
  type: string;
  serviceEndpoint: string;
}

const SERVICE_TYPES = [
  "AuthorizationServer",
  "CredentialIssuer",
  "FederatedCatalog",
] as const;

type DidKind = "jwk" | "web";

const NewDidDialog = ({ onCreated }: { onCreated: () => Promise<void> }) => {
  const [open, setOpen] = useState(false);
  const [kind, setKind] = useState<DidKind>("jwk");

  const [alias, setAlias] = useState("");
  const [domain, setDomain] = useState("");
  const [path, setPath] = useState("");
  const [port, setPort] = useState("");
  const [keysId, setKeysId] = useState<string[]>([]);
  const [services, setServices] = useState<DidServiceInput[]>([]);

  const { data: keysRes } = useQuery({
    queryKey: KEYS_QUERY_KEY,
    queryFn: () => customInstance<KeysResponse>("/wallet/keys", { method: "GET" }),
  });
  const allKeys = keysRes?.status === 200 ? keysRes.data : [];

  const resetForm = () => {
    setKind("jwk");
    setAlias("");
    setDomain("");
    setPath("");
    setPort("");
    setKeysId([]);
    setServices([]);
  };

  const register = useMutation({
    mutationFn: () => {
      const builder =
        kind === "web"
          ? {
              Web: {
                domain: domain.trim(),
                path: path.trim() ? path.trim() : null,
                port: port.trim() ? port.trim() : null,
              },
            }
          : { Jwk: { pem: "" } }; // backend resolves the PEM from keys_id[0]
      const cleanedServices = services
        .map((s) => ({
          ...(s.id ? { id: s.id } : {}),
          type: s.type.trim(),
          serviceEndpoint: s.serviceEndpoint.trim(),
        }))
        .filter((s) => s.type && s.serviceEndpoint);
      return customInstance("/wallet/did", {
        method: "POST",
        data: {
          builder,
          keys_id: keysId,
          alias,
          service: cleanedServices.length > 0 ? cleanedServices : null,
        },
      });
    },
    onSuccess: async () => {
      toast.success("DID registered");
      setOpen(false);
      resetForm();
      await onCreated();
    },
    onError: (err) => reportError(err, "Failed to register DID"),
  });

  const toggleKey = (id: string) => {
    if (kind === "jwk") {
      setKeysId([id]); // jwk binds to a single key
    } else {
      setKeysId((curr) =>
        curr.includes(id) ? curr.filter((k) => k !== id) : [...curr, id],
      );
    }
  };

  const addService = () =>
    setServices((s) => [...s, { type: "", serviceEndpoint: "" }]);
  const removeService = (idx: number) =>
    setServices((s) => s.filter((_, i) => i !== idx));
  const patchService = (idx: number, patch: Partial<DidServiceInput>) =>
    setServices((s) => s.map((v, i) => (i === idx ? { ...v, ...patch } : v)));

  const canSubmit =
    alias.trim() &&
    keysId.length > 0 &&
    (kind === "jwk" || domain.trim() !== "") &&
    !register.isPending;

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>
        <Button size="sm">
          <Plus className="h-3 w-3" />
          <span className="ml-1">New DID</span>
        </Button>
      </DialogTrigger>
      <DialogContent className="max-w-2xl max-h-[85vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>Register a new DID</DialogTitle>
          <DialogDescription>
            Pick a method, attach an existing wallet key, optionally declare service endpoints.
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-5">
          {/* Method selector */}
          <div className="space-y-1">
            <Label className="text-xs">DID method</Label>
            <div className="flex gap-2">
              <Button
                size="sm"
                variant={kind === "jwk" ? "default" : "outline"}
                onClick={() => {
                  setKind("jwk");
                  setKeysId((curr) => curr.slice(0, 1));
                }}
              >
                did:jwk
              </Button>
              <Button
                size="sm"
                variant={kind === "web" ? "default" : "outline"}
                onClick={() => setKind("web")}
              >
                did:web
              </Button>
            </div>
            <p className="text-[10px] text-muted-foreground italic">
              {kind === "jwk"
                ? "did:jwk derives the identifier from a single key's public material."
                : "did:web is hosted at the given URL; you can bind multiple keys."}
            </p>
          </div>

          <div className="space-y-1">
            <Label className="text-xs">Alias</Label>
            <Input
              placeholder={kind === "jwk" ? "my-jwk-did" : "my-web-did"}
              value={alias}
              onChange={(e) => setAlias(e.target.value)}
            />
          </div>

          {kind === "web" && (
            <>
              <div className="space-y-1">
                <Label className="text-xs">Domain</Label>
                <Input
                  placeholder="example.com"
                  value={domain}
                  onChange={(e) => setDomain(e.target.value)}
                />
              </div>

              <div className="grid grid-cols-2 gap-3">
                <div className="space-y-1">
                  <Label className="text-xs">Path (optional)</Label>
                  <Input
                    placeholder="agents/01"
                    value={path}
                    onChange={(e) => setPath(e.target.value)}
                  />
                </div>
                <div className="space-y-1">
                  <Label className="text-xs">Port (optional)</Label>
                  <Input
                    placeholder="8443"
                    value={port}
                    onChange={(e) => setPort(e.target.value)}
                  />
                </div>
              </div>
            </>
          )}

          {/* Keys */}
          <div className="space-y-1">
            <Label className="text-xs">
              {kind === "jwk" ? "Bind key (single)" : "Attach keys (one or more)"}
            </Label>
            {allKeys.length === 0 ? (
              <p className="text-[10px] text-muted-foreground italic">
                No keys available. Create one from the Keys tab.
              </p>
            ) : (
              <div className="space-y-1 max-h-40 overflow-y-auto border border-white/10 rounded-lg p-2">
                {allKeys.map((k) => (
                  <label
                    key={k.id}
                    className="flex items-center gap-2 p-1 text-xs cursor-pointer hover:bg-white/5 rounded"
                  >
                    <input
                      type={kind === "jwk" ? "radio" : "checkbox"}
                      name="did-keys"
                      checked={keysId.includes(k.id)}
                      onChange={() => toggleKey(k.id)}
                    />
                    <span className="font-mono">{k.alias || k.id}</span>
                    <span className="font-mono text-[10px] text-muted-foreground truncate">
                      {k.id}
                    </span>
                  </label>
                ))}
              </div>
            )}
          </div>

          {/* Services */}
          <div className="space-y-2">
            <div className="flex items-center justify-between">
              <Label className="text-xs">Services (optional)</Label>
              <Button size="sm" variant="ghost" onClick={addService}>
                <Plus className="h-3 w-3" />
                <span className="ml-1">Add service</span>
              </Button>
            </div>
            {services.length === 0 ? (
              <p className="text-[10px] text-muted-foreground italic">
                Declare any service endpoint to expose in the DID document.
              </p>
            ) : (
              <div className="space-y-2">
                {services.map((svc, idx) => (
                  <div
                    key={idx}
                    className="border border-white/10 rounded-lg p-3 space-y-2 bg-white/[0.02]"
                  >
                    <div className="flex items-center gap-2">
                      <div className="flex-1 space-y-1">
                        <Label className="text-[10px] text-muted-foreground">Type</Label>
                        <input
                          list={`service-types-${idx}`}
                          className="w-full bg-black/30 border border-white/10 rounded px-2 py-1.5 text-xs font-mono text-foreground"
                          placeholder="AuthorizationServer / CredentialIssuer / ..."
                          value={svc.type}
                          onChange={(e) => patchService(idx, { type: e.target.value })}
                        />
                        <datalist id={`service-types-${idx}`}>
                          {SERVICE_TYPES.map((t) => (
                            <option key={t} value={t} />
                          ))}
                        </datalist>
                      </div>
                      <Button
                        size="sm"
                        variant="ghost"
                        onClick={() => removeService(idx)}
                        className="mt-4"
                      >
                        <Trash2 className="h-3 w-3 text-destructive" />
                      </Button>
                    </div>
                    <div className="space-y-1">
                      <Label className="text-[10px] text-muted-foreground">Service Endpoint</Label>
                      <Input
                        placeholder="https://example.com/oidc"
                        value={svc.serviceEndpoint}
                        onChange={(e) =>
                          patchService(idx, { serviceEndpoint: e.target.value })
                        }
                      />
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={() => setOpen(false)}>
            Cancel
          </Button>
          <Button disabled={!canSubmit} onClick={() => register.mutate()}>
            {register.isPending ? (
              <Loader2 className="h-3 w-3 animate-spin mr-1" />
            ) : (
              <Plus className="h-3 w-3 mr-1" />
            )}
            Register DID
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};

export const Route = createFileRoute("/wallet/did")({
  component: WalletDIDPage,
});
