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

import React, { useState } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { createFileRoute, Link, useNavigate } from "@tanstack/react-router";
import { Copy, GitBranchPlus, History, KeyRound, Sparkles, Trash2 } from "lucide-react";
import { toast } from "sonner";
import {
  Badge,
  Button,
  cn,
  ConfirmDialog,
  FieldGrid,
  FormatDate,
  JsonViewer,
  PageHeader,
  PageLayout,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
} from "@eunomia/ui";
import {
  ConnectorParametersTable,
  ConnectorSpecView,
  extractInteractionMode,
  extractPrimaryProtocol,
  groupConnectorTemplates,
} from "shared/src/components/connector/ConnectorSpecView";
import { ConnectorTemplateModel } from "shared/src/components/connector/connectorTemplateModel";
import { InstantiateConnectorDialog } from "shared/src/components/connector/InstantiateConnectorDialog";
import {
  getGetConnectorTemplateQueryKey,
  getGetConnectorTemplatesQueryKey,
  useDeleteConnectorTemplate,
  useGetConnectorTemplate,
} from "shared/src/data/orval/connector-templates/connector-templates";

interface DetailSearch {
  version?: string;
}

export const Route = createFileRoute("/my-catalog/connectors/$templateName")({
  component: ConnectorTemplateDetailPage,
  validateSearch: (search: Record<string, unknown>): DetailSearch => ({
    version: search.version != null && search.version !== "" ? String(search.version) : undefined,
  }),
});

function ConnectorTemplateDetailPage() {
  const { templateName } = Route.useParams();
  const { version: chosen } = Route.useSearch();
  const navigate = useNavigate();
  const queryClient = useQueryClient();

  const [deleting, setDeleting] = useState(false);
  const [instantiateOpen, setInstantiateOpen] = useState(false);

  const { data: versionsRes, isLoading } = useGetConnectorTemplate(templateName);
  const versions =
    versionsRes?.status === 200
      ? (groupConnectorTemplates(versionsRes.data)[0]?.versions ?? [])
      : [];
  const shown = versions.find((v) => v.version === chosen) ?? versions[0];

  const deleteMutation = useDeleteConnectorTemplate();

  if (!shown) {
    return (
      <PageLayout>
        <p className="py-16 text-center text-sm text-ink/55">
          {isLoading ? "Loading…" : `Connector template "${templateName}" was not found.`}
        </p>
      </PageLayout>
    );
  }

  const mode = extractInteractionMode(shown.interaction);
  const protocol = extractPrimaryProtocol(shown.interaction);
  const builtIns = ConnectorTemplateModel.builtInPlaceholders({
    authentication: shown.authentication,
    interaction: shown.interaction,
    parameters: shown.parameters ?? [],
  });
  const refLabel = `${shown.name}@${shown.version}`;

  const goToVersion = (ver: string) =>
    navigate({
      to: "/my-catalog/connectors/$templateName",
      params: { templateName },
      search: { version: ver },
    });

  const handleDelete = async () => {
    if (!shown.version) return;
    try {
      const res = await deleteMutation.mutateAsync({
        name: templateName,
        version: shown.version,
      });
      if (res.status !== 202) {
        toast.error("Could not delete connector template version.");
        return;
      }
      await queryClient.invalidateQueries({ queryKey: getGetConnectorTemplatesQueryKey() });
      await queryClient.invalidateQueries({
        queryKey: getGetConnectorTemplateQueryKey(templateName),
      });
      toast.success(`Deleted ${templateName} v${shown.version}.`);
      setDeleting(false);
      if (versions.length <= 1) {
        navigate({ to: "/my-catalog/connectors" });
      } else {
        navigate({
          to: "/my-catalog/connectors/$templateName",
          params: { templateName },
          search: {},
        });
      }
    } catch (err) {
      toast.error(err instanceof Error ? err.message : "Failed to delete template version.");
    }
  };

  return (
    <PageLayout>
      <PageHeader
        eyebrow={
          <Link to="/my-catalog/connectors" className="hover:underline">
            Connector templates
          </Link>
        }
        title={shown.name || templateName}
        badge={
          <>
            <Badge variant="info">{mode}</Badge>
            <Badge variant="outline">{protocol}</Badge>
          </>
        }
        description={shown.description || undefined}
        actions={
          <>
            <Select value={shown.version ?? ""} onValueChange={goToVersion}>
              <SelectTrigger className="h-9 w-40" aria-label="Version">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {versions.map((v, i) => (
                  <SelectItem key={v.version ?? String(i)} value={v.version ?? ""}>
                    v{v.version}
                    {i === 0 ? " · latest" : ""}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            <Button onClick={() => setInstantiateOpen(true)}>
              <Sparkles /> Instantiate…
            </Button>
            <Button asChild variant="outline">
              <Link
                to="/my-catalog/connectors/new"
                search={{ from: templateName, version: shown.version ?? undefined }}
              >
                <GitBranchPlus /> New version
              </Link>
            </Button>
            <Button variant="outline_destructive" onClick={() => setDeleting(true)}>
              <Trash2 /> Delete
            </Button>
          </>
        }
      />

      <FieldGrid
        columns={4}
        className="mb-4"
        fields={[
          {
            label: "Reference",
            value: (
              <span className="inline-flex items-center gap-1.5">
                <code className="font-mono text-xs">{refLabel}</code>
                <button
                  type="button"
                  title="Copy reference"
                  onClick={() => {
                    navigator.clipboard.writeText(refLabel);
                    toast.success("Copied reference");
                  }}
                  className="text-ink/40 hover:text-ink"
                >
                  <Copy className="h-3 w-3" />
                </button>
              </span>
            ),
          },
          {
            label: "Author",
            value: shown.author || <span className="text-ink/40">—</span>,
          },
          {
            label: "Parameters",
            value: <span className="font-mono text-xs">{shown.parameters?.length ?? 0}</span>,
          },
          {
            label: "Created",
            value: <FormatDate date={shown.createdAt ?? undefined} />,
          },
        ]}
      />

      <div className="grid grid-cols-1 gap-4 xl:grid-cols-[minmax(0,1fr)_19rem]">
        <div className="min-w-0 space-y-4">
          <section className="rounded-lg border border-ink/10 p-4">
            <h2 className="mb-2 text-sm font-semibold">Specification</h2>
            <Tabs defaultValue="structure">
              <TabsList>
                <TabsTrigger value="structure">Structure</TabsTrigger>
                <TabsTrigger value="json">Raw JSON</TabsTrigger>
              </TabsList>
              <TabsContent value="structure" className="pt-3">
                <ConnectorSpecView
                  interaction={shown.interaction}
                  authentication={shown.authentication}
                />
              </TabsContent>
              <TabsContent value="json" className="pt-3">
                <JsonViewer value={shown} />
              </TabsContent>
            </Tabs>
          </section>

          <section className="rounded-lg border border-ink/10 p-4">
            <div className="mb-2.5 flex items-center gap-2">
              <h2 className="text-sm font-semibold">Declared parameters</h2>
              <span className="rounded-full bg-ink/[0.07] px-2 py-0.5 text-xs text-ink/60">
                {shown.parameters?.length ?? 0}
              </span>
            </div>
            <ConnectorParametersTable parameters={shown.parameters ?? []} />
          </section>
        </div>

        <aside className="space-y-4">
          {builtIns.length > 0 && (
            <section className="rounded-lg border border-ink/10">
              <header className="border-b border-ink/[0.07] px-3 py-2">
                <h2 className="flex items-center gap-2 text-sm font-semibold">
                  <KeyRound className="h-4 w-4 text-ink/45" />
                  Runtime bindings
                  <span className="rounded-full bg-ink/[0.07] px-1.5 text-xs font-normal text-ink/60">
                    {builtIns.length}
                  </span>
                </h2>
              </header>
              <ul className="divide-y divide-ink/[0.06] px-3 py-1">
                {builtIns.map((name) => (
                  <li key={name} className="py-1.5 font-mono text-xs text-ink/80 break-all">
                    {`{{__${name}__}}`}
                  </li>
                ))}
              </ul>
            </section>
          )}

          <section className="rounded-lg border border-ink/10">
            <header className="border-b border-ink/[0.07] px-3 py-2">
              <h2 className="flex items-center gap-2 text-sm font-semibold">
                <History className="h-4 w-4 text-ink/45" />
                Versions
                <span className="rounded-full bg-ink/[0.07] px-1.5 text-xs font-normal text-ink/60">
                  {versions.length}
                </span>
              </h2>
            </header>
            <div className="p-2">
              <ol className="space-y-0.5">
                {versions.map((v, i) => (
                  <li key={v.version ?? String(i)}>
                    <button
                      type="button"
                      onClick={() => v.version && goToVersion(v.version)}
                      className={cn(
                        "flex w-full items-center justify-between rounded-md px-2.5 py-1.5 text-left text-sm hover:bg-ink/[0.05]",
                        v.version === shown.version && "bg-ink/[0.07] font-medium",
                      )}
                    >
                      <span className="font-mono text-xs">
                        v{v.version}
                        {i === 0 && (
                          <span className="ml-2 font-sans text-[11px] font-normal text-ink/45">
                            latest
                          </span>
                        )}
                      </span>
                      <span className="text-xs text-ink/50">
                        <FormatDate date={v.createdAt ?? undefined} format="YYYY-MM-DD" />
                      </span>
                    </button>
                  </li>
                ))}
              </ol>
            </div>
          </section>
        </aside>
      </div>

      <InstantiateConnectorDialog
        open={instantiateOpen}
        onOpenChange={setInstantiateOpen}
        initialTemplateName={shown.name ?? templateName}
        initialTemplateVersion={shown.version ?? undefined}
      />

      <ConfirmDialog
        open={deleting}
        onOpenChange={setDeleting}
        title={`Delete ${templateName} v${shown.version}?`}
        description="Existing resolved connector instances created from this version continue to function independently."
        confirmLabel="Delete"
        onConfirm={handleDelete}
      />
    </PageLayout>
  );
}
