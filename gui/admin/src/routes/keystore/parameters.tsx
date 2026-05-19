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
import { cn } from "shared/src/lib/utils";
import { ChevronDown, ChevronUp, Trash2, SlidersHorizontal, Pencil, Plus } from "lucide-react";

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

          <p className="text-[11px] text-muted-foreground/60">
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
// ---------------------------------------------------------------------------

const ParameterRow = ({ param }: { param: KeystoreParameterView }) => {
  const queryClient = useQueryClient();
  const [expanded, setExpanded] = useState(false);
  const [editing, setEditing] = useState(false);

  const { mutate: del, isPending: isDeleting } = useDeleteParameter({
    mutation: {
      onSuccess: () => {
        queryClient.invalidateQueries({ queryKey: getListParametersQueryKey() });
      },
    },
  });

  const valueStr = JSON.stringify(param.value, null, 2);

  return (
    <>
      <div className="bg-white/[0.03] border border-white/10 rounded-xl overflow-hidden">
        <div
          className="flex items-start justify-between gap-4 p-4 cursor-pointer select-none"
          onClick={() => setExpanded((v) => !v)}
        >
          <div className="flex items-start gap-3 min-w-0">
            <SlidersHorizontal className="h-4 w-4 mt-0.5 text-primary/70 shrink-0" />
            <div className="min-w-0 space-y-1">
              <p className="font-mono text-sm text-foreground/90 truncate">{param.key}</p>
              {param.description && (
                <p className="text-xs text-muted-foreground">{param.description}</p>
              )}
              <div className="flex items-center gap-2 flex-wrap">
                <Badge variant="info" className="text-[10px] h-4 px-1.5">
                  v{param.version}
                </Badge>
                <span className="text-[10px] text-muted-foreground/60">
                  {new Date(param.updatedAt).toLocaleString()}
                </span>
              </div>
            </div>
          </div>

          <div className="flex items-center gap-1 shrink-0">
            <Button
              variant="ghost"
              size="icon"
              className="h-7 w-7"
              onClick={(e) => {
                e.stopPropagation();
                setExpanded((v) => !v);
              }}
            >
              {expanded ? (
                <ChevronUp className="h-3.5 w-3.5" />
              ) : (
                <ChevronDown className="h-3.5 w-3.5" />
              )}
            </Button>
            <Button
              variant="ghost"
              size="icon"
              className="h-7 w-7 text-muted-foreground hover:text-foreground"
              onClick={(e) => {
                e.stopPropagation();
                setEditing(true);
              }}
            >
              <Pencil className="h-3.5 w-3.5" />
            </Button>
            <Button
              variant="ghost"
              size="icon"
              className="h-7 w-7 text-destructive/70 hover:text-destructive"
              isLoading={isDeleting}
              onClick={(e) => {
                e.stopPropagation();
                del({ key: param.key.replace(/^\//, "") });
              }}
            >
              <Trash2 className="h-3.5 w-3.5" />
            </Button>
          </div>
        </div>

        <div
          className={cn(
            "grid transition-all duration-200 ease-in-out",
            expanded ? "grid-rows-[1fr]" : "grid-rows-[0fr]",
          )}
        >
          <div className="overflow-hidden">
            <pre className="px-4 pb-4 text-xs font-mono text-muted-foreground/80 bg-black/20 whitespace-pre-wrap break-all">
              {valueStr}
            </pre>
          </div>
        </div>
      </div>

      {editing && (
        <EditParameterDialog param={param} open={editing} onClose={() => setEditing(false)} />
      )}
    </>
  );
};

// ---------------------------------------------------------------------------
// Page
// ---------------------------------------------------------------------------

const KeystoreParameters = () => {
  const { data: response, isLoading, error } = useListParameters();
  const [creating, setCreating] = useState(false);

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
        title={`Parameters (${params.length})`}
        action={
          <Button size="sm" onClick={() => setCreating(true)}>
            <Plus className="h-3.5 w-3.5 mr-1" />
            New
          </Button>
        }
      >
        {params.length === 0 ? (
          <div className="flex flex-col items-center justify-center min-h-[200px] border border-dashed border-white/10 rounded-xl bg-white/5 p-8">
            <SlidersHorizontal className="h-8 w-8 text-muted-foreground/40 mb-3" />
            <p className="text-muted-foreground text-sm">No parameters yet</p>
          </div>
        ) : (
          <div className="space-y-3">
            {params.map((p) => (
              <ParameterRow key={p.key} param={p} />
            ))}
          </div>
        )}
      </PageSection>

      {creating && <NewParameterDialog open={creating} onClose={() => setCreating(false)} />}
    </>
  );
};

export const Route = createFileRoute("/keystore/parameters")({
  component: KeystoreParameters,
});
