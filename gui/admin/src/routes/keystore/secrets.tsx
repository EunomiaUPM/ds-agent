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
import { DataTable } from "shared/src/components/DataTable";
import { FormatDate } from "shared/src/components/ui/format-date";
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
import { Trash2, Pencil, Plus } from "lucide-react";

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
            <p className="text-xs text-muted-foreground/60">
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
            <p className="text-xs text-muted-foreground/60">
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

          <p className="text-xs text-muted-foreground/60">
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
// Page
// ---------------------------------------------------------------------------

const KeystoreSecrets = () => {
  const queryClient = useQueryClient();
  const { data: response, isLoading, error } = useListSecrets();
  const [creating, setCreating] = useState(false);
  const [editing, setEditing] = useState<KeystoreSecretView | null>(null);

  const { mutate: del } = useDeleteSecret({
    mutation: {
      onSuccess: () => {
        queryClient.invalidateQueries({ queryKey: getListSecretsQueryKey() });
      },
    },
  });

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
        title="Secrets"
        action={
          <Button size="sm" onClick={() => setCreating(true)}>
            <Plus className="h-3.5 w-3.5 mr-1.5" />
            New
          </Button>
        }
      >
        <DataTable
          className="text-sm"
          data={secrets}
          keyExtractor={(s) => s.key}
          searchPlaceholder="Filter secrets by key or description..."
          emptyMessage="No secrets yet"
          columns={[
            {
              header: "Key",
              accessorKey: "key",
              cell: (s) => <span className="font-mono text-xs">{s.key}</span>,
            },
            {
              header: "Description",
              accessorKey: "description",
              cell: (s) =>
                s.description ? (
                  <span className="text-xs text-muted-foreground block max-w-[280px] truncate">
                    {s.description}
                  </span>
                ) : (
                  <span className="text-xs text-muted-foreground/50">—</span>
                ),
            },
            {
              header: "Version",
              accessorKey: "version",
              cell: (s) => <Badge variant="info">v{s.version}</Badge>,
            },
            {
              header: "Value",
              accessorKey: "value",
              sortable: false,
              cell: (s) => <Badge variant="code">{s.value}</Badge>,
            },
            {
              header: "Updated at",
              accessorKey: "updatedAt",
              sortValue: (s) => new Date(s.updatedAt).getTime(),
              cell: (s) => <FormatDate date={s.updatedAt} />,
            },
            {
              header: "Actions",
              sortable: false,
              searchable: false,
              cell: (s) => (
                <div className="flex items-center gap-1">
                  <Button
                    variant="ghost"
                    size="icon"
                    className="h-7 w-7 text-muted-foreground hover:text-foreground"
                    onClick={() => setEditing(s)}
                  >
                    <Pencil className="h-3.5 w-3.5" />
                  </Button>
                  <Button
                    variant="ghost"
                    size="icon"
                    className="h-7 w-7 text-destructive hover:text-destructive hover:bg-destructive/10"
                    onClick={() => del({ key: s.key.replace(/^\//, "") })}
                  >
                    <Trash2 className="h-3.5 w-3.5" />
                  </Button>
                </div>
              ),
            },
          ]}
        />
      </PageSection>

      {creating && <NewSecretDialog open={creating} onClose={() => setCreating(false)} />}
      {editing && (
        <EditSecretDialog secret={editing} open={!!editing} onClose={() => setEditing(null)} />
      )}
    </>
  );
};

export const Route = createFileRoute("/keystore/secrets")({
  component: KeystoreSecrets,
});
