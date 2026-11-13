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

import React, { useMemo, useState } from "react";
import { createFileRoute, Link, useNavigate } from "@tanstack/react-router";
import { Plug, Plus, Sparkles } from "lucide-react";
import {
  Badge,
  Button,
  DataTable,
  FormatDate,
  Input,
  PageHeader,
  PageLayout,
  SegmentedControl,
} from "@eunomia/ui";
import {
  extractInteractionMode,
  extractPrimaryProtocol,
  groupConnectorTemplates,
  type ConnectorTemplateGroup,
} from "shared/src/components/connector/ConnectorSpecView";
import { InstantiateConnectorDialog } from "shared/src/components/connector/InstantiateConnectorDialog";
import { useGetConnectorTemplates } from "shared/src/data/orval/connector-templates/connector-templates";

export const Route = createFileRoute("/my-catalog/connectors/")({
  component: ConnectorTemplatesPage,
});

type ModeFilter = "all" | "PULL" | "PUSH";
type ProtocolFilter = "all" | "HTTP" | "KAFKA";

const MODE_OPTIONS: { value: ModeFilter; label: string }[] = [
  { value: "all", label: "All modes" },
  { value: "PULL", label: "PULL" },
  { value: "PUSH", label: "PUSH" },
];

const PROTOCOL_OPTIONS: { value: ProtocolFilter; label: string }[] = [
  { value: "all", label: "All protocols" },
  { value: "HTTP", label: "HTTP" },
  { value: "KAFKA", label: "KAFKA" },
];

function ConnectorTemplatesPage() {
  const navigate = useNavigate();
  const [mode, setMode] = useState<ModeFilter>("all");
  const [protocol, setProtocol] = useState<ProtocolFilter>("all");
  const [search, setSearch] = useState("");
  const [instantiating, setInstantiating] = useState<{
    name: string;
    version?: string;
  } | null>(null);

  const { data, isLoading } = useGetConnectorTemplates({ limit: 100 });

  const groups = useMemo(() => {
    const items = data?.status === 200 ? data.data.items : [];
    const q = search.trim().toLowerCase();
    return groupConnectorTemplates(items).filter((g) => {
      const m = extractInteractionMode(g.latest.interaction);
      const p = extractPrimaryProtocol(g.latest.interaction);
      if (mode !== "all" && m !== mode) return false;
      if (protocol !== "all" && p !== protocol) return false;
      if (q) {
        const haystack =
          `${g.name} ${g.latest.description ?? ""} ${g.latest.author ?? ""}`.toLowerCase();
        if (!haystack.includes(q)) return false;
      }
      return true;
    });
  }, [data, mode, protocol, search]);

  const filtered = mode !== "all" || protocol !== "all" || !!search.trim();

  return (
    <PageLayout>
      <PageHeader
        eyebrow="My catalog"
        title="Connector templates"
        description="Parameterized HTTP and Kafka blueprints (PULL & PUSH) that instantiate resolved connector bindings on your distributions."
        actions={
          <Button asChild>
            <Link to="/my-catalog/connectors/new">
              <Plus /> New template
            </Link>
          </Button>
        }
      />

      <div className="mb-4 flex flex-wrap items-center gap-2">
        <SegmentedControl value={mode} onChange={setMode} options={MODE_OPTIONS} ariaLabel="Mode" />
        <SegmentedControl
          value={protocol}
          onChange={setProtocol}
          options={PROTOCOL_OPTIONS}
          ariaLabel="Protocol"
        />
        <Input
          placeholder="Search by name, description or author…"
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          onClear={() => setSearch("")}
          containerClassName="w-72"
          className="h-8 text-xs"
        />
      </div>

      {!isLoading && groups.length === 0 && !filtered ? (
        <div className="rounded-lg border border-dashed border-ink/15 px-6 py-12 text-center">
          <Plug className="mx-auto h-10 w-10 text-ink/30" />
          <p className="mt-3 text-sm font-medium">No connector templates yet</p>
          <p className="mx-auto mt-1 max-w-md text-sm text-ink/55">
            Create an HTTP Pull, HTTP Push Webhook, or Kafka Push Stream template to bind data
            sources to your distributions.
          </p>
          <Button asChild size="sm" className="mt-4">
            <Link to="/my-catalog/connectors/new">
              <Plus /> Create first template
            </Link>
          </Button>
        </div>
      ) : (
        <DataTable<ConnectorTemplateGroup>
          data={groups}
          keyExtractor={(g) => g.name}
          hideToolbar
          emptyMessage="No connector template matches the current filters."
          onRowClick={(g) =>
            navigate({
              to: "/my-catalog/connectors/$templateName",
              params: { templateName: g.name },
            })
          }
          columns={[
            {
              header: "Template",
              cell: (g) => (
                <span className="flex flex-col">
                  <span className="font-mono text-sm font-medium">{g.name}</span>
                  {g.latest.description && (
                    <span className="text-xs text-ink/55">{g.latest.description}</span>
                  )}
                </span>
              ),
            },
            {
              header: "Mode",
              cell: (g) => (
                <Badge variant="info">{extractInteractionMode(g.latest.interaction)}</Badge>
              ),
            },
            {
              header: "Protocol",
              cell: (g) => {
                const proto = extractPrimaryProtocol(g.latest.interaction);
                return <Badge variant={proto === "KAFKA" ? "info" : "outline"}>{proto}</Badge>;
              },
            },
            {
              header: "Parameters",
              cell: (g) => (
                <span className="font-mono text-xs text-ink/70">
                  {g.latest.parameters?.length ?? 0}
                </span>
              ),
            },
            {
              header: "Latest",
              cell: (g) => (
                <span className="inline-flex items-center gap-1.5 font-mono text-xs">
                  <span>v{g.latest.version}</span>
                  {g.versions.length > 1 && (
                    <span className="text-ink/45">({g.versions.length} versions)</span>
                  )}
                </span>
              ),
            },
            {
              header: "Created",
              cell: (g) => <FormatDate date={g.latest.createdAt ?? undefined} />,
            },
            {
              header: "",
              className: "text-right",
              cell: (g) => (
                <span onClick={(e: React.MouseEvent) => e.stopPropagation()}>
                  <Button
                    size="xs"
                    variant="outline"
                    onClick={() =>
                      setInstantiating({
                        name: g.name,
                        version: g.latest.version ?? undefined,
                      })
                    }
                  >
                    <Sparkles className="h-3 w-3" /> Instantiate
                  </Button>
                </span>
              ),
            },
          ]}
        />
      )}

      <InstantiateConnectorDialog
        open={!!instantiating}
        onOpenChange={(open) => !open && setInstantiating(null)}
        initialTemplateName={instantiating?.name}
        initialTemplateVersion={instantiating?.version}
      />
    </PageLayout>
  );
}
