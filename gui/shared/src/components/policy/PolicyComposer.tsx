import React, { useEffect, useMemo, useState } from "react";
import { Button } from "shared/src/components/ui/button";
import { Textarea } from "shared/src/components/ui/textarea";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "shared/src/components/ui/dialog";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "shared/src/components/ui/tabs";
import { JsonViewer } from "shared/src/components/json/JsonViewer";
import { OdrlRulesView } from "./OdrlRulesView";
import { ControlPointChips } from "./ControlPointChips";
import { PolicyBuilder } from "./builder/PolicyBuilder";
import { PolicyTreeOps, type PolicyTree } from "./builder/policyTree";
import { useGetAllPolicyTemplates } from "shared/src/data/orval/policy-templates/policy-templates";
import { PolicyTemplateApi } from "shared/src/data/policyTemplateApi";
import type {
  PaginatedPolicyTemplateDto,
  PolicyPart,
  PolicyPreviewDto,
} from "shared/src/data/orval/model";

export interface PolicyComposerProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  /** What the policy is attached to, shown in the title. */
  targetLabel?: React.ReactNode;
  /** Receives the parts to compose, in order, and the offer's description. */
  onSubmit: (parts: PolicyPart[], description?: string) => Promise<void> | void;
}

/** Dialog to compose an ODRL offer by dragging library pieces and blocks onto a canvas; the agent
 * flattens it and the preview shows the plain ODRL a consumer will see. */
export function PolicyComposer({ open, onOpenChange, targetLabel, onSubmit }: PolicyComposerProps) {
  const [tree, setTree] = useState<PolicyTree>(PolicyTreeOps.empty);
  const [description, setDescription] = useState("");
  const [submitting, setSubmitting] = useState(false);
  const [preview, setPreview] = useState<PolicyPreviewDto | null>(null);
  const [previewError, setPreviewError] = useState<string | null>(null);

  const { data } = useGetAllPolicyTemplates({ limit: 100 }, { query: { enabled: open } });
  const templates = useMemo(() => {
    const items = data?.status === 200 ? (data.data as PaginatedPolicyTemplateDto).items : [];
    return PolicyTemplateApi.group(items).map((g) => g.latest);
  }, [data]);

  const parts = useMemo(() => PolicyTreeOps.parts(tree), [tree]);
  const problem = PolicyTreeOps.problems(tree, "offer")[0];

  // The agent flattens the parts; ask again a moment after the last change.
  useEffect(() => {
    if (!open || problem) {
      setPreview(null);
      setPreviewError(null);
      return;
    }
    let current = true;
    const timer = setTimeout(() => {
      PolicyTemplateApi.previewParts(parts)
        .then((p) => current && (setPreview(p), setPreviewError(null)))
        .catch((e: Error) => current && (setPreview(null), setPreviewError(e.message)));
    }, 400);
    return () => {
      current = false;
      clearTimeout(timer);
    };
  }, [open, problem, parts]);

  const submit = async () => {
    setSubmitting(true);
    try {
      await onSubmit(parts, description.trim() || undefined);
      setTree(PolicyTreeOps.empty());
      setDescription("");
      onOpenChange(false);
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="flex h-[92vh] max-w-[min(96vw,90rem)] flex-col gap-0 p-0">
        <DialogHeader className="border-b border-ink/10 px-6 py-4">
          <DialogTitle>New access policy</DialogTitle>
          <DialogDescription>
            {targetLabel && <>For {targetLabel}. </>}Drag pieces from the library and blocks onto
            the canvas, nest conditions inside rules and fill in their values.
          </DialogDescription>
        </DialogHeader>

        <div className="grid min-h-0 flex-1 grid-cols-1 overflow-hidden xl:grid-cols-[minmax(0,1fr)_24rem]">
          <div className="flex min-h-0 flex-col gap-4 overflow-y-auto px-6 py-5">
            <Textarea
              rows={2}
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              placeholder="What this offer allows, in a sentence consumers will read in your catalog"
              className="resize-none"
            />
            <PolicyBuilder mode="offer" tree={tree} onChange={setTree} templates={templates} />
          </div>

          <div className="overflow-y-auto border-t border-ink/10 bg-ink/[0.015] px-6 py-5 xl:border-l xl:border-t-0">
            <div className="mb-3 flex flex-wrap items-center gap-2">
              <span className="text-xs text-ink/55">Meant to apply at</span>
              {preview?.controlPoints.length ? (
                <ControlPointChips points={preview.controlPoints} />
              ) : (
                <span className="text-xs text-ink/40">—</span>
              )}
            </div>
            {previewError && (
              <p className="mb-3 rounded-md border border-danger-500/30 bg-danger-500/10 px-3 py-2 text-xs text-danger-700 dark:text-danger-300">
                {previewError}
              </p>
            )}
            <Tabs defaultValue="rules">
              <TabsList>
                <TabsTrigger value="rules">Preview</TabsTrigger>
                <TabsTrigger value="json">ODRL</TabsTrigger>
              </TabsList>
              <TabsContent value="rules" className="pt-4">
                {preview ? (
                  <OdrlRulesView policy={preview.odrl} />
                ) : (
                  <p className="text-sm text-ink/50">{problem ?? "Composing…"}</p>
                )}
              </TabsContent>
              <TabsContent value="json" className="pt-4">
                <JsonViewer value={preview?.odrl ?? {}} />
              </TabsContent>
            </Tabs>
          </div>
        </div>

        <DialogFooter className="flex items-center gap-3 border-t border-ink/10 px-6 py-3 sm:justify-between">
          <span className="text-sm text-ink/55">{problem ?? previewError}</span>
          <div className="flex gap-2">
            <Button variant="outline" onClick={() => onOpenChange(false)}>
              Cancel
            </Button>
            <Button onClick={submit} disabled={!!problem || !!previewError} isLoading={submitting}>
              Add policy
            </Button>
          </div>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
