/*
 * Copyright (C) 2026 - Universidad Politécnica de Madrid - UPM
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program. If not, see <https://www.gnu.org/licenses/>.
 */

import React, { useEffect, useMemo, useState } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { Eye, Plus, Sparkles, Trash2 } from "lucide-react";
import { toast } from "sonner";
import {
  Badge,
  Button,
  Checkbox,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  Identifier,
  Input,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  TagInput,
} from "@eunomia/ui";
import { CatalogViews, type CatalogDoc } from "shared/src/data/catalog";
import { useGetMainCatalogs } from "shared/src/data/orval/catalogs/catalogs";
import { useGetDatasetsByCatalogId } from "shared/src/data/orval/datasets/datasets";
import { useGetDistributionsByDatasetId } from "shared/src/data/orval/distributions/distributions";
import {
  getGetConnectorInstanceByDistributionQueryKey,
  useUpsertConnectorInstance,
} from "shared/src/data/orval/connector/connector";
import { useGetConnectorTemplates } from "shared/src/data/orval/connector-templates/connector-templates";
import type {
  ConnectorInstanceDto,
  ConnectorParameterDefinition,
  ConnectorTemplateDto,
} from "shared/src/data/orval/model";
import {
  ConnectorSpecView,
  extractInteractionMode,
  extractPrimaryProtocol,
  groupConnectorTemplates,
} from "./ConnectorSpecView";

export interface InstantiateConnectorDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  /** Pre-selected distribution URN (when opened from a Distribution row or page). */
  distributionId?: string;
  distributionTitle?: string;
  /** Pre-selected template name (when opened from Connector Templates or Replace). */
  initialTemplateName?: string;
  initialTemplateVersion?: string;
  onInstantiated?: (instance: ConnectorInstanceDto) => void;
}

type MapEntry = { key: string; value: string };

function parseInitialParamValue(def: ConnectorParameterDefinition): unknown {
  if (def.defaultValue == null || def.defaultValue === "") {
    switch (def.paramType) {
      case "BOOLEAN":
        return false;
      case "VEC<STRING>":
        return [] as string[];
      case "MAP<STRING,STRING>":
        return [] as MapEntry[];
      default:
        return "";
    }
  }
  const raw = def.defaultValue;
  switch (def.paramType) {
    case "INT": {
      const n = Number(raw);
      return Number.isFinite(n) ? String(n) : "";
    }
    case "BOOLEAN":
      return raw.trim().toLowerCase() === "true";
    case "VEC<STRING>": {
      try {
        const parsed = JSON.parse(raw);
        if (Array.isArray(parsed)) return parsed.map(String);
      } catch {
        // fallback to comma-separated
      }
      return raw
        .split(",")
        .map((s) => s.trim())
        .filter(Boolean);
    }
    case "MAP<STRING,STRING>": {
      try {
        const parsed = JSON.parse(raw);
        if (parsed && typeof parsed === "object" && !Array.isArray(parsed)) {
          return Object.entries(parsed as Record<string, unknown>).map(([key, value]) => ({
            key,
            value: String(value),
          }));
        }
      } catch {
        // ignore invalid default JSON
      }
      return [] as MapEntry[];
    }
    default:
      return raw;
  }
}

function serializeParameterPayload(
  defs: ConnectorParameterDefinition[],
  values: Record<string, unknown>,
): { payload: Record<string, unknown>; missing: string[] } {
  const payload: Record<string, unknown> = {};
  const missing: string[] = [];

  for (const def of defs) {
    const raw = values[def.name];
    switch (def.paramType) {
      case "STRING": {
        const str = typeof raw === "string" ? raw : "";
        if (!str.trim()) {
          if (def.required && def.defaultValue == null) missing.push(def.title || def.name);
        } else {
          payload[def.name] = str;
        }
        break;
      }
      case "INT": {
        const str = typeof raw === "string" ? raw.trim() : String(raw ?? "").trim();
        if (!str) {
          if (def.required && def.defaultValue == null) missing.push(def.title || def.name);
        } else {
          const parsed = Number.parseInt(str, 10);
          if (Number.isNaN(parsed)) {
            missing.push(`${def.title || def.name} (must be integer)`);
          } else {
            payload[def.name] = parsed;
          }
        }
        break;
      }
      case "BOOLEAN": {
        payload[def.name] = Boolean(raw);
        break;
      }
      case "VEC<STRING>": {
        const arr = Array.isArray(raw) ? raw.map(String).filter(Boolean) : [];
        if (arr.length === 0) {
          if (def.required && def.defaultValue == null) missing.push(def.title || def.name);
        } else {
          payload[def.name] = arr;
        }
        break;
      }
      case "MAP<STRING,STRING>": {
        const entries = Array.isArray(raw) ? (raw as MapEntry[]) : [];
        const mapObj: Record<string, string> = {};
        for (const e of entries) {
          if (e.key.trim()) {
            mapObj[e.key.trim()] = e.value;
          }
        }
        if (Object.keys(mapObj).length === 0) {
          if (def.required && def.defaultValue == null) missing.push(def.title || def.name);
        } else {
          payload[def.name] = mapObj;
        }
        break;
      }
    }
  }

  return { payload, missing };
}

function extractApiError(err: unknown, fallback: string): string {
  if (!err) return fallback;
  if (typeof err === "string") return err;
  if (typeof err === "object") {
    const rec = err as Record<string, unknown>;
    if (typeof rec.details === "string" && rec.details) {
      return `${rec.message ?? fallback}: ${rec.details}`;
    }
    if (typeof rec.message === "string" && rec.message) {
      return rec.message;
    }
    if (rec.data && typeof rec.data === "object") {
      return extractApiError(rec.data, fallback);
    }
  }
  return fallback;
}

export function InstantiateConnectorDialog({
  open,
  onOpenChange,
  distributionId: fixedDistributionId,
  distributionTitle,
  initialTemplateName,
  initialTemplateVersion,
  onInstantiated,
}: InstantiateConnectorDialogProps) {
  const queryClient = useQueryClient();

  const { data: templatesRes, isLoading: loadingTemplates } = useGetConnectorTemplates(
    { limit: 100 },
    { query: { enabled: open } },
  );
  const groups = useMemo(() => {
    const items = templatesRes?.status === 200 ? templatesRes.data.items : [];
    return groupConnectorTemplates(items);
  }, [templatesRes]);

  const [selectedName, setSelectedName] = useState<string>(initialTemplateName ?? "");
  const [selectedVersion, setSelectedVersion] = useState<string>(initialTemplateVersion ?? "");
  const [selectedDatasetId, setSelectedDatasetId] = useState<string>("");
  const [selectedDistributionId, setSelectedDistributionId] = useState<string>(
    fixedDistributionId ?? "",
  );
  const [instanceDescription, setInstanceDescription] = useState<string>("");
  const [paramValues, setParamValues] = useState<Record<string, unknown>>({});
  const [previewInstance, setPreviewInstance] = useState<ConnectorInstanceDto | null>(null);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  // Only fetch catalog/datasets/distributions when `distributionId` was NOT passed in.
  const needTargetPicker = open && !fixedDistributionId;
  const { data: mainCatalogData } = useGetMainCatalogs({ query: { enabled: needTargetPicker } });
  const mainCatalog =
    mainCatalogData?.status === 200
      ? CatalogViews.catalog(mainCatalogData.data as CatalogDoc)
      : undefined;
  const { data: datasetsData } = useGetDatasetsByCatalogId(mainCatalog?.id ?? "", {
    query: { enabled: needTargetPicker && !!mainCatalog?.id },
  });
  const datasets = useMemo(
    () =>
      datasetsData?.status === 200
        ? CatalogViews.list(datasetsData.data, CatalogViews.dataset)
        : [],
    [datasetsData],
  );
  const { data: distributionsData } = useGetDistributionsByDatasetId(selectedDatasetId, {
    query: { enabled: needTargetPicker && !!selectedDatasetId },
  });
  const distributions = useMemo(
    () =>
      distributionsData?.status === 200
        ? CatalogViews.list(distributionsData.data, CatalogViews.distribution)
        : [],
    [distributionsData],
  );

  // Synchronise defaults when dialog opens or template selection changes.
  useEffect(() => {
    if (!open) return;
    setErrorMessage(null);
    setPreviewInstance(null);
    setSelectedDistributionId(fixedDistributionId ?? "");
    if (initialTemplateName) {
      setSelectedName(initialTemplateName);
      setSelectedVersion(initialTemplateVersion ?? "");
    } else if (!selectedName && groups.length > 0) {
      setSelectedName(groups[0].name);
      setSelectedVersion(groups[0].latest.version ?? "");
    }
  }, [open, fixedDistributionId, initialTemplateName, initialTemplateVersion, groups.length]);

  useEffect(() => {
    if (!needTargetPicker) return;
    if (!selectedDatasetId && datasets.length > 0) {
      setSelectedDatasetId(datasets[0].id);
    }
  }, [needTargetPicker, datasets, selectedDatasetId]);

  useEffect(() => {
    if (!needTargetPicker) return;
    if (distributions.length > 0 && !distributions.some((d) => d.id === selectedDistributionId)) {
      setSelectedDistributionId(distributions[0].id);
    } else if (distributions.length === 0) {
      setSelectedDistributionId("");
    }
  }, [needTargetPicker, distributions, selectedDistributionId]);

  const activeGroup = useMemo(
    () => groups.find((g) => g.name === selectedName),
    [groups, selectedName],
  );

  useEffect(() => {
    if (!activeGroup) return;
    const exists = activeGroup.versions.some((v) => v.version === selectedVersion);
    if (!exists) {
      setSelectedVersion(activeGroup.latest.version ?? "");
    }
  }, [activeGroup, selectedVersion]);

  const activeTemplate: ConnectorTemplateDto | undefined = useMemo(
    () =>
      activeGroup?.versions.find((v) => v.version === selectedVersion) ?? activeGroup?.latest,
    [activeGroup, selectedVersion],
  );

  useEffect(() => {
    if (!activeTemplate) {
      setParamValues({});
      return;
    }
    const next: Record<string, unknown> = {};
    for (const def of activeTemplate.parameters ?? []) {
      next[def.name] = parseInitialParamValue(def);
    }
    setParamValues(next);
    setPreviewInstance(null);
    setErrorMessage(null);
  }, [activeTemplate?.name, activeTemplate?.version]);

  const upsertMutation = useUpsertConnectorInstance();

  const effectiveDistributionId = fixedDistributionId || selectedDistributionId;

  const submit = async (dryRun: boolean) => {
    setErrorMessage(null);
    if (!activeTemplate?.name || !activeTemplate?.version) {
      setErrorMessage("Select a connector template and version first.");
      return;
    }
    if (!effectiveDistributionId) {
      setErrorMessage("Select a target distribution to bind this connector instance.");
      return;
    }

    const { payload, missing } = serializeParameterPayload(
      activeTemplate.parameters ?? [],
      paramValues,
    );
    if (missing.length > 0) {
      setErrorMessage(`Missing or invalid required parameters: ${missing.join(", ")}`);
      return;
    }

    try {
      const response = await upsertMutation.mutateAsync({
        data: {
          templateName: activeTemplate.name,
          templateVersion: activeTemplate.version,
          distributionId: effectiveDistributionId,
          parameters: payload,
          metadata: instanceDescription.trim()
            ? { description: instanceDescription.trim() }
            : undefined,
          dryRun,
        },
      });

      if (response.status !== 200) {
        setErrorMessage(extractApiError(response.data, "Failed to instantiate connector."));
        return;
      }

      if (dryRun) {
        setPreviewInstance(response.data);
        toast.success("Resolved dry-run preview generated.");
      } else {
        await queryClient.invalidateQueries({
          queryKey: getGetConnectorInstanceByDistributionQueryKey(effectiveDistributionId),
        });
        await queryClient.invalidateQueries({ queryKey: ["/doctor"] });
        toast.success(
          `Bound connector instance (${activeTemplate.name} v${activeTemplate.version}).`,
        );
        onInstantiated?.(response.data);
        onOpenChange(false);
      }
    } catch (err) {
      setErrorMessage(extractApiError(err, "Failed to instantiate connector."));
    }
  };

  const setParam = (name: string, val: unknown) => {
    setPreviewInstance(null);
    setParamValues((prev) => ({ ...prev, [name]: val }));
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-h-[90vh] max-w-3xl overflow-y-auto">
        <DialogHeader>
          <DialogTitle>Instantiate connector from template</DialogTitle>
          <DialogDescription>
            {fixedDistributionId ? (
              <>
                Bind a parameterized connector template to distribution{" "}
                <span className="font-medium text-ink">
                  {distributionTitle || fixedDistributionId}
                </span>
                . Any previous connector instance on this distribution is replaced.
              </>
            ) : (
              "Choose a target distribution and fill in the template parameters to create and bind a resolved connector instance."
            )}
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-5 py-2">
          {/* 1. Target distribution selector when launched from the template library */}
          {!fixedDistributionId && (
            <div className="rounded-lg border border-ink/10 bg-ink/[0.02] p-3.5">
              <p className="mb-2.5 text-xs font-semibold uppercase tracking-wide text-ink/55">
                Target distribution
              </p>
              <div className="grid grid-cols-1 gap-3 sm:grid-cols-2">
                <label className="flex flex-col gap-1">
                  <span className="text-xs font-medium">Dataset</span>
                  <Select value={selectedDatasetId} onValueChange={setSelectedDatasetId}>
                    <SelectTrigger className="h-8 text-xs">
                      <SelectValue placeholder="Select dataset…" />
                    </SelectTrigger>
                    <SelectContent>
                      {datasets.map((ds) => (
                        <SelectItem key={ds.id} value={ds.id}>
                          {ds.title || ds.id}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </label>

                <label className="flex flex-col gap-1">
                  <span className="text-xs font-medium">Distribution</span>
                  <Select
                    value={selectedDistributionId}
                    onValueChange={setSelectedDistributionId}
                    disabled={distributions.length === 0}
                  >
                    <SelectTrigger className="h-8 text-xs">
                      <SelectValue
                        placeholder={
                          distributions.length === 0
                            ? "No distributions in dataset"
                            : "Select distribution…"
                        }
                      />
                    </SelectTrigger>
                    <SelectContent>
                      {distributions.map((dist) => (
                        <SelectItem key={dist.id} value={dist.id}>
                          {dist.title || dist.format || dist.id}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </label>
              </div>
              {selectedDistributionId && (
                <div className="mt-2 text-xs text-ink/55">
                  Distribution ID: <Identifier value={selectedDistributionId} />
                </div>
              )}
            </div>
          )}

          {/* 2. Connector template & version selector */}
          <div className="rounded-lg border border-ink/10 bg-ink/[0.02] p-3.5">
            <p className="mb-2.5 text-xs font-semibold uppercase tracking-wide text-ink/55">
              Connector template
            </p>
            {loadingTemplates ? (
              <p className="text-xs text-ink/50">Loading connector templates…</p>
            ) : groups.length === 0 ? (
              <p className="text-xs text-ink/55">
                No connector templates exist yet. Create one under My area → Connector Templates.
              </p>
            ) : (
              <>
                <div className="grid grid-cols-1 gap-3 sm:grid-cols-3">
                  <label className="flex flex-col gap-1 sm:col-span-2">
                    <span className="text-xs font-medium">Template name</span>
                    <Select value={selectedName} onValueChange={setSelectedName}>
                      <SelectTrigger className="h-8 text-xs">
                        <SelectValue placeholder="Choose template…" />
                      </SelectTrigger>
                      <SelectContent>
                        {groups.map((g) => (
                          <SelectItem key={g.name} value={g.name}>
                            {g.name} ({extractInteractionMode(g.latest.interaction)} ·{" "}
                            {extractPrimaryProtocol(g.latest.interaction)})
                          </SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                  </label>

                  <label className="flex flex-col gap-1">
                    <span className="text-xs font-medium">Version</span>
                    <Select
                      value={selectedVersion}
                      onValueChange={setSelectedVersion}
                      disabled={!activeGroup}
                    >
                      <SelectTrigger className="h-8 text-xs">
                        <SelectValue placeholder="Version" />
                      </SelectTrigger>
                      <SelectContent>
                        {(activeGroup?.versions ?? []).map((v) => (
                          <SelectItem key={v.version ?? "0"} value={v.version ?? ""}>
                            v{v.version}
                          </SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                  </label>
                </div>

                {activeTemplate && (
                  <div className="mt-2.5 flex flex-wrap items-center gap-2 text-xs text-ink/65">
                    <Badge variant="info">
                      {extractInteractionMode(activeTemplate.interaction)}
                    </Badge>
                    <Badge variant="outline">
                      {extractPrimaryProtocol(activeTemplate.interaction)}
                    </Badge>
                    {activeTemplate.author && <span>by {activeTemplate.author}</span>}
                    {activeTemplate.description && (
                      <span className="text-ink/55">— {activeTemplate.description}</span>
                    )}
                  </div>
                )}
              </>
            )}
          </div>

          {/* 3. Dynamic parameter form */}
          {activeTemplate && (
            <div className="space-y-3">
              <div className="flex items-center justify-between">
                <p className="text-xs font-semibold uppercase tracking-wide text-ink/55">
                  Parameters ({activeTemplate.parameters?.length ?? 0})
                </p>
              </div>

              {(activeTemplate.parameters?.length ?? 0) === 0 ? (
                <p className="rounded-md border border-dashed border-ink/15 px-3 py-3 text-xs text-ink/55">
                  This template requires no user-supplied parameters (all values are static or
                  injected at runtime).
                </p>
              ) : (
                <div className="grid grid-cols-1 gap-3 sm:grid-cols-2">
                  {activeTemplate.parameters.map((def) => {
                    const val = paramValues[def.name];
                    const isWide =
                      def.paramType === "VEC<STRING>" || def.paramType === "MAP<STRING,STRING>";
                    return (
                      <div
                        key={def.name}
                        className={`flex flex-col gap-1 ${isWide ? "sm:col-span-2" : ""}`}
                      >
                        <div className="flex items-center gap-1.5 text-xs font-medium">
                          <span>{def.title || def.name}</span>
                          <span className="font-mono text-[10px] text-ink/45">
                            {`{{__${def.name}__}}`} · {def.paramType}
                          </span>
                          {def.required && (
                            <span className="text-[10px] font-semibold text-danger-600">*</span>
                          )}
                        </div>

                        {def.paramType === "BOOLEAN" ? (
                          <span className="flex h-8 items-center gap-2">
                            <Checkbox
                              checked={val === true}
                              onCheckedChange={(c) => setParam(def.name, c === true)}
                            />
                            <span className="text-xs text-ink/65">
                              {val === true ? "true" : "false"}
                            </span>
                          </span>
                        ) : def.paramType === "INT" ? (
                          <Input
                            className="h-8 text-xs"
                            type="number"
                            step={1}
                            placeholder={def.defaultValue ?? "0"}
                            value={typeof val === "string" ? val : String(val ?? "")}
                            onChange={(e) => setParam(def.name, e.target.value)}
                          />
                        ) : def.paramType === "VEC<STRING>" ? (
                          <TagInput
                            value={Array.isArray(val) ? (val as string[]) : []}
                            onChange={(next) => setParam(def.name, next)}
                            placeholder={
                              def.defaultValue ?? "Type item (e.g. redpanda:9092) and press Enter"
                            }
                          />
                        ) : def.paramType === "MAP<STRING,STRING>" ? (
                          <div className="space-y-1.5 rounded-md border border-ink/10 p-2">
                            {(Array.isArray(val) ? (val as MapEntry[]) : []).map((entry, idx) => (
                              <div key={idx} className="flex items-center gap-2">
                                <Input
                                  className="h-7 text-xs font-mono"
                                  placeholder="Key"
                                  value={entry.key}
                                  onChange={(e) => {
                                    const copy = [...(val as MapEntry[])];
                                    copy[idx] = { ...copy[idx], key: e.target.value };
                                    setParam(def.name, copy);
                                  }}
                                />
                                <Input
                                  className="h-7 text-xs font-mono"
                                  placeholder="Value"
                                  value={entry.value}
                                  onChange={(e) => {
                                    const copy = [...(val as MapEntry[])];
                                    copy[idx] = { ...copy[idx], value: e.target.value };
                                    setParam(def.name, copy);
                                  }}
                                />
                                <Button
                                  type="button"
                                  size="icon_sm"
                                  variant="ghost"
                                  onClick={() => {
                                    const copy = (val as MapEntry[]).filter((_, i) => i !== idx);
                                    setParam(def.name, copy);
                                  }}
                                >
                                  <Trash2 className="h-3.5 w-3.5" />
                                </Button>
                              </div>
                            ))}
                            <Button
                              type="button"
                              size="xs"
                              variant="outline"
                              onClick={() =>
                                setParam(def.name, [
                                  ...(Array.isArray(val) ? (val as MapEntry[]) : []),
                                  { key: "", value: "" },
                                ])
                              }
                            >
                              <Plus className="h-3 w-3" /> Add entry
                            </Button>
                          </div>
                        ) : (
                          <Input
                            className="h-8 text-xs"
                            placeholder={def.defaultValue ?? `Enter ${def.title || def.name}…`}
                            value={typeof val === "string" ? val : ""}
                            onChange={(e) => setParam(def.name, e.target.value)}
                          />
                        )}

                        {def.description && (
                          <span className="text-[11px] text-ink/50">{def.description}</span>
                        )}
                      </div>
                    );
                  })}
                </div>
              )}

              <label className="flex flex-col gap-1 pt-1">
                <span className="text-xs font-medium">Instance notes (optional)</span>
                <Input
                  className="h-8 text-xs"
                  placeholder="e.g. Production telemetry feed bound to Redpanda cluster"
                  value={instanceDescription}
                  onChange={(e) => setInstanceDescription(e.target.value)}
                />
              </label>
            </div>
          )}

          {errorMessage && (
            <div className="rounded-md border border-danger-500/30 bg-danger-500/10 px-3 py-2 text-xs text-danger-700 dark:text-danger-300">
              {errorMessage}
            </div>
          )}

          {previewInstance && (
            <div className="rounded-lg border border-brand-sky/30 bg-brand-sky/[0.04] p-3.5">
              <div className="mb-2 flex items-center justify-between">
                <span className="text-xs font-semibold text-brand-sky">
                  Resolved instance preview (dryRun)
                </span>
                <Badge variant="outline">Not saved yet</Badge>
              </div>
              <ConnectorSpecView
                interaction={previewInstance.interaction}
                authentication={previewInstance.authenticationConfig}
              />
            </div>
          )}
        </div>

        <DialogFooter className="gap-2">
          <Button variant="ghost" onClick={() => onOpenChange(false)}>
            Cancel
          </Button>
          <Button
            variant="outline"
            disabled={!activeTemplate || upsertMutation.isPending}
            onClick={() => submit(true)}
          >
            <Eye /> Preview (dryRun)
          </Button>
          <Button
            disabled={!activeTemplate || !effectiveDistributionId || upsertMutation.isPending}
            onClick={() => submit(false)}
          >
            <Sparkles /> Instantiate connector
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
