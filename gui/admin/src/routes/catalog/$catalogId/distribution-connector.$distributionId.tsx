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
import { createFileRoute, Link } from "@tanstack/react-router";
import { FormatDate } from "shared/src/components/ui/format-date";
import { Badge } from "shared/src/components/ui/badge";
import { Button } from "shared/src/components/ui/button";
import {
  Card,
  CardHeader,
  CardTitle,
  CardDescription,
  CardContent,
} from "shared/src/components/ui/card";
import { PageLayout } from "shared/src/components/layout/PageLayout";
import {
  InnerSidebarLayout,
  InnerSidebar,
  InnerBody,
  InnerSidebarSection,
  InnerSidebarNav,
  InnerSidebarNavItem,
} from "shared/src/components/layout/InnerSidebarLayout";
import {
  useGetConnectorInstanceByDistribution,
  getGetConnectorInstanceByDistributionQueryOptions,
} from "shared/src/data/orval/connector/connector";
import {
  useGetDistributionById,
  getGetDistributionByIdQueryOptions,
} from "shared/src/data/orval/distributions/distributions";
import { useGetCatalogById } from "shared/src/data/orval/catalogs/catalogs";
import { formatUrn } from "shared/src/lib/utils";
import {
  ConnectorInstanceDto,
  PullLifecycle,
  PushLifecycle,
} from "shared/src/data/orval/model";
import { CopyButton } from "@/components/dataplane/CopyButton";
import Heading from "shared/components/ui/heading";
import {
  ArrowLeft,
  ArrowRight,
  Box,
  Calendar,
  Check,
  Code2,
  Copy,
  Cpu,
  Database,
  ExternalLink,
  Eye,
  EyeOff,
  FileCode,
  Info,
  Key,
  Layers,
  Link2,
  Lock,
  RefreshCw,
  Server,
  Shield,
  ShieldCheck,
  Sparkles,
  Tag,
  User,
  Workflow,
} from "lucide-react";

// =============================================================================
// HELPERS
// =============================================================================

const METHOD_COLORS: Record<string, string> = {
  GET: "bg-emerald-500/10 text-emerald-700 dark:text-emerald-400 border-emerald-500/30",
  POST: "bg-sky-500/10 text-sky-700 dark:text-sky-400 border-sky-500/30",
  PUT: "bg-amber-500/10 text-amber-700 dark:text-amber-400 border-amber-500/30",
  PATCH: "bg-amber-500/10 text-amber-700 dark:text-amber-400 border-amber-500/30",
  DELETE: "bg-rose-500/10 text-rose-700 dark:text-rose-400 border-rose-500/30",
};

const AUTH_LABELS: Record<string, string> = {
  NO_AUTH: "No Auth (Public)",
  BASIC_AUTH: "HTTP Basic Auth",
  BEARER_TOKEN: "Bearer Token",
  API_KEY: "API Key",
  OAUTH2: "OAuth 2.0 Flow",
};

function MethodBadge({ method }: { method?: string | string[] }) {
  if (!method) return <span className="text-muted-foreground text-xs">—</span>;
  const methods = Array.isArray(method) ? method : [method];

  return (
    <div className="flex flex-wrap gap-1">
      {methods.map((m) => {
        const key = String(m).toUpperCase();
        const classes = METHOD_COLORS[key] ?? "bg-ink/5 text-foreground/80 border-ink/10";
        return (
          <Badge key={key} variant="outline" className={`font-mono text-xs font-semibold ${classes}`}>
            {key}
          </Badge>
        );
      })}
    </div>
  );
}

function SecretValueDisplay({
  secret,
  label,
}: {
  secret?: unknown;
  label: string;
}) {
  const [revealed, setRevealed] = useState(false);

  if (!secret) {
    return <span className="text-muted-foreground text-xs italic">Not configured</span>;
  }

  // If secret is an object with source metadata (SecretString)
  if (typeof secret === "object" && secret !== null) {
    const s = secret as Record<string, any>;
    const type = s.type || (s.source && s.source.type);

    if (type === "VAULT_REF" || s.path) {
      const path = s.path || (s.content && s.content.path) || "vault/secret";
      const key = s.key || (s.content && s.content.key) || "key";
      return (
        <div className="flex items-center gap-2 font-mono text-xs">
          <Badge variant="outline" className="border-amber-500/40 text-amber-700 dark:text-amber-400 text-xs">
            VAULT REF
          </Badge>
          <span className="text-foreground/80">
            {String(path)} <span className="text-muted-foreground">#</span> {String(key)}
          </span>
          <CopyButton text={`${path}#${key}`} />
        </div>
      );
    }

    if (type === "ENV_VAR") {
      const varName = s.content || s.name || "ENV_VAR";
      return (
        <div className="flex items-center gap-2 font-mono text-xs">
          <Badge variant="outline" className="border-sky-500/40 text-sky-700 dark:text-sky-400 text-xs">
            ENV VAR
          </Badge>
          <span className="text-brand-sky">${String(varName)}</span>
          <CopyButton text={String(varName)} />
        </div>
      );
    }

    if (type === "PLAIN" || type === "BASE64" || s.content) {
      const raw = String(s.content ?? "");
      return (
        <div className="flex items-center gap-2 font-mono text-xs">
          {type === "BASE64" && (
            <Badge variant="outline" className="border-purple-500/40 text-purple-700 dark:text-purple-400 text-xs">
              BASE64
            </Badge>
          )}
          <span className="text-foreground">
            {revealed ? raw : "••••••••••••••••"}
          </span>
          <Button
            variant="ghost"
            size="icon"
            className="h-6 w-6 text-muted-foreground hover:text-foreground"
            onClick={() => setRevealed(!revealed)}
            title={revealed ? "Hide secret" : "Reveal secret"}
          >
            {revealed ? <EyeOff className="h-3 w-3" /> : <Eye className="h-3 w-3" />}
          </Button>
          <CopyButton text={raw} />
        </div>
      );
    }
  }

  // If secret is just a string or primitive
  const text = String(secret);
  return (
    <div className="flex items-center gap-2 font-mono text-xs">
      <span className="text-foreground">
        {revealed ? text : "••••••••••••••••"}
      </span>
      <Button
        variant="ghost"
        size="icon"
        className="h-6 w-6 text-muted-foreground hover:text-foreground"
        onClick={() => setRevealed(!revealed)}
        title={revealed ? "Hide secret" : "Reveal secret"}
      >
        {revealed ? <EyeOff className="h-3 w-3" /> : <Eye className="h-3 w-3" />}
      </Button>
      <CopyButton text={text} />
    </div>
  );
}

function ProtocolSpecCard({
  title,
  spec,
}: {
  title: string;
  spec?: Record<string, any>;
}) {
  if (!spec) return null;

  const protocol = (spec.protocol as string) || "HTTP";
  const isHttp = protocol.toUpperCase() === "HTTP" || protocol.toUpperCase() === "HTTPS";
  const isKafka = protocol.toUpperCase() === "KAFKA";

  // HTTP fields
  const urlTemplate = spec.urlTemplate || spec.accessUrl || spec.url;
  const method = spec.method;
  const headers = spec.headers as Record<string, string> | undefined;
  const bodyTemplate = spec.bodyTemplate;

  // Kafka fields
  const brokers = spec.brokers as string[] | undefined;
  const topic = spec.topic as string | undefined;
  const groupId = spec.groupId as string | undefined;

  return (
    <Card className="border-ink/10 bg-background-800/40">
      <CardHeader className="pb-3 border-b border-ink/5 flex flex-row items-center justify-between">
        <div>
          <CardTitle className="text-sm font-semibold flex items-center gap-2">
            <Server className="h-4 w-4 text-brand-sky" />
            {title}
          </CardTitle>
          <CardDescription className="text-xs">
            Transport protocol specification configured for dataplane execution.
          </CardDescription>
        </div>
        <Badge
          variant="outline"
          className={
            isKafka
              ? "border-orange-500/40 text-orange-700 dark:text-orange-400 font-mono text-xs"
              : "border-sky-500/40 text-sky-700 dark:text-sky-400 font-mono text-xs"
          }
        >
          {protocol.toUpperCase()}
        </Badge>
      </CardHeader>

      <CardContent className="pt-4 space-y-4 text-xs">
        {/* HTTP Specification */}
        {isHttp && (
          <>
            <div className="space-y-1.5">
              <span className="text-muted-foreground text-xs uppercase tracking-wider font-semibold block">
                Target Endpoint URL Template
              </span>
              <div className="flex items-center justify-between gap-2 p-2.5 rounded-lg bg-sunken/30 border border-ink/5 font-mono text-xs">
                <span className="text-brand-sky break-all">{urlTemplate || "—"}</span>
                {urlTemplate && <CopyButton text={urlTemplate} />}
              </div>
            </div>

            <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
              <div className="space-y-1.5">
                <span className="text-muted-foreground text-xs uppercase tracking-wider font-semibold block">
                  Allowed HTTP Method(s)
                </span>
                <MethodBadge method={method} />
              </div>

              <div className="space-y-1.5">
                <span className="text-muted-foreground text-xs uppercase tracking-wider font-semibold block">
                  Transport Security
                </span>
                <Badge variant="outline" className="border-emerald-500/30 text-emerald-700 dark:text-emerald-400 text-xs">
                  {urlTemplate?.startsWith("https") ? "TLS Encrypted (HTTPS)" : "Plaintext (HTTP)"}
                </Badge>
              </div>
            </div>

            {/* Headers */}
            {headers && typeof headers === "object" && Object.keys(headers).length > 0 && (
              <div className="space-y-1.5">
                <span className="text-muted-foreground text-xs uppercase tracking-wider font-semibold block">
                  Custom Request Headers ({Object.keys(headers).length})
                </span>
                <div className="rounded-lg border border-ink/5 bg-sunken/20 divide-y divide-ink/5">
                  {Object.entries(headers).map(([k, v]) => (
                    <div key={k} className="flex items-center justify-between p-2 font-mono text-xs">
                      <span className="text-muted-foreground">{k}</span>
                      <span className="text-foreground">{String(v)}</span>
                    </div>
                  ))}
                </div>
              </div>
            )}

            {/* Body Template */}
            {bodyTemplate && (
              <div className="space-y-1.5">
                <span className="text-muted-foreground text-xs uppercase tracking-wider font-semibold block">
                  Body Template Payload
                </span>
                <pre className="p-3 rounded-lg bg-sunken/40 border border-ink/5 font-mono text-xs text-brand-sky overflow-x-auto">
                  {typeof bodyTemplate === "object"
                    ? JSON.stringify(bodyTemplate, null, 2)
                    : String(bodyTemplate)}
                </pre>
              </div>
            )}
          </>
        )}

        {/* Kafka Specification */}
        {isKafka && (
          <div className="space-y-3">
            {topic && (
              <div className="flex items-center justify-between p-2 rounded-lg bg-sunken/20 border border-ink/5">
                <span className="text-muted-foreground">Kafka Topic:</span>
                <span className="font-mono text-orange-700 dark:text-orange-400 font-semibold">{topic}</span>
              </div>
            )}
            {groupId && (
              <div className="flex items-center justify-between p-2 rounded-lg bg-sunken/20 border border-ink/5">
                <span className="text-muted-foreground">Consumer Group:</span>
                <span className="font-mono text-foreground">{groupId}</span>
              </div>
            )}
            {brokers && brokers.length > 0 && (
              <div className="space-y-1.5">
                <span className="text-muted-foreground text-xs uppercase tracking-wider font-semibold block">
                  Cluster Brokers
                </span>
                <div className="flex flex-wrap gap-2">
                  {brokers.map((b) => (
                    <Badge key={b} variant="outline" className="font-mono text-xs border-ink/10">
                      {b}
                    </Badge>
                  ))}
                </div>
              </div>
            )}
          </div>
        )}
      </CardContent>
    </Card>
  );
}

// =============================================================================
// ROUTE COMPONENT
// =============================================================================

type ConnectorTab = "interaction" | "authentication" | "metadata";

function RouteComponent() {
  const { catalogId, distributionId } = Route.useParams();
  const [activeTab, setActiveTab] = useState<ConnectorTab>("interaction");
  const [copiedId, setCopiedId] = useState(false);

  const { data: distributionData, refetch: refetchDistribution } =
    useGetDistributionById(distributionId);
  const { data: connectorData, refetch: refetchConnector } =
    useGetConnectorInstanceByDistribution(distributionId);
  const { data: catalogData } = useGetCatalogById(catalogId);

  const distribution = distributionData?.status === 200 ? distributionData.data : undefined;
  const connector =
    connectorData?.status === 200 ? (connectorData.data as ConnectorInstanceDto) : undefined;
  const catalog = catalogData?.status === 200 ? catalogData.data : undefined;

  const handleRefresh = () => {
    refetchDistribution();
    refetchConnector();
  };

  const handleCopyId = () => {
    if (!connector?.id) return;
    navigator.clipboard.writeText(connector.id);
    setCopiedId(true);
    setTimeout(() => setCopiedId(false), 2000);
  };

  const interaction = connector?.interaction;
  const mode = interaction?.mode || "PULL";
  const isPush = mode === "PUSH";
  const dataAccess =
    !isPush && interaction && "dataAccess" in interaction
      ? (interaction.dataAccess as Record<string, any>)
      : undefined;
  const subscribe =
    isPush && interaction && "subscribe" in interaction
      ? (interaction.subscribe as Record<string, any>)
      : undefined;
  const unsubscribe =
    isPush && interaction && "unsubscribe" in interaction
      ? (interaction.unsubscribe as Record<string, any>)
      : undefined;

  const auth = connector?.authenticationConfig as Record<string, any> | undefined;
  const authType = auth?.type || "NO_AUTH";
  const authLabel = AUTH_LABELS[authType] || authType;

  return (
    <PageLayout>
      <InnerSidebarLayout>
        {/* Secondary Left Sidebar */}
        <InnerSidebar>
          {/* Card 1: Connector Instance (1:1 Association) */}
          <Card className="border-ink/10 bg-background-800/40">
            <CardHeader className="pb-3 space-y-2">
              <div className="flex items-center justify-between">
                <Badge variant="outline" className="border-brand-sky/40 text-brand-sky text-xs font-semibold">
                  1:1 Instance Binding
                </Badge>
                <div className="flex items-center gap-1.5 font-mono text-xs text-emerald-700 dark:text-emerald-400">
                  <span className="h-1.5 w-1.5 rounded-full bg-emerald-400 animate-pulse" />
                  <span>Active</span>
                </div>
              </div>

              <div>
                <CardTitle className="text-base font-semibold leading-tight flex items-center gap-2">
                  <Server className="h-4 w-4 text-brand-sky flex-shrink-0" />
                  Connector Instance
                </CardTitle>
                <CardDescription className="text-xs mt-1">
                  Concrete dataplane runtime entity bound 1:1 to distribution.
                </CardDescription>
              </div>
            </CardHeader>

            <CardContent className="space-y-3 pt-0 text-xs">
              {connector?.id && (
                <div className="p-2.5 rounded-lg bg-sunken/20 border border-ink/5 space-y-1 font-mono text-xs">
                  <div className="flex items-center justify-between text-muted-foreground">
                    <span className="font-semibold uppercase tracking-wider text-xs">Instance URN (1:1):</span>
                    <CopyButton text={connector.id} />
                  </div>
                  <span className="text-brand-sky break-all block">{formatUrn(connector.id)}</span>
                </div>
              )}

              {/* 1:1 Associated Distribution info */}
              <div className="pt-2 border-t border-ink/10 space-y-2">
                <span className="text-xs text-muted-foreground font-semibold uppercase tracking-wider block">
                  1:1 Associated Distribution
                </span>
                <div className="p-2.5 rounded-lg bg-ink/5 border border-ink/5 space-y-1.5">
                  <div className="flex items-center justify-between gap-1">
                    <p className="font-semibold text-xs text-foreground truncate">
                      {distribution?.dctTitle || "Distribution"}
                    </p>
                    {distribution?.dctFormat && (
                      <Badge variant="code" className="text-xs font-mono flex-shrink-0">
                        {distribution.dctFormat}
                      </Badge>
                    )}
                  </div>
                  <div className="flex items-center justify-between gap-1 font-mono text-xs text-muted-foreground">
                    <span className="truncate">{formatUrn(distributionId)}</span>
                    <CopyButton text={distributionId} />
                  </div>
                  {distribution?.dcatAccessService && (
                    <Link
                      to="/catalog/$catalogId/data-service/$dataServiceId"
                      params={{
                        catalogId,
                        dataServiceId: distribution.dcatAccessService,
                      }}
                      className="inline-flex items-center gap-1 text-emerald-700 dark:text-emerald-400 hover:underline font-mono text-xs pt-1"
                    >
                      <Server className="h-3 w-3" />
                      <span>Access Data Service</span>
                    </Link>
                  )}
                </div>
              </div>

              {/* Parent Dataset Context (if available) */}
              {distribution?.datasetId && (
                <div className="pt-2 border-t border-ink/10 space-y-1">
                  <span className="text-xs text-muted-foreground font-semibold uppercase tracking-wider block">
                    Parent Dataset Context
                  </span>
                  <Link
                    to="/catalog/$catalogId/dataset/$datasetId"
                    params={{
                      catalogId,
                      datasetId: distribution.datasetId,
                    }}
                    className="flex items-center gap-2 p-2 rounded-lg bg-ink/5 border border-ink/5 hover:bg-ink/10 transition-colors group"
                  >
                    <Box className="h-3.5 w-3.5 text-primary flex-shrink-0" />
                    <span className="font-mono text-xs truncate group-hover:underline text-foreground">
                      {formatUrn(distribution.datasetId)}
                    </span>
                    <ArrowRight className="h-3 w-3 ml-auto opacity-60 group-hover:opacity-100 transition-opacity" />
                  </Link>
                </div>
              )}

              {/* Quick Status Tags */}
              <div className="pt-2 border-t border-ink/10 space-y-1.5 text-xs">
                <div className="flex justify-between items-center">
                  <span className="text-muted-foreground">Lifecycle Mode:</span>
                  <Badge
                    variant="outline"
                    className={
                      isPush
                        ? "border-orange-500/40 text-orange-700 dark:text-orange-400 font-mono text-xs"
                        : "border-sky-500/40 text-sky-700 dark:text-sky-400 font-mono text-xs"
                    }
                  >
                    {mode}
                  </Badge>
                </div>
                <div className="flex justify-between items-center">
                  <span className="text-muted-foreground">Auth Strategy:</span>
                  <Badge variant="outline" className="border-emerald-500/30 text-emerald-700 dark:text-emerald-400 font-mono text-xs">
                    {authLabel}
                  </Badge>
                </div>
              </div>
            </CardContent>
          </Card>

          {/* Card 2: Origin Connector Template (Archetype) */}
          <Card className="border-purple-500/20 bg-purple-500/5">
            <CardHeader className="pb-3 space-y-1.5">
              <div className="flex items-center justify-between">
                <Badge variant="outline" className="border-purple-500/40 text-purple-700 dark:text-purple-300 text-xs">
                  Origin Template Blueprint
                </Badge>
                <Badge variant="outline" className="font-mono text-xs border-ink/10">
                  v{connector?.version || "1.0.0"}
                </Badge>
              </div>
              <CardTitle className="text-sm font-semibold flex items-center gap-2 text-purple-800 dark:text-purple-200">
                <Cpu className="h-3.5 w-3.5 text-purple-700 dark:text-purple-400" />
                {connector?.name || "Template Archetype"}
              </CardTitle>
              <CardDescription className="text-xs text-purple-800 dark:text-purple-200/70 line-clamp-3">
                {connector?.description || "Reusable blueprint defining parameter schemas and transport specs."}
              </CardDescription>
            </CardHeader>

            <CardContent className="space-y-2 pt-0 text-xs">
              {connector?.author && (
                <div className="flex justify-between items-center">
                  <span className="text-muted-foreground flex items-center gap-1">
                    <User className="h-3 w-3 opacity-60" />
                    Author:
                  </span>
                  <span className="text-foreground/80">{connector.author}</span>
                </div>
              )}

              {connector?.createdAt && (
                <div className="flex justify-between items-center">
                  <span className="text-muted-foreground flex items-center gap-1">
                    <Calendar className="h-3 w-3 opacity-60" />
                    Created:
                  </span>
                  <FormatDate date={connector.createdAt} />
                </div>
              )}

              {/* Educational Note */}
              <div className="p-2.5 rounded-lg bg-sunken/30 border border-purple-500/20 text-xs text-purple-800 dark:text-purple-200/80 space-y-1">
                <div className="flex items-center gap-1 font-semibold text-purple-700 dark:text-purple-300">
                  <Info className="h-3 w-3 flex-shrink-0" />
                  <span>Template vs. Instance</span>
                </div>
                <p className="leading-relaxed">
                  The Template is the reusable blueprint with parameters. The Instance is the concrete entity bound 1:1 to this Distribution with resolved values.
                </p>
              </div>
            </CardContent>
          </Card>

          {/* Card 3: Catalog Context */}
          <Card className="border-ink/10 bg-background-800/40">
            <CardContent className="p-3">
              <span className="text-xs text-muted-foreground font-semibold uppercase tracking-wider block mb-1.5">
                Catalog Context
              </span>
              <Link
                to="/catalog/$catalogId"
                params={{ catalogId }}
                className="flex items-center gap-2 p-2 rounded-lg bg-ink/5 border border-ink/5 hover:bg-ink/10 transition-colors group"
              >
                <Database className="h-3.5 w-3.5 text-brand-sky flex-shrink-0" />
                <span className="font-medium text-xs truncate group-hover:underline">
                  {catalog?.dctTitle || formatUrn(catalogId)}
                </span>
                <ArrowRight className="h-3 w-3 ml-auto opacity-60 group-hover:opacity-100 transition-opacity" />
              </Link>
            </CardContent>
          </Card>

          {/* Navigation Section */}
          <InnerSidebarSection title="Configuration & Specs">
            <InnerSidebarNav>
              <InnerSidebarNavItem
                icon={Layers}
                label="Interaction & Protocol"
                count={mode}
                active={activeTab === "interaction"}
                onClick={() => setActiveTab("interaction")}
              />
              <InnerSidebarNavItem
                icon={Lock}
                label="Upstream Authentication"
                count={authType}
                active={activeTab === "authentication"}
                onClick={() => setActiveTab("authentication")}
              />
              <InnerSidebarNavItem
                icon={Workflow}
                label="1:1 Architecture & Specs"
                active={activeTab === "metadata"}
                onClick={() => setActiveTab("metadata")}
              />
            </InnerSidebarNav>
          </InnerSidebarSection>
        </InnerSidebar>

        {/* Main Content Body */}
        <InnerBody>
          {/* Top Header & Breadcrumb */}
          <div className="space-y-3 pb-4 border-b border-ink/10">
            <div className="flex items-center gap-2 text-xs text-muted-foreground flex-wrap">
              <Link
                to="/catalog/$catalogId"
                params={{ catalogId }}
                className="hover:text-foreground transition-colors flex items-center gap-1"
              >
                <ArrowLeft className="h-3.5 w-3.5" />
                <span>{catalog?.dctTitle || "Catalog"}</span>
              </Link>

              {distribution?.datasetId && (
                <>
                  <span>/</span>
                  <Link
                    to="/catalog/$catalogId/dataset/$datasetId"
                    params={{
                      catalogId,
                      datasetId: distribution.datasetId,
                    }}
                    className="hover:text-foreground transition-colors flex items-center gap-1"
                  >
                    <Box className="h-3 w-3" />
                    <span>Dataset</span>
                  </Link>
                </>
              )}

              <span>/</span>
              <span className="text-foreground font-medium">Connector Instance</span>
            </div>

            <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
              <div>
                <div className="flex items-center gap-2 flex-wrap">
                  <Heading level="h2" className="!mb-0">
                    Connector Instance
                  </Heading>
                  <Badge
                    variant="outline"
                    className="border-brand-sky/40 text-brand-sky font-mono text-xs font-semibold"
                  >
                    1:1 Distribution Association
                  </Badge>
                  <Badge
                    variant="outline"
                    className="border-purple-500/40 text-purple-700 dark:text-purple-300 font-mono text-xs"
                  >
                    Blueprint: {connector?.name || "Template"} v{connector?.version || "1.0.0"}
                  </Badge>
                  <Badge
                    variant="outline"
                    className={
                      isPush
                        ? "border-orange-500/40 text-orange-700 dark:text-orange-400 font-mono text-xs"
                        : "border-sky-500/40 text-sky-700 dark:text-sky-400 font-mono text-xs"
                    }
                  >
                    {mode} MODE
                  </Badge>
                  <Badge variant="outline" className="border-emerald-500/30 text-emerald-700 dark:text-emerald-400 text-xs">
                    {authLabel}
                  </Badge>
                </div>
                <p className="text-xs text-muted-foreground mt-1">
                  Concrete runtime configuration bound 1:1 to distribution:{" "}
                  <span className="font-mono text-foreground font-medium">
                    {distribution?.dctTitle || distributionId}
                  </span>
                </p>
              </div>

              <div className="flex items-center gap-2 flex-wrap">
                {distribution?.datasetId && (
                  <Link
                    to="/catalog/$catalogId/dataset/$datasetId"
                    params={{
                      catalogId,
                      datasetId: distribution.datasetId,
                    }}
                    className="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg border border-ink/10 bg-ink/5 hover:bg-ink/10 text-xs text-foreground transition-colors"
                  >
                    <Box className="h-3.5 w-3.5 text-brand-sky" />
                    <span>View Dataset</span>
                  </Link>
                )}
                <Button
                  variant="outline"
                  size="sm"
                  onClick={handleRefresh}
                  className="gap-1.5 text-xs"
                >
                  <RefreshCw className="h-3.5 w-3.5" />
                  Refresh
                </Button>
              </div>
            </div>
          </div>

          {/* TAB 1: INTERACTION & TRANSPORT */}
          {activeTab === "interaction" && (
            <div className="space-y-6">
              {/* Interaction Mode Overview */}
              <div className="p-4 rounded-xl border border-ink/10 bg-background-800/40 flex items-start gap-4">
                <div className="p-2.5 rounded-lg bg-primary/10 text-primary border border-primary/20 shrink-0">
                  <Layers className="h-5 w-5" />
                </div>
                <div className="space-y-1">
                  <div className="flex items-center gap-2">
                    <p className="font-semibold text-sm text-foreground">
                      Lifecycle Mode: {mode} (Dataplane Execution Specification)
                    </p>
                    <Badge variant="outline" className="border-brand-sky/40 text-brand-sky text-xs font-mono">
                      1:1 Bound
                    </Badge>
                  </div>
                  <p className="text-xs text-muted-foreground">
                    {isPush
                      ? "Push interaction registers a webhook or callback subscription with the upstream service to receive real-time events."
                      : "Pull interaction fetches data directly on demand from the upstream resource endpoint during transfer sessions."}
                    {" All template placeholders (e.g. {{__PARAM__}}) have been resolved into concrete instance parameters."}
                  </p>
                </div>
              </div>

              {/* Protocol Spec Cards */}
              {!isPush && dataAccess && (
                <ProtocolSpecCard
                  title="Data Access Specification"
                  spec={dataAccess}
                />
              )}

              {isPush && (
                <>
                  {subscribe && (
                    <ProtocolSpecCard
                      title="Subscribe Lifecycle Step"
                      spec={subscribe}
                    />
                  )}
                  {unsubscribe && (
                    <ProtocolSpecCard
                      title="Unsubscribe Lifecycle Step"
                      spec={unsubscribe}
                    />
                  )}
                </>
              )}

              {!connector && (
                <Card className="border-dashed border-ink/10 p-8 text-center bg-transparent">
                  <Server className="h-8 w-8 text-muted-foreground/50 mx-auto mb-2" />
                  <p className="text-sm font-medium text-foreground">No connector instance found</p>
                  <p className="text-xs text-muted-foreground mt-1">
                    No technical connector instance is bound to distribution {distributionId}.
                  </p>
                </Card>
              )}
            </div>
          )}

          {/* TAB 2: AUTHENTICATION */}
          {activeTab === "authentication" && (
            <div className="space-y-6">
              <div className="p-4 rounded-xl border border-ink/10 bg-background-800/40 flex items-start gap-4">
                <div className="p-2.5 rounded-lg bg-emerald-500/10 text-emerald-700 dark:text-emerald-400 border border-emerald-500/20 shrink-0">
                  <ShieldCheck className="h-5 w-5" />
                </div>
                <div className="space-y-1">
                  <div className="flex items-center gap-2">
                    <p className="font-semibold text-sm text-foreground">
                      Resolved Upstream Authentication: {authLabel}
                    </p>
                    <Badge variant="outline" className="border-emerald-500/40 text-emerald-700 dark:text-emerald-400 text-xs font-mono">
                      Instance Credentials
                    </Badge>
                  </div>
                  <p className="text-xs text-muted-foreground">
                    Credentials configured specifically for this 1:1 instance. Sensitive tokens, passwords, and API keys are securely stored and injected into the dataplane transfer session.
                  </p>
                </div>
              </div>

              <Card className="border-ink/10 bg-background-800/40">
                <CardHeader className="pb-3 border-b border-ink/5">
                  <div className="flex items-center justify-between">
                    <CardTitle className="text-sm font-semibold flex items-center gap-2">
                      <Lock className="h-4 w-4 text-brand-sky" />
                      Upstream Authentication Configuration
                    </CardTitle>
                    <Badge variant="outline" className="border-brand-sky/30 text-brand-sky text-xs">
                      {authLabel}
                    </Badge>
                  </div>
                  <CardDescription className="text-xs">
                    Credentials used by the dataplane to access or subscribe to the upstream data service.
                  </CardDescription>
                </CardHeader>

                <CardContent className="pt-4 space-y-4 text-xs">
                  {/* NO AUTH */}
                  {authType === "NO_AUTH" && (
                    <div className="p-4 rounded-lg bg-sunken/20 border border-ink/5 flex items-center gap-3">
                      <ShieldCheck className="h-5 w-5 text-emerald-700 dark:text-emerald-400 shrink-0" />
                      <div>
                        <p className="font-semibold text-foreground">Public / Unauthenticated Access</p>
                        <p className="text-xs text-muted-foreground mt-0.5">
                          The remote data service endpoint does not require credentials. Requests are dispatched without authorization headers.
                        </p>
                      </div>
                    </div>
                  )}

                  {/* BASIC AUTH */}
                  {authType === "BASIC_AUTH" && (
                    <div className="rounded-lg border border-ink/10 divide-y divide-ink/5">
                      <div className="flex items-center justify-between p-3">
                        <span className="text-muted-foreground font-medium">Username</span>
                        <span className="font-mono text-foreground font-semibold">
                          {auth?.username || "—"}
                        </span>
                      </div>
                      <div className="flex items-center justify-between p-3">
                        <span className="text-muted-foreground font-medium">Password</span>
                        <SecretValueDisplay secret={auth?.password} label="Password" />
                      </div>
                    </div>
                  )}

                  {/* BEARER TOKEN */}
                  {authType === "BEARER_TOKEN" && (
                    <div className="rounded-lg border border-ink/10 divide-y divide-ink/5">
                      <div className="flex items-center justify-between p-3">
                        <span className="text-muted-foreground font-medium">Bearer Token</span>
                        <SecretValueDisplay secret={auth?.token} label="Token" />
                      </div>
                    </div>
                  )}

                  {/* API KEY */}
                  {authType === "API_KEY" && (
                    <div className="rounded-lg border border-ink/10 divide-y divide-ink/5">
                      <div className="flex items-center justify-between p-3">
                        <span className="text-muted-foreground font-medium">Header / Parameter Key</span>
                        <Badge variant="code" className="font-mono text-xs">
                          {auth?.key || "X-API-Key"}
                        </Badge>
                      </div>
                      <div className="flex items-center justify-between p-3">
                        <span className="text-muted-foreground font-medium">Location</span>
                        <Badge variant="outline" className="border-ink/10 text-xs">
                          {auth?.location || "HEADER"}
                        </Badge>
                      </div>
                      <div className="flex items-center justify-between p-3">
                        <span className="text-muted-foreground font-medium">Key Value / Secret</span>
                        <SecretValueDisplay secret={auth?.value} label="API Key Value" />
                      </div>
                    </div>
                  )}

                  {/* OAUTH2 */}
                  {authType === "OAUTH2" && (
                    <div className="rounded-lg border border-ink/10 divide-y divide-ink/5">
                      <div className="flex items-center justify-between p-3">
                        <span className="text-muted-foreground font-medium">Grant Type</span>
                        <Badge variant="outline" className="border-primary/40 text-primary font-mono text-xs">
                          {auth?.grantType || "CLIENT_CREDENTIALS"}
                        </Badge>
                      </div>
                      <div className="flex items-center justify-between p-3">
                        <span className="text-muted-foreground font-medium">Token URL</span>
                        <div className="flex items-center gap-1 font-mono text-xs text-brand-sky">
                          <span>{auth?.tokenUrl || "—"}</span>
                          {auth?.tokenUrl && <CopyButton text={auth.tokenUrl} />}
                        </div>
                      </div>
                      <div className="flex items-center justify-between p-3">
                        <span className="text-muted-foreground font-medium">Client ID</span>
                        <span className="font-mono text-foreground">{auth?.clientId || "—"}</span>
                      </div>
                      <div className="flex items-center justify-between p-3">
                        <span className="text-muted-foreground font-medium">Client Secret</span>
                        <SecretValueDisplay secret={auth?.clientSecret} label="Client Secret" />
                      </div>
                      {Array.isArray(auth?.scopes) && auth.scopes.length > 0 && (
                        <div className="flex items-center justify-between p-3">
                          <span className="text-muted-foreground font-medium">Requested Scopes</span>
                          <div className="flex flex-wrap gap-1">
                            {auth.scopes.map((s: string) => (
                              <Badge key={s} variant="outline" className="text-xs font-mono">
                                {s}
                              </Badge>
                            ))}
                          </div>
                        </div>
                      )}
                      {auth?.onTokenExpire && (
                        <div className="flex items-center justify-between p-3">
                          <span className="text-muted-foreground font-medium">On Token Expire</span>
                          <Badge variant="outline" className="text-xs">
                            {auth.onTokenExpire}
                          </Badge>
                        </div>
                      )}
                    </div>
                  )}
                </CardContent>
              </Card>
            </div>
          )}

          {/* TAB 3: 1:1 ARCHITECTURE & TECHNICAL SPECS */}
          {activeTab === "metadata" && (
            <div className="space-y-6">
              {/* Architecture Explanation Banner */}
              <div className="p-5 rounded-xl border border-ink/10 bg-background-800/40 space-y-4">
                <div className="flex items-center gap-2">
                  <Workflow className="h-5 w-5 text-brand-sky" />
                  <Heading level="h4" className="!mb-0 text-foreground">
                    1:1 Association & Archetype Blueprint Architecture
                  </Heading>
                </div>
                <p className="text-xs text-muted-foreground leading-relaxed">
                  In the Eunomia Dataspaces technical model, a <strong className="text-purple-700 dark:text-purple-300 font-medium">Connector Template</strong> is a reusable archetype declaring parameter schemas and protocol skeletons. In contrast, a <strong className="text-brand-sky font-medium">Connector Instance</strong> is a concrete runtime entity bound <strong className="text-foreground font-semibold">1:1</strong> to a specific <strong className="text-emerald-700 dark:text-emerald-300 font-medium">DCAT Distribution</strong> with fully resolved authentication and endpoint parameters.
                </p>

                {/* Visual Architecture Flow */}
                <div className="grid grid-cols-1 md:grid-cols-3 gap-3 pt-2 font-mono text-xs">
                  <div className="p-3 rounded-lg bg-purple-500/10 border border-purple-500/20 space-y-1 text-center">
                    <Badge variant="outline" className="border-purple-500/40 text-purple-700 dark:text-purple-300 text-xs">
                      1. REUSABLE ARCHETYPE
                    </Badge>
                    <p className="font-semibold text-foreground text-xs mt-1">Connector Template</p>
                    <p className="text-xs text-muted-foreground">Parameter schemas & placeholders</p>
                  </div>

                  <div className="p-3 rounded-lg bg-sky-500/10 border border-sky-500/20 space-y-1 text-center">
                    <Badge variant="outline" className="border-sky-500/40 text-sky-700 dark:text-sky-300 text-xs">
                      2. CONCRETE RUNTIME
                    </Badge>
                    <p className="font-semibold text-brand-sky text-xs mt-1">Connector Instance</p>
                    <p className="text-xs text-muted-foreground">Resolved endpoints & credentials</p>
                  </div>

                  <div className="p-3 rounded-lg bg-emerald-500/10 border border-emerald-500/20 space-y-1 text-center">
                    <Badge variant="outline" className="border-emerald-500/40 text-emerald-700 dark:text-emerald-300 text-xs">
                      3. 1:1 BOUND RESOURCE
                    </Badge>
                    <p className="font-semibold text-emerald-700 dark:text-emerald-400 text-xs mt-1">DCAT Distribution</p>
                    <p className="text-xs text-muted-foreground">Dataset file / API resource</p>
                  </div>
                </div>
              </div>

              {/* Side-by-Side Comparison */}
              <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
                {/* Instance Column (1:1 Concrete Entity) */}
                <Card className="border-brand-sky/20 bg-background-800/40">
                  <CardHeader className="pb-3 border-b border-ink/5">
                    <div className="flex items-center justify-between">
                      <CardTitle className="text-sm font-semibold flex items-center gap-2 text-brand-sky">
                        <Server className="h-4 w-4" />
                        Connector Instance (1:1 Entity)
                      </CardTitle>
                      <Badge variant="outline" className="border-brand-sky/30 text-brand-sky text-xs">
                        Concrete
                      </Badge>
                    </div>
                    <CardDescription className="text-xs">
                      Resolved runtime instance querying dataplane transfers.
                    </CardDescription>
                  </CardHeader>
                  <CardContent className="pt-0">
                    <div className="divide-y divide-ink/5 text-xs">
                      <div className="flex flex-col sm:flex-row sm:items-center justify-between p-3 gap-1">
                        <span className="text-muted-foreground">Instance ID</span>
                        <span className="font-mono text-brand-sky text-xs break-all">{connector?.id || "—"}</span>
                      </div>
                      <div className="flex flex-col sm:flex-row sm:items-center justify-between p-3 gap-1">
                        <span className="text-muted-foreground">1:1 Distribution Ref</span>
                        <span className="font-mono text-xs text-foreground break-all">{distributionId}</span>
                      </div>
                      <div className="flex flex-col sm:flex-row sm:items-center justify-between p-3 gap-1">
                        <span className="text-muted-foreground">Distribution Title</span>
                        <span className="font-medium text-foreground">{distribution?.dctTitle || "—"}</span>
                      </div>
                      {distribution?.datasetId && (
                        <div className="flex flex-col sm:flex-row sm:items-center justify-between p-3 gap-1">
                          <span className="text-muted-foreground">Parent Dataset Ref</span>
                          <span className="font-mono text-xs text-foreground/80 break-all">{distribution.datasetId}</span>
                        </div>
                      )}
                      <div className="flex flex-col sm:flex-row sm:items-center justify-between p-3 gap-1">
                        <span className="text-muted-foreground">Lifecycle Mode</span>
                        <span className="font-mono text-foreground font-semibold">{mode}</span>
                      </div>
                      <div className="flex flex-col sm:flex-row sm:items-center justify-between p-3 gap-1">
                        <span className="text-muted-foreground">Resolved Auth</span>
                        <span className="font-mono text-foreground">{authType}</span>
                      </div>
                      <div className="flex flex-col sm:flex-row sm:items-center justify-between p-3 gap-1">
                        <span className="text-muted-foreground">Created At</span>
                        <FormatDate date={connector?.createdAt} />
                      </div>
                    </div>
                  </CardContent>
                </Card>

                {/* Template Column (Reusable Blueprint) */}
                <Card className="border-purple-500/20 bg-background-800/40">
                  <CardHeader className="pb-3 border-b border-ink/5">
                    <div className="flex items-center justify-between">
                      <CardTitle className="text-sm font-semibold flex items-center gap-2 text-purple-700 dark:text-purple-300">
                        <Cpu className="h-4 w-4" />
                        Origin Template Blueprint
                      </CardTitle>
                      <Badge variant="outline" className="border-purple-500/30 text-purple-700 dark:text-purple-300 text-xs">
                        Archetype
                      </Badge>
                    </div>
                    <CardDescription className="text-xs">
                      The archetype template defining the parameter schema and protocol pattern.
                    </CardDescription>
                  </CardHeader>
                  <CardContent className="pt-0">
                    <div className="divide-y divide-ink/5 text-xs">
                      <div className="flex flex-col sm:flex-row sm:items-center justify-between p-3 gap-1">
                        <span className="text-muted-foreground">Template Name</span>
                        <span className="font-medium text-foreground">{connector?.name || "—"}</span>
                      </div>
                      <div className="flex flex-col sm:flex-row sm:items-center justify-between p-3 gap-1">
                        <span className="text-muted-foreground">Template Version</span>
                        <span className="font-mono text-foreground/80">{connector?.version || "1.0.0"}</span>
                      </div>
                      <div className="flex flex-col sm:flex-row sm:items-center justify-between p-3 gap-1">
                        <span className="text-muted-foreground">Author / Maintainer</span>
                        <span className="font-medium text-foreground">{connector?.author || "Eunomia System"}</span>
                      </div>
                      <div className="flex flex-col sm:flex-row sm:items-center justify-between p-3 gap-1">
                        <span className="text-muted-foreground">Description</span>
                        <span className="text-foreground/80 max-w-xs text-right line-clamp-2">
                          {connector?.description || "Reusable blueprint defining parameter schemas and transport specs."}
                        </span>
                      </div>
                      <div className="flex flex-col sm:flex-row sm:items-center justify-between p-3 gap-1">
                        <span className="text-muted-foreground">Scope / Reusability</span>
                        <Badge variant="outline" className="border-ink/10 text-xs">
                          Many Instances per Template
                        </Badge>
                      </div>
                    </div>
                  </CardContent>
                </Card>
              </div>

              {/* Raw JSON Payload */}
              <details className="rounded-xl border border-ink/10 bg-background-800/40 p-4 text-xs">
                <summary className="cursor-pointer font-medium text-muted-foreground hover:text-foreground flex items-center gap-2">
                  <Code2 className="h-4 w-4" />
                  Inspect Raw ConnectorInstanceDto Specification (JSON)
                </summary>
                <pre className="mt-3 p-4 rounded-lg bg-sunken/40 font-mono text-xs text-brand-sky overflow-x-auto border border-ink/5">
                  {JSON.stringify(connector, null, 2)}
                </pre>
              </details>
            </div>
          )}
        </InnerBody>
      </InnerSidebarLayout>
    </PageLayout>
  );
}

/**
 * Route for displaying distribution connector details.
 */
export const Route = createFileRoute("/catalog/$catalogId/distribution-connector/$distributionId")({
  component: RouteComponent,
  pendingComponent: () => <div>Loading...</div>,
  loader: async ({ context: { queryClient }, params: { distributionId } }) => {
    await queryClient.ensureQueryData(getGetDistributionByIdQueryOptions(distributionId));
    return queryClient.ensureQueryData(
      getGetConnectorInstanceByDistributionQueryOptions(distributionId),
    );
  },
});
