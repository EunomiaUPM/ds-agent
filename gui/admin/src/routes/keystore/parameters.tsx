import { createFileRoute } from "@tanstack/react-router";
import { useQueryClient } from "@tanstack/react-query";
import {
  useListParameters,
  useCreateParameter,
  useDeleteParameter,
  useUpdateParameter,
  getListParametersQueryKey,
} from "shared/src/data/orval/keystore-parameters/keystore-parameters";
import { KeystoreParameterView } from "shared/src/data/orval/model";
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
import { Eye, Trash2, Pencil, Plus } from "lucide-react";

// ---------------------------------------------------------------------------
// New dialog
// ---------------------------------------------------------------------------

interface NewParameterDialogProps {
  open: boolean;
  onClose: () => void;
}

const NewParameterDialog = ({ open, onClose }: NewParameterDialogProps) => {
  const queryClient = useQueryClient();
  const [key, setKey] = useState("");
  const [valueStr, setValueStr] = useState("");
  const [description, setDescription] = useState("");
  const [jsonError, setJsonError] = useState<string | null>(null);

  const { mutate: create, isPending } = useCreateParameter({
    mutation: {
      onSuccess: () => {
        queryClient.invalidateQueries({ queryKey: getListParametersQueryKey() });
        onClose();
      },
    },
  });

  const handleSubmit = () => {
    if (!key.trim()) {
      setJsonError("Key is required");
      return;
    }
    let parsed: unknown;
    try {
      parsed = JSON.parse(valueStr);
      setJsonError(null);
    } catch {
      setJsonError("Invalid JSON");
      return;
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
          <DialogTitle>New parameter</DialogTitle>
        </DialogHeader>

        <div className="space-y-4 py-2">
          <div className="space-y-1.5">
            <label className="text-sm font-medium">Key</label>
            <Input
              className="font-mono text-xs"
              placeholder="/my/parameter/key"
              value={key}
              onChange={(e) => {
                setKey(e.target.value);
                setJsonError(null);
              }}
            />
          </div>

          <div className="space-y-1.5">
            <label className="text-sm font-medium">Value (JSON)</label>
            <Textarea
              className="font-mono text-xs min-h-[100px]"
              placeholder='"string", 42, {"key": "value"}, [...]'
              value={valueStr}
              onChange={(e) => {
                setValueStr(e.target.value);
                setJsonError(null);
              }}
            />
            {jsonError && <p className="text-xs text-destructive">{jsonError}</p>}
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

interface EditParameterDialogProps {
  param: KeystoreParameterView;
  open: boolean;
  onClose: () => void;
}

const EditParameterDialog = ({ param, open, onClose }: EditParameterDialogProps) => {
  const queryClient = useQueryClient();
  const [valueStr, setValueStr] = useState(JSON.stringify(param.value, null, 2));
  const [description, setDescription] = useState(param.description ?? "");
  const [jsonError, setJsonError] = useState<string | null>(null);

  const { mutate: update, isPending } = useUpdateParameter({
    mutation: {
      onSuccess: () => {
        queryClient.invalidateQueries({ queryKey: getListParametersQueryKey() });
        onClose();
      },
    },
  });

  const handleSubmit = () => {
    let parsed: unknown;
    try {
      parsed = JSON.parse(valueStr);
      setJsonError(null);
    } catch {
      setJsonError("Invalid JSON");
      return;
    }

    update({
      key: param.key.replace(/^\//, ""),
      data: {
        value: parsed,
        expectedVersion: param.version,
        description: description || null,
      },
    });
  };

  return (
    <Dialog open={open} onOpenChange={(o) => !o && onClose()}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Edit parameter</DialogTitle>
          <p className="text-xs font-mono text-muted-foreground mt-1">{param.key}</p>
        </DialogHeader>

        <div className="space-y-4 py-2">
          <div className="space-y-1.5">
            <label className="text-sm font-medium">Value (JSON)</label>
            <Textarea
              className="font-mono text-xs min-h-[120px]"
              value={valueStr}
              onChange={(e) => {
                setValueStr(e.target.value);
                setJsonError(null);
              }}
            />
            {jsonError && <p className="text-xs text-destructive">{jsonError}</p>}
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
            Current version: {param.version} — will be incremented on save
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
// Page
// ---------------------------------------------------------------------------

const KeystoreParameters = () => {
  const queryClient = useQueryClient();
  const { data: response, isLoading, error } = useListParameters();
  const [creating, setCreating] = useState(false);
  const [editing, setEditing] = useState<KeystoreParameterView | null>(null);
  const [inspecting, setInspecting] = useState<KeystoreParameterView | null>(null);

  const { mutate: del } = useDeleteParameter({
    mutation: {
      onSuccess: () => {
        queryClient.invalidateQueries({ queryKey: getListParametersQueryKey() });
      },
    },
  });

  if (isLoading) {
    return (
      <PageSection title="Parameters">
        <div className="space-y-3">
          {[...Array(3)].map((_, i) => (
            <Skeleton key={i} className="h-20 w-full rounded-xl" />
          ))}
        </div>
      </PageSection>
    );
  }

  if (error || response?.status !== 200) {
    return (
      <PageSection title="Parameters">
        <p className="text-destructive font-mono text-xs">Error loading parameters</p>
      </PageSection>
    );
  }

  const params = response.data;

  return (
    <>
      <PageSection
        title="Parameters"
        action={
          <Button size="sm" onClick={() => setCreating(true)}>
            <Plus className="h-3.5 w-3.5 mr-1" />
            New
          </Button>
        }
      >
        <DataTable
          className="text-sm"
          data={params}
          keyExtractor={(p) => p.key}
          searchPlaceholder="Filter parameters by key or description..."
          emptyMessage="No parameters yet"
          defaultSortKey="key"
          defaultSortDirection="asc"
          pageSize={10}
          columns={[
            {
              header: "Key",
              accessorKey: "key",
              cell: (p) => <span className="font-mono text-xs">{p.key}</span>,
            },
            {
              header: "Description",
              accessorKey: "description",
              cell: (p) =>
                p.description ? (
                  <span className="text-xs text-muted-foreground block max-w-[280px] truncate">
                    {p.description}
                  </span>
                ) : (
                  <span className="text-xs text-muted-foreground/50">—</span>
                ),
            },
            {
              header: "Version",
              accessorKey: "version",
              cell: (p) => <Badge variant="info">v{p.version}</Badge>,
            },
            {
              header: "Value",
              sortable: false,
              searchValue: (p) => JSON.stringify(p.value),
              cell: (p) => (
                <span className="font-mono text-xs text-muted-foreground block max-w-[260px] truncate">
                  {JSON.stringify(p.value)}
                </span>
              ),
            },
            {
              header: "Updated at",
              accessorKey: "updatedAt",
              sortValue: (p) => new Date(p.updatedAt).getTime(),
              cell: (p) => <FormatDate date={p.updatedAt} />,
            },
            {
              header: "Actions",
              sortable: false,
              searchable: false,
              cell: (p) => (
                <div className="flex items-center gap-1">
                  <Button
                    variant="ghost"
                    size="icon"
                    className="h-7 w-7"
                    onClick={() => setInspecting(p)}
                  >
                    <Eye className="h-3.5 w-3.5" />
                  </Button>
                  <Button
                    variant="ghost"
                    size="icon"
                    className="h-7 w-7 text-muted-foreground hover:text-foreground"
                    onClick={() => setEditing(p)}
                  >
                    <Pencil className="h-3.5 w-3.5" />
                  </Button>
                  <Button
                    variant="ghost"
                    size="icon"
                    className="h-7 w-7 text-destructive hover:text-destructive hover:bg-destructive/10"
                    onClick={() => del({ key: p.key.replace(/^\//, "") })}
                  >
                    <Trash2 className="h-3.5 w-3.5" />
                  </Button>
                </div>
              ),
            },
          ]}
        />
      </PageSection>

      {creating && <NewParameterDialog open={creating} onClose={() => setCreating(false)} />}
      {editing && (
        <EditParameterDialog param={editing} open={!!editing} onClose={() => setEditing(null)} />
      )}

      <Dialog open={!!inspecting} onOpenChange={() => setInspecting(null)}>
        <DialogContent className="max-w-2xl max-h-[85vh] flex flex-col">
          <DialogHeader>
            <DialogTitle className="font-mono text-sm">{inspecting?.key}</DialogTitle>
          </DialogHeader>
          <pre className="flex-1 overflow-y-auto rounded bg-muted/40 p-4 text-xs font-mono whitespace-pre-wrap break-all">
            {JSON.stringify(inspecting?.value, null, 2)}
          </pre>
          <DialogFooter>
            <Button onClick={() => setInspecting(null)}>Close</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </>
  );
};

export const Route = createFileRoute("/keystore/parameters")({
  component: KeystoreParameters,
});
