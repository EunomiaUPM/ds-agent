// Side panel that edits a dataset, distribution or data service of the local catalog, or
// creates one under a parent. Saves name the version they started from.

import React, { useEffect, useMemo, useState } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { toast } from "sonner";
import { Loader2 } from "lucide-react";
import {
  Sheet,
  SheetContent,
  SheetDescription,
  SheetHeader,
  SheetTitle,
} from "@eunomia/ui";
import { Button } from "@eunomia/ui";
import { ConfirmDialog } from "@eunomia/ui";
import { FormatDate } from "@eunomia/ui";
import { Identifier } from "@eunomia/ui";
import {
  DataServiceFields,
  DatasetFields,
  DistributionFields,
} from "shared/src/components/catalog/DcatFields";
import { DcatForms, type DcatDocument, type FormProblems } from "shared/src/data/dcat";
import { CatalogResources, type CatalogResourceKind } from "shared/src/data/catalogResources";
import { CatalogViews, text } from "shared/src/data/catalog";
import { useGetDataServicesByCatalogId } from "shared/src/data/orval/data-services/data-services";
import { useGetDatasetById } from "shared/src/data/orval/datasets/datasets";

/** What the editor works on: a stored resource, or a new one under `parentId`. */
export interface EditorTarget {
  kind: CatalogResourceKind;
  id?: string;
  parentId?: string;
}

// Each kind's form, the property naming its parent, and how forms map to documents.
const KINDS = {
  dataset: {
    parent: "inCatalog",
    empty: DcatForms.emptyDataset,
    read: DcatForms.readDataset,
    write: DcatForms.datasetDocument,
    problems: DcatForms.datasetProblems,
  },
  distribution: {
    parent: "ofDataset",
    empty: DcatForms.emptyDistribution,
    read: DcatForms.readDistribution,
    write: DcatForms.distributionDocument,
    problems: DcatForms.distributionProblems,
  },
  dataService: {
    parent: "inCatalog",
    empty: DcatForms.emptyDataService,
    read: DcatForms.readDataService,
    write: DcatForms.dataServiceDocument,
    problems: DcatForms.dataServiceProblems,
  },
} as const;

type AnyForm = ReturnType<(typeof KINDS)[CatalogResourceKind]["empty"]>;

export interface CatalogResourceEditorProps {
  target: EditorTarget | null;
  onClose: () => void;
  /** Catalog whose data services a distribution can be served by. */
  catalogId?: string;
  onSaved?: (doc: DcatDocument) => void;
}

export function CatalogResourceEditor({
  target,
  onClose,
  catalogId,
  onSaved,
}: CatalogResourceEditorProps) {
  const queryClient = useQueryClient();
  const [base, setBase] = useState<DcatDocument>({});
  const [etag, setEtag] = useState<string>();
  const [form, setForm] = useState<AnyForm | null>(null);
  const [initial, setInitial] = useState("");
  const [problems, setProblems] = useState<FormProblems>({});
  const [loadError, setLoadError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  const [discarding, setDiscarding] = useState(false);

  const kind = target?.kind;
  const config = kind ? KINDS[kind] : undefined;
  const noun = kind ? CatalogResources.noun[kind] : "";

  const { data: servicesData } = useGetDataServicesByCatalogId(catalogId ?? "", {
    query: { enabled: kind === "distribution" && !!catalogId },
  });
  // A distribution shows the schema it inherits from its dataset.
  const datasetId = kind === "distribution" ? text(base.ofDataset) || target?.parentId || "" : "";
  const { data: datasetData } = useGetDatasetById(datasetId, {
    query: { enabled: !!datasetId },
  });
  const inheritedSchema =
    datasetData?.status === 200 ? text((datasetData.data as DcatDocument).schema) : "";
  const dataServices =
    servicesData?.status === 200
      ? CatalogViews.list(servicesData.data, CatalogViews.dataService)
      : [];

  useEffect(() => {
    if (!target) return;
    const { empty, read, parent } = KINDS[target.kind];
    const start = (doc: DcatDocument, tag: string | undefined, next: AnyForm) => {
      setBase(doc);
      setEtag(tag);
      setForm(next);
      setInitial(JSON.stringify(next));
    };
    setProblems({});
    setLoadError(null);
    setForm(null);
    if (!target.id) {
      start(target.parentId ? { [parent]: target.parentId } : {}, undefined, empty());
      return;
    }
    let live = true;
    CatalogResources.load(target.kind, target.id)
      .then(({ doc, etag: tag }) => live && start(doc, tag, read(doc)))
      .catch((e: Error) => live && setLoadError(e.message));
    return () => {
      live = false;
    };
  }, [target]);

  const dirty = useMemo(() => !!form && JSON.stringify(form) !== initial, [form, initial]);

  const close = () => {
    if (saving) return;
    if (dirty) setDiscarding(true);
    else onClose();
  };

  const save = async () => {
    if (!target || !config || !form) return;
    const found = (config.problems as (f: AnyForm) => FormProblems)(form);
    setProblems(found);
    if (Object.keys(found).length > 0) {
      toast.error("Some fields need a look before saving.");
      return;
    }
    const doc = (config.write as (f: AnyForm, b: DcatDocument) => DcatDocument)(form, base);
    setSaving(true);
    try {
      const stored = target.id
        ? await CatalogResources.save(target.kind, target.id, doc, etag)
        : await CatalogResources.create(target.kind, doc);
      await CatalogResources.invalidate(queryClient);
      toast.success(target.id ? `Saved the ${noun}.` : `Created the ${noun}.`);
      onSaved?.(stored);
      onClose();
    } catch (e) {
      toast.error(e instanceof Error ? e.message : String(e));
    } finally {
      setSaving(false);
    }
  };

  const fields = () => {
    if (!form) return null;
    const props = { onChange: (next: AnyForm) => setForm(next), problems };
    switch (kind) {
      case "dataset":
        return <DatasetFields value={form as never} {...props} />;
      case "distribution":
        return (
          <DistributionFields
            value={form as never}
            dataServices={dataServices}
            inheritedSchema={inheritedSchema}
            {...props}
          />
        );
      case "dataService":
        return <DataServiceFields value={form as never} {...props} />;
      default:
        return null;
    }
  };

  return (
    <>
      <Sheet open={!!target} onOpenChange={(open) => !open && close()}>
        <SheetContent className="flex w-full flex-col gap-0 p-0 sm:max-w-2xl">
          <SheetHeader className="border-b border-ink/10 px-6 py-4 text-left">
            <SheetTitle>{target?.id ? `Edit ${noun}` : `New ${noun}`}</SheetTitle>
            <SheetDescription asChild>
              <div className="flex flex-wrap items-center gap-x-3 gap-y-1 text-xs">
                {target?.id ? (
                  <>
                    <Identifier value={target.id} />
                    {!!base.modified && (
                      <span>
                        Last saved <FormatDate date={text(base.modified)} />
                      </span>
                    )}
                  </>
                ) : (
                  <span>It is published in the catalog as soon as you save it.</span>
                )}
              </div>
            </SheetDescription>
          </SheetHeader>

          <div className="flex-1 overflow-y-auto px-6 py-5">
            {loadError ? (
              <p className="rounded-md border border-danger-700/40 bg-danger-50 px-3 py-2 text-sm text-danger-700 dark:bg-danger-950/40 dark:text-danger-300">
                {loadError}
              </p>
            ) : form ? (
              fields()
            ) : (
              <div className="flex items-center gap-2 text-sm text-muted-foreground">
                <Loader2 className="h-4 w-4 animate-spin" /> Loading…
              </div>
            )}
          </div>

          <div className="flex items-center justify-between gap-3 border-t border-ink/10 px-6 py-3">
            <span className="text-xs text-muted-foreground">
              {dirty ? "Unsaved changes" : target?.id ? "No changes" : ""}
            </span>
            <div className="flex gap-2">
              <Button variant="outline" size="sm" onClick={close} disabled={saving}>
                Cancel
              </Button>
              <Button
                size="sm"
                onClick={save}
                isLoading={saving}
                disabled={!form || (!!target?.id && !dirty)}
              >
                {target?.id ? "Save changes" : `Create ${noun}`}
              </Button>
            </div>
          </div>
        </SheetContent>
      </Sheet>

      <ConfirmDialog
        open={discarding}
        onOpenChange={setDiscarding}
        title="Discard your changes?"
        description={`What you changed in this ${noun} is lost.`}
        confirmLabel="Discard"
        onConfirm={async () => onClose()}
      />
    </>
  );
}
