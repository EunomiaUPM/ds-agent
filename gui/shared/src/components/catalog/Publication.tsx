// Publication state of datasets and distributions: a draft stays in the management API and here,
// but DSP, DCAT 3 and DCAT-AP leave it out, so federated crawlers never see it.

import React, { useState } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { toast } from "sonner";
import { EyeOff, Send } from "lucide-react";
import { cn } from "@eunomia/ui";
import { Button, type ButtonSizes } from "@eunomia/ui";
import { ConfirmDialog } from "@eunomia/ui";
import { CatalogResources } from "shared/src/data/catalogResources";

type Kind = "dataset" | "distribution";

/** "Draft" or "Published", as a small pill. */
export function PublicationBadge({ draft, className }: { draft: boolean; className?: string }) {
  return (
    <span
      title={
        draft
          ? "Draft: not in the DSP catalog nor the DCAT 3 / DCAT-AP exports"
          : "Published: peers and crawlers see it"
      }
      className={cn(
        "inline-flex items-center rounded-full border px-2 py-0.5 text-xs font-medium",
        draft
          ? "border-warn-600/40 bg-warn-50 text-warn-800 dark:bg-warn-950/40 dark:text-warn-200"
          : "border-success-600/40 bg-success-50 text-success-800 dark:bg-success-950/40 dark:text-success-200",
        className,
      )}
    >
      {draft ? "Draft" : "Published"}
    </span>
  );
}

interface ToggleProps {
  kind: Kind;
  id: string;
  draft: boolean;
  title?: string;
  size?: ButtonSizes;
  /** Only the icon, for table rows. */
  compact?: boolean;
}

/** Publishes a draft at once; moving a published one back to draft asks first. */
export function PublicationToggle({
  kind,
  id,
  draft,
  title,
  size = "default",
  compact,
}: ToggleProps) {
  const queryClient = useQueryClient();
  const [asking, setAsking] = useState(false);
  const [busy, setBusy] = useState(false);

  const apply = async (toDraft: boolean) => {
    await CatalogResources.setPublication(kind, id, toDraft);
    await CatalogResources.invalidate(queryClient);
    toast.success(toDraft ? `The ${kind} is a draft now.` : `Published the ${kind}.`);
  };

  const publish = async () => {
    setBusy(true);
    try {
      await apply(false);
    } catch (e) {
      toast.error(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  };

  const label = draft ? "Publish" : "Move to draft";
  return (
    <>
      <Button
        size={compact ? "icon_sm" : size}
        variant={draft ? "default" : "outline"}
        title={compact ? label : undefined}
        isLoading={busy}
        onClick={(e: React.MouseEvent) => {
          e.stopPropagation();
          if (draft) publish();
          else setAsking(true);
        }}
      >
        {draft ? <Send /> : <EyeOff />}
        {!compact && label}
      </Button>
      <ConfirmDialog
        open={asking}
        onOpenChange={setAsking}
        title={`Move this ${kind} to draft?`}
        description={
          <>
            <span className="font-medium text-ink/90">{title || id}</span> stays here and in the
            management API, but leaves the DSP catalog peers read and the DCAT 3 / DCAT-AP exports.
          </>
        }
        confirmLabel="Move to draft"
        destructive={false}
        onConfirm={() => apply(true)}
      >
        {kind === "dataset"
          ? "Its distributions and offers go with it. Peers that already fetched it are not told yet."
          : "Consumers can no longer find it in the dataset's distributions."}
      </ConfirmDialog>
    </>
  );
}
