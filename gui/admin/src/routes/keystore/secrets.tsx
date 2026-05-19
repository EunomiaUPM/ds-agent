import { createFileRoute } from "@tanstack/react-router";
import { useQueryClient } from "@tanstack/react-query";
import {
  useListSecrets,
  useCreateSecret,
  useDeleteSecret,
  useUpdateSecret,
  getListSecretsQueryKey,
} from "shared/src/data/orval/keystore-secrets/keystore-secrets";
import { KeystoreSecretView } from "shared/src/data/orval/model";
import { PageSection } from "shared/src/components/layout/PageSection";
import { Skeleton } from "shared/src/components/ui/skeleton";
import { Button } from "shared/src/components/ui/button";
import { Badge } from "shared/src/components/ui/badge";
import { Input } from "shared/src/components/ui/input";
import { Textarea } from "shared/src/components/ui/textarea";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from "shared/src/components/ui/dialog";
import { useState } from "react";
import { ShieldAlert, Trash2, Pencil, Plus } from "lucide-react";

// ---------------------------------------------------------------------------
// New dialog
// ---------------------------------------------------------------------------

interface NewSecretDialogProps {
  open: boolean;
  onClose: () => void;
}

const NewSecretDialog = ({ open, onClose }: NewSecretDialogProps) => {
  const queryClient = useQueryClient();
  const [key, setKey] = useState("");
  const [valueStr, setValueStr] = useState("");
  const [description, setDescription] = useState("");
  const [error, setError] = useState<string | null>(null);

  const { mutate: create, isPending } = useCreateSecret({
    mutation: {
      onSuccess: () => {
        queryClient.invalidateQueries({ queryKey: getListSecretsQueryKey() });
        onClose();
      },
    },
  });

  const handleSubmit = () => {
    if (!key.trim()) {
      setError("Key is required");
      return;
    }
    if (!valueStr.trim()) {
      setError("Value is required");
      return;
    }

    let parsed: unknown;
    try {
      parsed = JSON.parse(valueStr);
    } catch {
      parsed = valueStr;
    }

    create({
      data: {
        key: key.startsWith("/") ? key : `/${key}`,
        value: parsed,
        description: description || null,
      },
    });
  };

  return (
    <Dialog open={open} onOpenChange={(o) => !o && onClose()}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>New secret</DialogTitle>
        </DialogHeader>

        <div className="space-y-4 py-2">
          <div className="space-y-1.5">
            <label className="text-sm font-medium">Key</label>
            <Input
              className="font-mono text-xs"
              placeholder="/my/secret/key"
              value={key}
              onChange={(e) => {
                setKey(e.target.value);
                setError(null);
              }}
            />
          </div>

          <div className="space-y-1.5">
            <label className="text-sm font-medium">Value</label>
            <Textarea
              className="font-mono text-xs min-h-[100px]"
              placeholder='Enter secret value (any JSON: "string", 42, {...}, [...])'
              value={valueStr}
              onChange={(e) => {
                setValueStr(e.target.value);
                setError(null);
              }}
            />
            {error && <p className="text-xs text-destructive">{error}</p>}
            <p className="text-[11px] text-muted-foreground/60">
              Bare strings are stored as-is. JSON objects and arrays are also supported.
            </p>
          </div>

          <div className="space-y-1.5">
            <label className="text-sm font-medium">Description</label>
            <Input
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              placeholder="Optional description"
            />
          </div>
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={onClose}>
            Cancel
          </Button>
          <Button onClick={handleSubmit} isLoading={isPending}>
            Create
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};

// ---------------------------------------------------------------------------
// Edit dialog
// ---------------------------------------------------------------------------

interface EditSecretDialogProps {
  secret: KeystoreSecretView;
  open: boolean;
  onClose: () => void;
}

const EditSecretDialog = ({ secret, open, onClose }: EditSecretDialogProps) => {
  const queryClient = useQueryClient();
  const [valueStr, setValueStr] = useState("");
  const [description, setDescription] = useState(secret.description ?? "");
  const [jsonError, setJsonError] = useState<string | null>(null);

  const { mutate: update, isPending } = useUpdateSecret({
    mutation: {
      onSuccess: () => {
        queryClient.invalidateQueries({ queryKey: getListSecretsQueryKey() });
        onClose();
      },
    },
  });

  const handleSubmit = () => {
    if (!valueStr.trim()) {
      setJsonError("Value is required");
      return;
    }

    let parsed: unknown;
    try {
      parsed = JSON.parse(valueStr);
      setJsonError(null);
    } catch {
      // Treat bare string as a JSON string literal
      parsed = valueStr;
    }

    update({
      key: secret.key.replace(/^\//, ""),
      data: {
        value: parsed,
        expectedVersion: secret.version,
        description: description || null,
      },
    });
  };

  return (
    <Dialog open={open} onOpenChange={(o) => !o && onClose()}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Edit secret</DialogTitle>
          <p className="text-xs font-mono text-muted-foreground mt-1">{secret.key}</p>
        </DialogHeader>

        <div className="space-y-4 py-2">
          <div className="space-y-1.5">
            <label className="text-sm font-medium">New value</label>
            <Textarea
              className="font-mono text-xs min-h-[100px]"
              placeholder='Enter new secret value (any JSON: "string", 42, {...}, [...])'
              value={valueStr}
              onChange={(e) => {
                setValueStr(e.target.value);
                setJsonError(null);
              }}
            />
            {jsonError && <p className="text-xs text-destructive">{jsonError}</p>}
            <p className="text-[11px] text-muted-foreground/60">
              Current value is never shown. Enter a new value to replace it.
            </p>
          </div>

          <div className="space-y-1.5">
            <label className="text-sm font-medium">Description</label>
            <Input
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              placeholder="Optional description"
            />
          </div>

          <p className="text-[11px] text-muted-foreground/60">
            Current version: {secret.version} — will be incremented on save
          </p>
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={onClose}>
            Cancel
          </Button>
          <Button onClick={handleSubmit} isLoading={isPending}>
            Save
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};

// ---------------------------------------------------------------------------
// Row
// ---------------------------------------------------------------------------

const SecretRow = ({ secret }: { secret: KeystoreSecretView }) => {
  const queryClient = useQueryClient();
  const [editing, setEditing] = useState(false);

  const { mutate: del, isPending: isDeleting } = useDeleteSecret({
    mutation: {
      onSuccess: () => {
        queryClient.invalidateQueries({ queryKey: getListSecretsQueryKey() });
      },
    },
  });

  return (
    <>
      <div className="bg-white/[0.03] border border-white/10 rounded-xl p-4 flex items-start justify-between gap-4">
        <div className="flex items-start gap-3 min-w-0">
          <ShieldAlert className="h-4 w-4 mt-0.5 text-amber-400/70 shrink-0" />
          <div className="min-w-0 space-y-1">
            <p className="font-mono text-sm text-foreground/90 truncate">{secret.key}</p>
            {secret.description && (
              <p className="text-xs text-muted-foreground">{secret.description}</p>
            )}
            <div className="flex items-center gap-2 flex-wrap">
              <Badge variant="info" className="text-[10px] h-4 px-1.5">
                v{secret.version}
              </Badge>
              <code className="text-xs font-mono text-muted-foreground/60 bg-black/20 px-1.5 py-0.5 rounded">
                {secret.value}
              </code>
              <span className="text-[10px] text-muted-foreground/60">
                {new Date(secret.updatedAt).toLocaleString()}
              </span>
            </div>
          </div>
        </div>

        <div className="flex items-center gap-1 shrink-0">
          <Button
            variant="ghost"
            size="icon"
            className="h-7 w-7 text-muted-foreground hover:text-foreground"
            onClick={() => setEditing(true)}
          >
            <Pencil className="h-3.5 w-3.5" />
          </Button>
          <Button
            variant="ghost"
            size="icon"
            className="h-7 w-7 text-destructive/70 hover:text-destructive"
            isLoading={isDeleting}
            onClick={() => del({ key: secret.key.replace(/^\//, "") })}
          >
            <Trash2 className="h-3.5 w-3.5" />
          </Button>
        </div>
      </div>

      {editing && (
        <EditSecretDialog secret={secret} open={editing} onClose={() => setEditing(false)} />
      )}
    </>
  );
};

// ---------------------------------------------------------------------------
// Page
// ---------------------------------------------------------------------------

const KeystoreSecrets = () => {
  const { data: response, isLoading, error } = useListSecrets();
  const [creating, setCreating] = useState(false);

  if (isLoading) {
    return (
      <PageSection title="Secrets">
        <div className="space-y-3">
          {[...Array(3)].map((_, i) => (
            <Skeleton key={i} className="h-16 w-full rounded-xl" />
          ))}
        </div>
      </PageSection>
    );
  }

  if (error || response?.status !== 200) {
    return (
      <PageSection title="Secrets">
        <p className="text-destructive font-mono text-xs">Error loading secrets</p>
      </PageSection>
    );
  }

  const secrets = response.data;

  return (
    <>
      <PageSection
        title={`Secrets (${secrets.length})`}
        action={
          <Button size="sm" onClick={() => setCreating(true)}>
            <Plus className="h-3.5 w-3.5 mr-1.5" />
            New
          </Button>
        }
      >
        {secrets.length === 0 ? (
          <div className="flex flex-col items-center justify-center min-h-[200px] border border-dashed border-white/10 rounded-xl bg-white/5 p-8">
            <ShieldAlert className="h-8 w-8 text-muted-foreground/40 mb-3" />
            <p className="text-muted-foreground text-sm">No secrets yet</p>
          </div>
        ) : (
          <div className="space-y-3">
            {secrets.map((s) => (
              <SecretRow key={s.key} secret={s} />
            ))}
          </div>
        )}
      </PageSection>

      {creating && <NewSecretDialog open={creating} onClose={() => setCreating(false)} />}
    </>
  );
};

export const Route = createFileRoute("/keystore/secrets")({
  component: KeystoreSecrets,
});
