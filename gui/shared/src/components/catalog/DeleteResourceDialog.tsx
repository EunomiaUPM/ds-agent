import React from "react";
import { useQueryClient } from "@tanstack/react-query";
import { toast } from "sonner";
import { ConfirmDialog } from "@eunomia/ui";
import { CatalogResources, type CatalogResourceKind } from "shared/src/data/catalogResources";
import { CatalogViews, type CatalogDoc } from "shared/src/data/catalog";
import { useGetMainDataService } from "shared/src/data/orval/data-services/data-services";

/** The resource to delete, as the dialog names it. */
export interface DeleteTarget {
  kind: CatalogResourceKind | "policy";
  id: string;
  title?: string;
}

// What goes with each kind, as the catalog agent deletes it.
const CONSEQUENCES: Record<DeleteTarget["kind"], string> = {
  dataset:
    "Its distributions and access policies are deleted with it, and consumers no longer find it in the catalog.",
  distribution: "Consumers can no longer ask for a transfer in this distribution's format.",
  dataService:
    "The agent refuses while a distribution is still served by it; move or delete those first.",
  policy: "Consumers can no longer negotiate the dataset under this offer.",
};

const NOUNS: Record<DeleteTarget["kind"], string> = {
  ...CatalogResources.noun,
  policy: "policy",
};

export function DeleteResourceDialog({
  target,
  onClose,
  onDeleted,
}: {
  target: DeleteTarget | null;
  onClose: () => void;
  onDeleted?: (target: DeleteTarget) => void;
}) {
  const queryClient = useQueryClient();
  const noun = target ? NOUNS[target.kind] : "";
  const { data: mainData } = useGetMainDataService();
  const mainServiceId =
    mainData?.status === 200 ? CatalogViews.dataService(mainData.data as CatalogDoc).id : "";

  const remove = async () => {
    if (!target) return;
    // Offerings fall back to the main data service, so it stays.
    if (target.kind === "dataService" && target.id === mainServiceId) {
      throw new Error("The main data service cannot be deleted.");
    }
    if (target.kind === "policy") await CatalogResources.removePolicy(target.id);
    else await CatalogResources.remove(target.kind, target.id);
    toast.success(`Deleted the ${noun}.`);
    onDeleted?.(target);
    await CatalogResources.invalidate(queryClient);
  };

  return (
    <ConfirmDialog
      open={!!target}
      onOpenChange={(open) => !open && onClose()}
      title={`Delete this ${noun}?`}
      description={
        target && (
          <>
            <span className="font-medium text-ink/90">{target.title || target.id}</span> is deleted
            for good. This cannot be undone.
          </>
        )
      }
      confirmLabel={`Delete ${noun}`}
      onConfirm={remove}
    >
      {target && CONSEQUENCES[target.kind]}
    </ConfirmDialog>
  );
}
