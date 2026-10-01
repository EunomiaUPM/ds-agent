import { createFileRoute } from "@tanstack/react-router";
import {
  useGetApplicationConfig,
  getGetApplicationConfigQueryKey,
} from "shared/src/data/orval/keystore-config/keystore-config";
import { useQueryClient } from "@tanstack/react-query";
import { Skeleton } from "shared/src/components/ui/skeleton";
import { Button } from "shared/src/components/ui/button";
import { Badge } from "shared/src/components/ui/badge";
import { Input } from "shared/src/components/ui/input";
import { cn } from "shared/src/lib/utils";
import { toast } from "sonner";
import { useState, useMemo } from "react";
import {
  Server,
  Layers,
  ShieldCheck,
  FileText,
  ArrowLeftRight,
  Globe,
  Database,
  Settings2,
  Search,
  X,
  Copy,
  Check,
  Download,
  RefreshCw,
  Eye,
  EyeOff,
  Code2,
  SlidersHorizontal,
  ChevronDown,
  ChevronUp,
  ExternalLink,
  Boxes,
  Lock,
  ListTree,
  AlertCircle,
} from "lucide-react";

// =============================================================================
// SERVICE METADATA & HELPER FUNCTIONS
// =============================================================================

interface ServiceMeta {
  title: string;
  description: string;
  icon: React.ComponentType<{ className?: string }>;
  accentColor: string;
  tag: string;
}

const SERVICE_REGISTRY: Record<string, ServiceMeta> = {
  monolith: {
    title: "Monolith Core",
    description: "Core node runtime: HTTP routing, SQL persistence, credentials, and network topology",
    icon: Server,
    accentColor: "text-blue-500 bg-blue-500/10 border-blue-500/20",
    tag: "Core",
  },
  catalog: {
    title: "Catalog Agent",
    description: "Federated data catalog, datasets, data services, and ODRL policy templates",
    icon: Layers,
    accentColor: "text-purple-500 bg-purple-500/10 border-purple-500/20",
    tag: "Catalog",
  },
  ssi_auth: {
    title: "SSI & Identity",
    description: "Self-sovereign identity management, DIDs (Jwk/Web), Verifiable Credentials, and Gaia-X",
    icon: ShieldCheck,
    accentColor: "text-emerald-500 bg-emerald-500/10 border-emerald-500/20",
    tag: "Security",
  },
  contracts: {
    title: "Contract Negotiation",
    description: "DSP data contract negotiation protocol, contract agreements, and policy evaluation",
    icon: FileText,
    accentColor: "text-amber-500 bg-amber-500/10 border-amber-500/20",
    tag: "DSP",
  },
  transfer: {
    title: "Data Transfer Plane",
    description: "Data planes, streaming pipelines, and transfer process lifecycle management",
    icon: ArrowLeftRight,
    accentColor: "text-cyan-500 bg-cyan-500/10 border-cyan-500/20",
    tag: "Transfer",
  },
  gateway: {
    title: "API Gateway Proxy",
    description: "Perimeter ingress gateway and reverse proxy router for GUI client requests",
    icon: Globe,
    accentColor: "text-rose-500 bg-rose-500/10 border-rose-500/20",
    tag: "Gateway",
  },
  cache: {
    title: "Cache & State",
    description: "In-memory Redis session storage, state caching, and message queues",
    icon: Database,
    accentColor: "text-indigo-500 bg-indigo-500/10 border-indigo-500/20",
    tag: "Cache",
  },
};

function getServiceMeta(key: string): ServiceMeta {
  if (SERVICE_REGISTRY[key]) {
    return SERVICE_REGISTRY[key];
  }
  return {
    title: formatKeyLabel(key),
    description: `Runtime configuration parameters for ${key} module`,
    icon: Settings2,
    accentColor: "text-primary bg-primary/10 border-primary/20",
    tag: "Config",
  };
}

function formatKeyLabel(key: string): string {
  return key
    .replace(/_/g, " ")
    .replace(/([a-z])([A-Z])/g, "$1 $2")
    .replace(/\b\w/g, (c) => c.toUpperCase());
}

function isSensitiveKey(key: string): boolean {
  return /password|secret|token|seed|api_key/i.test(key);
}

function isUrl(str: string): boolean {
  return typeof str === "string" && /^https?:\/\//i.test(str);
}

function isFilePath(str: string): boolean {
  return typeof str === "string" && (str.startsWith("./") || str.startsWith("../") || str.startsWith("/"));
}

function copyToClipboard(text: string, label: string = "Copied to clipboard") {
  navigator.clipboard.writeText(text);
  toast.success(label);
}

function downloadConfigJson(data: unknown) {
  try {
    const jsonStr = JSON.stringify(data, null, 2);
    const blob = new Blob([jsonStr], { type: "application/json" });
    const url = URL.createObjectURL(blob);
    const link = document.createElement("a");
    link.href = url;
    link.download = `eunomia-node-config-${new Date().toISOString().slice(0, 10)}.json`;
    link.click();
    URL.revokeObjectURL(url);
    toast.success("Configuration file downloaded");
  } catch {
    toast.error("Failed to download configuration file");
  }
}

// =============================================================================
// SEARCH HIGHLIGHTER
// =============================================================================

function HighlightText({ text, query }: { text: string; query: string }) {
  if (!query || !text) return <>{text}</>;
  const lowerText = text.toLowerCase();
  const lowerQuery = query.toLowerCase();
  const index = lowerText.indexOf(lowerQuery);
  if (index === -1) return <>{text}</>;

  const before = text.substring(0, index);
  const match = text.substring(index, index + query.length);
  const after = text.substring(index + query.length);

  return (
    <>
      {before}
      <mark className="bg-warn-500/30 text-warn-900 dark:text-warn-100 rounded px-0.5 font-semibold">
        {match}
      </mark>
      <HighlightText text={after} query={query} />
    </>
  );
}

// =============================================================================
// SYNTAX HIGHLIGHTED CODE BLOCK
// =============================================================================

function CodeBlock({
  data,
  title,
  searchQuery,
}: {
  data: unknown;
  title?: string;
  searchQuery?: string;
}) {
  const [copied, setCopied] = useState(false);
  const jsonString = useMemo(() => JSON.stringify(data, null, 2), [data]);
  const lines = useMemo(() => jsonString.split("\n"), [jsonString]);

  const handleCopy = () => {
    navigator.clipboard.writeText(jsonString);
    setCopied(true);
    toast.success("JSON copied to clipboard");
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className="rounded-xl border border-ink/10 bg-sunken/40 overflow-hidden text-xs font-mono shadow-sm">
      {/* Code Header Bar */}
      <div className="flex items-center justify-between px-4 py-2.5 bg-ink/5 border-b border-ink/10">
        <div className="flex items-center gap-2">
          <div className="flex gap-1.5">
            <div className="w-2.5 h-2.5 rounded-full bg-danger-500/60" />
            <div className="w-2.5 h-2.5 rounded-full bg-warn-500/60" />
            <div className="w-2.5 h-2.5 rounded-full bg-success-500/60" />
          </div>
          {title && (
            <span className="text-muted-foreground/80 font-medium pl-2 text-[11px]">
              {title}.json
            </span>
          )}
          <span className="text-muted-foreground/50 text-[10px]">
            ({lines.length} lines • {(new Blob([jsonString]).size / 1024).toFixed(1)} KB)
          </span>
        </div>
        <Button
          variant="ghost"
          size="sm"
          onClick={handleCopy}
          className="h-6 px-2 text-[11px] font-sans gap-1 text-muted-foreground hover:text-foreground"
        >
          {copied ? <Check className="h-3 w-3 text-success-500" /> : <Copy className="h-3 w-3" />}
          {copied ? "Copied" : "Copy"}
        </Button>
      </div>

      {/* Code Body with Line Numbers */}
      <div className="p-3 overflow-x-auto max-h-[550px] overflow-y-auto leading-relaxed">
        <table className="w-full border-collapse">
          <tbody>
            {lines.map((line, idx) => {
              const lineNum = idx + 1;
              const isMatch =
                searchQuery && searchQuery.trim() !== ""
                  ? line.toLowerCase().includes(searchQuery.toLowerCase())
                  : false;

              const keyValMatch = line.match(/^(\s*)(".*?")(\s*:\s*)(.*)$/);

              return (
                <tr
                  key={idx}
                  className={cn(
                    "hover:bg-ink/[0.04] transition-colors",
                    isMatch && "bg-warn-500/10",
                  )}
                >
                  <td className="select-none text-right pr-4 text-muted-foreground/40 w-10 text-[11px] align-top">
                    {lineNum}
                  </td>
                  <td className="whitespace-pre break-all pl-2 font-mono">
                    {keyValMatch ? (
                      <>
                        <span>{keyValMatch[1]}</span>
                        <span className="text-sky-600 dark:text-sky-300 font-semibold">
                          <HighlightText text={keyValMatch[2]} query={searchQuery || ""} />
                        </span>
                        <span className="text-muted-foreground/60">{keyValMatch[3]}</span>
                        <span className="text-foreground/90">
                          <HighlightText text={keyValMatch[4]} query={searchQuery || ""} />
                        </span>
                      </>
                    ) : (
                      <span className="text-foreground/80">
                        <HighlightText text={line} query={searchQuery || ""} />
                      </span>
                    )}
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>
    </div>
  );
}

// =============================================================================
// PRIMITIVE PROPERTY VALUE COMPONENT
// =============================================================================

function FormattedValue({
  value,
  propKey,
  searchQuery,
}: {
  value: unknown;
  propKey: string;
  searchQuery?: string;
}) {
  const [showSecret, setShowSecret] = useState(false);
  const sensitive = isSensitiveKey(propKey);

  if (value === null || value === undefined) {
    return (
      <span className="font-mono text-xs italic text-muted-foreground/40 select-none">null</span>
    );
  }

  if (typeof value === "boolean") {
    return (
      <Badge
        variant={value ? "success" : "outline"}
        className={cn(
          "gap-1 font-mono text-[11px] font-semibold px-2 py-0.5",
          !value && "text-muted-foreground/70 border-ink/20",
        )}
      >
        {value ? <Check className="h-3 w-3" /> : <X className="h-3 w-3" />}
        {String(value)}
      </Badge>
    );
  }

  if (typeof value === "number") {
    return (
      <span className="inline-flex items-center px-2 py-0.5 rounded font-mono text-xs font-semibold bg-primary-50 dark:bg-primary-950/40 text-primary-800 dark:text-primary-300 border border-primary-500/20">
        <HighlightText text={String(value)} query={searchQuery || ""} />
      </span>
    );
  }

  if (typeof value === "string") {
    if (sensitive) {
      return (
        <div className="inline-flex items-center gap-2">
          <span className="font-mono text-xs text-muted-foreground bg-sunken/60 px-2 py-0.5 rounded border border-ink/10">
            {showSecret ? value : "••••••••••••"}
          </span>
          <button
            type="button"
            onClick={() => setShowSecret((v) => !v)}
            className="text-muted-foreground hover:text-foreground transition-colors p-1"
            title={showSecret ? "Hide value" : "Show value"}
          >
            {showSecret ? <EyeOff className="h-3.5 w-3.5" /> : <Eye className="h-3.5 w-3.5" />}
          </button>
          <button
            type="button"
            onClick={() => copyToClipboard(value, "Credential copied")}
            className="text-muted-foreground hover:text-foreground transition-colors p-1"
            title="Copy credential"
          >
            <Copy className="h-3.5 w-3.5" />
          </button>
        </div>
      );
    }

    if (isUrl(value)) {
      return (
        <div className="inline-flex items-center gap-1.5 max-w-full">
          <a
            href={value}
            target="_blank"
            rel="noopener noreferrer"
            className="font-mono text-xs text-brand-sky hover:underline inline-flex items-center gap-1 truncate"
          >
            <HighlightText text={value} query={searchQuery || ""} />
            <ExternalLink className="h-3 w-3 shrink-0" />
          </a>
          <button
            type="button"
            onClick={() => copyToClipboard(value, "URL copied")}
            className="text-muted-foreground/60 hover:text-foreground transition-colors p-0.5"
            title="Copy URL"
          >
            <Copy className="h-3 w-3" />
          </button>
        </div>
      );
    }

    if (isFilePath(value)) {
      return (
        <span className="font-mono text-xs px-2 py-0.5 rounded bg-sunken/80 text-foreground/80 border border-ink/10 break-all">
          <HighlightText text={value} query={searchQuery || ""} />
        </span>
      );
    }

    return (
      <span className="font-mono text-xs text-foreground/90 break-all">
        <HighlightText text={value} query={searchQuery || ""} />
      </span>
    );
  }

  if (Array.isArray(value)) {
    if (value.length === 0) {
      return <span className="font-mono text-xs text-muted-foreground/40">[empty]</span>;
    }

    const allPrimitives = value.every((item) => typeof item !== "object" || item === null);
    if (allPrimitives) {
      return (
        <div className="flex flex-wrap gap-1.5 items-center">
          {value.map((item, i) => (
            <Badge key={i} variant="secondary" className="font-mono text-[11px]">
              <HighlightText text={String(item)} query={searchQuery || ""} />
            </Badge>
          ))}
        </div>
      );
    }

    return (
      <span className="font-mono text-xs text-muted-foreground/70">
        Array ({value.length} items)
      </span>
    );
  }

  return (
    <span className="font-mono text-xs text-muted-foreground">
      {JSON.stringify(value)}
    </span>
  );
}

// =============================================================================
// SUB-PANEL COMPONENT (FOR NESTED OBJECTS)
// =============================================================================

function SubObjectPanel({
  subKey,
  data,
  searchQuery,
  depth = 1,
}: {
  subKey: string;
  data: Record<string, unknown>;
  searchQuery?: string;
  depth?: number;
}) {
  const [open, setOpen] = useState(true);
  const entries = Object.entries(data);

  return (
    <div
      className={cn(
        "rounded-xl border transition-all overflow-hidden",
        depth === 1
          ? "border-ink/10 bg-card/60 backdrop-blur-sm shadow-xs"
          : "border-ink/5 bg-ink/[0.02]",
      )}
    >
      {/* Sub-panel header */}
      <button
        type="button"
        onClick={() => setOpen((v) => !v)}
        className="w-full flex items-center justify-between px-3.5 py-2.5 hover:bg-ink/[0.04] transition-colors border-b border-ink/5"
      >
        <div className="flex items-center gap-2">
          <span className="text-[11px] font-bold uppercase tracking-wider text-foreground/80 font-mono flex items-center gap-1.5">
            <SlidersHorizontal className="h-3 w-3 text-muted-foreground/70" />
            <HighlightText text={subKey} query={searchQuery || ""} />
          </span>
          <Badge variant="outline" className="text-[10px] px-1.5 py-0 h-4 border-ink/15 text-muted-foreground">
            {entries.length} {entries.length === 1 ? "prop" : "props"}
          </Badge>
        </div>
        <div className="flex items-center gap-2">
          <span className="text-muted-foreground/40">
            {open ? <ChevronUp className="h-3.5 w-3.5" /> : <ChevronDown className="h-3.5 w-3.5" />}
          </span>
        </div>
      </button>

      {/* Sub-panel content */}
      {open && (
        <div className="p-3.5 space-y-3">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-2.5">
            {entries.map(([k, v]) => {
              if (v !== null && typeof v === "object" && !Array.isArray(v)) {
                return (
                  <div key={k} className="col-span-full mt-1">
                    <SubObjectPanel
                      subKey={k}
                      data={v as Record<string, unknown>}
                      searchQuery={searchQuery}
                      depth={depth + 1}
                    />
                  </div>
                );
              }

              return (
                <div
                  key={k}
                  className="flex flex-col sm:flex-row sm:items-center justify-between gap-1 p-2 rounded-lg bg-ink/[0.02] border border-ink/5 hover:border-ink/15 transition-all"
                >
                  <span className="font-mono text-xs text-muted-foreground font-medium flex items-center gap-1.5">
                    {isSensitiveKey(k) && <Lock className="h-3 w-3 text-amber-500/80" />}
                    <HighlightText text={k} query={searchQuery || ""} />
                  </span>
                  <div className="sm:text-right">
                    <FormattedValue value={v} propKey={k} searchQuery={searchQuery} />
                  </div>
                </div>
              );
            })}
          </div>
        </div>
      )}
    </div>
  );
}

// =============================================================================
// SERVICE CARD COMPONENT
// =============================================================================

function ServiceConfigCard({
  serviceKey,
  data,
  searchQuery,
  defaultExpanded = true,
}: {
  serviceKey: string;
  data: unknown;
  searchQuery?: string;
  defaultExpanded?: boolean;
}) {
  const [expanded, setExpanded] = useState(defaultExpanded);
  const [viewMode, setViewMode] = useState<"structured" | "json">("structured");
  const [copied, setCopied] = useState(false);

  // Update expanded state if defaultExpanded prop changes
  useMemo(() => {
    setExpanded(defaultExpanded);
  }, [defaultExpanded]);

  const meta = getServiceMeta(serviceKey);
  const IconComponent = meta.icon;

  const isObject = data !== null && typeof data === "object" && !Array.isArray(data);
  const entries = isObject ? Object.entries(data as Record<string, unknown>) : [];

  const filteredEntries = useMemo(() => {
    if (!searchQuery || !searchQuery.trim() || !isObject) return entries;
    const q = searchQuery.toLowerCase();
    return entries.filter(([k, v]) => {
      const matchKey = k.toLowerCase().includes(q);
      const matchVal = JSON.stringify(v).toLowerCase().includes(q);
      return matchKey || matchVal;
    });
  }, [entries, isObject, searchQuery]);

  const { primitiveProps, objectProps } = useMemo(() => {
    const prim: [string, unknown][] = [];
    const obj: [string, unknown][] = [];
    filteredEntries.forEach(([k, v]) => {
      if (v !== null && typeof v === "object" && !Array.isArray(v)) {
        obj.push([k, v]);
      } else {
        prim.push([k, v]);
      }
    });
    return { primitiveProps: prim, objectProps: obj };
  }, [filteredEntries]);

  const handleCopySection = () => {
    copyToClipboard(JSON.stringify(data, null, 2), `${meta.title} JSON copied`);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className="rounded-2xl border border-ink/10 bg-card/60 backdrop-blur-sm shadow-sm overflow-hidden transition-all duration-200 hover:border-ink/20">
      {/* Card Header */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3 p-4 md:px-6 md:py-4 bg-ink/[0.02] border-b border-ink/10">
        <div className="flex items-start sm:items-center gap-3.5">
          <div
            className={cn(
              "p-2.5 rounded-xl border flex items-center justify-center shrink-0 shadow-xs",
              meta.accentColor,
            )}
          >
            <IconComponent className="h-5 w-5" />
          </div>
          <div>
            <div className="flex items-center gap-2 flex-wrap">
              <h3 className="font-semibold text-foreground text-base tracking-tight leading-none">
                <HighlightText text={meta.title} query={searchQuery || ""} />
              </h3>
              <Badge variant="code" className="text-[10px] py-0 px-1.5 border-ink/10">
                <HighlightText text={serviceKey} query={searchQuery || ""} />
              </Badge>
              <Badge variant="outline" className="text-[10px] px-1.5 py-0 border-ink/15 text-muted-foreground">
                {entries.length} {entries.length === 1 ? "property" : "properties"}
              </Badge>
            </div>
            <p className="text-xs text-muted-foreground mt-1 line-clamp-1 max-w-xl">
              {meta.description}
            </p>
          </div>
        </div>

        {/* Card Header Actions */}
        <div className="flex items-center gap-1.5 self-end sm:self-center shrink-0">
          <div className="inline-flex rounded-lg border border-ink/10 p-0.5 bg-sunken/40">
            <button
              type="button"
              onClick={() => {
                setViewMode("structured");
                setExpanded(true);
              }}
              className={cn(
                "p-1 rounded-md text-xs transition-all",
                viewMode === "structured"
                  ? "bg-card text-foreground shadow-xs"
                  : "text-muted-foreground hover:text-foreground",
              )}
              title="Structured view"
            >
              <ListTree className="h-3.5 w-3.5" />
            </button>
            <button
              type="button"
              onClick={() => {
                setViewMode("json");
                setExpanded(true);
              }}
              className={cn(
                "p-1 rounded-md text-xs transition-all",
                viewMode === "json"
                  ? "bg-card text-foreground shadow-xs"
                  : "text-muted-foreground hover:text-foreground",
              )}
              title="Raw JSON view"
            >
              <Code2 className="h-3.5 w-3.5" />
            </button>
          </div>

          <Button
            variant="ghost"
            size="icon_sm"
            onClick={handleCopySection}
            className="h-7 w-7 text-muted-foreground hover:text-foreground"
            title="Copy section JSON"
          >
            {copied ? <Check className="h-3.5 w-3.5 text-success-500" /> : <Copy className="h-3.5 w-3.5" />}
          </Button>

          <Button
            variant="ghost"
            size="icon_sm"
            onClick={() => setExpanded((v) => !v)}
            className="h-7 w-7 text-muted-foreground hover:text-foreground"
            title={expanded ? "Collapse section" : "Expand section"}
          >
            {expanded ? <ChevronUp className="h-4 w-4" /> : <ChevronDown className="h-4 w-4" />}
          </Button>
        </div>
      </div>

      {/* Card Content */}
      {expanded && (
        <div className="p-4 md:p-6 space-y-4">
          {viewMode === "json" ? (
            <CodeBlock data={data} title={serviceKey} searchQuery={searchQuery} />
          ) : !isObject ? (
            <div className="p-4 rounded-xl bg-ink/[0.03] border border-ink/10 font-mono text-xs">
              <FormattedValue value={data} propKey={serviceKey} searchQuery={searchQuery} />
            </div>
          ) : filteredEntries.length === 0 ? (
            <div className="p-6 text-center rounded-xl border border-dashed border-ink/15 text-muted-foreground text-xs">
              No properties matching search query in this section.
            </div>
          ) : (
            <div className="space-y-4">
              {primitiveProps.length > 0 && (
                <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-2.5">
                  {primitiveProps.map(([k, v]) => (
                    <div
                      key={k}
                      className="flex flex-col sm:flex-row sm:items-center justify-between gap-1 p-2.5 rounded-xl bg-ink/[0.02] border border-ink/10 hover:border-ink/20 transition-all shadow-2xs"
                    >
                      <span className="font-mono text-xs text-muted-foreground font-medium flex items-center gap-1.5">
                        {isSensitiveKey(k) && <Lock className="h-3 w-3 text-amber-500/80" />}
                        <HighlightText text={k} query={searchQuery || ""} />
                      </span>
                      <div className="sm:text-right">
                        <FormattedValue value={v} propKey={k} searchQuery={searchQuery} />
                      </div>
                    </div>
                  ))}
                </div>
              )}

              {objectProps.length > 0 && (
                <div className="space-y-3 pt-1">
                  {objectProps.map(([k, v]) => (
                    <SubObjectPanel
                      key={k}
                      subKey={k}
                      data={v as Record<string, unknown>}
                      searchQuery={searchQuery}
                    />
                  ))}
                </div>
              )}
            </div>
          )}
        </div>
      )}
    </div>
  );
}

// =============================================================================
// METRIC STATS HEADER CARDS
// =============================================================================

function MetricsOverview({ cfg }: { cfg: Record<string, any> }) {
  const servicesCount = Object.keys(cfg).length;

  const monolithCommon = cfg?.monolith?.common || {};
  const httpHost = monolithCommon?.hosts?.http;
  const dbInfo = monolithCommon?.db;
  const connection = monolithCommon?.connection;
  const cacheInfo = cfg?.monolith?.cache || cfg?.catalog?.cache;
  const ssiAuth = cfg?.ssi_auth;

  const isLocal = connection?.is_local ?? true;
  const isProd = connection?.is_prod ?? false;
  const hostUrl = httpHost?.url ? `${httpHost.protocol || "http"}://${httpHost.url}:${httpHost.port || "1100"}` : "127.0.0.1:1100";
  const dbLabel = dbInfo?.db_type ? `${dbInfo.db_type} (${dbInfo.port || "1300"})` : "Postgres";
  const cacheLabel = cacheInfo?.cache_type ? `${cacheInfo.cache_type} (${cacheInfo.port || "6380"})` : "Redis";
  const walletType = ssiAuth?.wallet_config?.wallet || "Fafnir";
  const didType = ssiAuth?.did_config?.type || "Jwk";

  return (
    <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-3.5 mb-6 w-full">
      {/* Card 1: Services */}
      <div className="p-4 rounded-2xl border border-ink/10 bg-card/60 backdrop-blur-sm shadow-xs flex flex-col justify-between">
        <div className="flex items-center justify-between">
          <span className="text-[11px] font-bold uppercase tracking-wider text-muted-foreground">
            Active Modules
          </span>
          <div className="p-2 rounded-lg bg-blue-500/10 text-blue-500 border border-blue-500/20">
            <Boxes className="h-4 w-4" />
          </div>
        </div>
        <div className="mt-3">
          <div className="text-2xl font-bold text-foreground tracking-tight">
            {servicesCount}{" "}
            <span className="text-sm font-normal text-muted-foreground">Services</span>
          </div>
          <p className="text-xs text-muted-foreground/80 mt-1 line-clamp-1">
            Monolith, Catalog, SSI, Contracts, Transfer, Gateway
          </p>
        </div>
      </div>

      {/* Card 2: Core Host */}
      <div className="p-4 rounded-2xl border border-ink/10 bg-card/60 backdrop-blur-sm shadow-xs flex flex-col justify-between">
        <div className="flex items-center justify-between">
          <span className="text-[11px] font-bold uppercase tracking-wider text-muted-foreground">
            Primary Host
          </span>
          <div className="p-2 rounded-lg bg-purple-500/10 text-purple-500 border border-purple-500/20">
            <Server className="h-4 w-4" />
          </div>
        </div>
        <div className="mt-3">
          <div className="text-lg font-bold font-mono text-brand-sky truncate">
            {hostUrl}
          </div>
          <p className="text-xs text-muted-foreground/80 mt-1 flex items-center gap-1.5">
            <Badge variant="outline" className="text-[10px] px-1 py-0 h-4 border-ink/15">
              API {monolithCommon?.api?.version || "v1"}
            </Badge>
            <span>Active HTTP port</span>
          </p>
        </div>
      </div>

      {/* Card 3: Storage & Cache */}
      <div className="p-4 rounded-2xl border border-ink/10 bg-card/60 backdrop-blur-sm shadow-xs flex flex-col justify-between">
        <div className="flex items-center justify-between">
          <span className="text-[11px] font-bold uppercase tracking-wider text-muted-foreground">
            Datastore & Cache
          </span>
          <div className="p-2 rounded-lg bg-amber-500/10 text-amber-500 border border-amber-500/20">
            <Database className="h-4 w-4" />
          </div>
        </div>
        <div className="mt-3">
          <div className="text-base font-bold text-foreground truncate">
            {dbLabel}
          </div>
          <p className="text-xs text-muted-foreground/80 mt-1">
            Cache: <span className="font-medium text-foreground">{cacheLabel}</span>
          </p>
        </div>
      </div>

      {/* Card 4: Security & Mode */}
      <div className="p-4 rounded-2xl border border-ink/10 bg-card/60 backdrop-blur-sm shadow-xs flex flex-col justify-between">
        <div className="flex items-center justify-between">
          <span className="text-[11px] font-bold uppercase tracking-wider text-muted-foreground">
            Identity & Environment
          </span>
          <div className="p-2 rounded-lg bg-emerald-500/10 text-emerald-500 border border-emerald-500/20">
            <ShieldCheck className="h-4 w-4" />
          </div>
        </div>
        <div className="mt-3">
          <div className="flex items-center gap-2">
            <Badge
              variant={isProd ? "success" : "secondary"}
              className="font-medium text-xs uppercase tracking-wide"
            >
              {isProd ? "Production" : isLocal ? "Local Dev" : "Staging"}
            </Badge>
            <span className="text-xs font-mono text-muted-foreground">{didType}</span>
          </div>
          <p className="text-xs text-muted-foreground/80 mt-1">
            Wallet: <span className="font-medium text-foreground">{walletType}</span>
          </p>
        </div>
      </div>
    </div>
  );
}

// =============================================================================
// MAIN COMPONENT
// =============================================================================

const KeystoreConfig = () => {
  const queryClient = useQueryClient();
  const { data: response, isLoading, error, refetch, isFetching } = useGetApplicationConfig();

  const [searchQuery, setSearchQuery] = useState("");
  const [selectedSection, setSelectedSection] = useState<string>("all");
  const [globalViewMode, setGlobalViewMode] = useState<"structured" | "json">("structured");
  const [copiedAll, setCopiedAll] = useState(false);
  const [allExpanded, setAllExpanded] = useState(true);

  const cfg = (response?.status === 200 ? response.data : {}) as Record<string, unknown>;
  const sectionKeys = Object.keys(cfg);

  const visibleSections = useMemo(() => {
    let keys = sectionKeys;
    if (selectedSection !== "all") {
      keys = keys.filter((k) => k === selectedSection);
    }
    if (searchQuery.trim()) {
      const q = searchQuery.toLowerCase();
      keys = keys.filter((k) => {
        const matchTitle = k.toLowerCase().includes(q);
        const matchMeta = SERVICE_REGISTRY[k]?.title.toLowerCase().includes(q);
        const matchData = JSON.stringify(cfg[k]).toLowerCase().includes(q);
        return matchTitle || matchMeta || matchData;
      });
    }
    return keys;
  }, [sectionKeys, selectedSection, searchQuery, cfg]);

  const handleCopyAll = () => {
    copyToClipboard(JSON.stringify(cfg, null, 2), "Complete configuration copied");
    setCopiedAll(true);
    setTimeout(() => setCopiedAll(false), 2000);
  };

  const handleRefresh = () => {
    queryClient.invalidateQueries({ queryKey: getGetApplicationConfigQueryKey() });
    refetch();
    toast.success("Refreshing configuration...");
  };

  // Loading state
  if (isLoading) {
    return (
      <div className="w-full space-y-6">
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-3.5">
          {[...Array(4)].map((_, i) => (
            <Skeleton key={i} className="h-28 w-full rounded-2xl" />
          ))}
        </div>
        <Skeleton className="h-12 w-full rounded-xl" />
        <div className="space-y-4">
          {[...Array(3)].map((_, i) => (
            <Skeleton key={i} className="h-44 w-full rounded-2xl" />
          ))}
        </div>
      </div>
    );
  }

  // Error state
  if (error || response?.status !== 200) {
    return (
      <div className="w-full rounded-2xl border border-danger-500/20 bg-danger-500/5 p-8 text-center space-y-4">
        <div className="mx-auto w-12 h-12 rounded-full bg-danger-500/10 flex items-center justify-center text-danger-500 border border-danger-500/20">
          <AlertCircle className="h-6 w-6" />
        </div>
        <div className="space-y-1">
          <h3 className="text-base font-semibold text-foreground">
            Failed to load application configuration
          </h3>
          <p className="text-xs text-muted-foreground max-w-md mx-auto">
            Could not retrieve configuration from <code className="font-mono text-danger-500">/v1/keystore/config</code>.
            Please verify that the backend keystore service is running.
          </p>
        </div>
        <Button variant="outline" size="sm" onClick={handleRefresh}>
          <RefreshCw className="h-3.5 w-3.5 mr-1.5" />
          Retry
        </Button>
      </div>
    );
  }

  return (
    <div className="w-full space-y-6">
      {/* 1. Metric Stat Cards */}
      <MetricsOverview cfg={cfg} />

      {/* 2. Controls & Search Toolbar */}
      <div className="p-4 rounded-2xl border border-ink/10 bg-card/60 backdrop-blur-sm shadow-xs space-y-4">
        <div className="flex flex-col md:flex-row items-stretch md:items-center justify-between gap-3">
          {/* Search Box */}
          <div className="relative flex-1">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground/60 pointer-events-none" />
            <Input
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              placeholder="Filter by parameter, port, host, db, redis, key..."
              className="pl-9 pr-8 text-xs font-mono h-9 bg-sunken/40 border-ink/10 rounded-xl"
            />
            {searchQuery && (
              <button
                type="button"
                onClick={() => setSearchQuery("")}
                className="absolute right-2.5 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground p-0.5"
                title="Clear search"
              >
                <X className="h-3.5 w-3.5" />
              </button>
            )}
          </div>

          {/* Action Buttons */}
          <div className="flex items-center gap-2 shrink-0 flex-wrap">
            {/* View Mode Toggle */}
            <div className="inline-flex rounded-xl border border-ink/10 p-0.5 bg-sunken/40">
              <button
                type="button"
                onClick={() => setGlobalViewMode("structured")}
                className={cn(
                  "px-3 py-1.5 rounded-lg text-xs font-medium inline-flex items-center gap-1.5 transition-all",
                  globalViewMode === "structured"
                    ? "bg-card text-foreground shadow-xs"
                    : "text-muted-foreground hover:text-foreground",
                )}
              >
                <ListTree className="h-3.5 w-3.5" />
                Visual
              </button>
              <button
                type="button"
                onClick={() => setGlobalViewMode("json")}
                className={cn(
                  "px-3 py-1.5 rounded-lg text-xs font-medium inline-flex items-center gap-1.5 transition-all",
                  globalViewMode === "json"
                    ? "bg-card text-foreground shadow-xs"
                    : "text-muted-foreground hover:text-foreground",
                )}
              >
                <Code2 className="h-3.5 w-3.5" />
                JSON
              </button>
            </div>

            {/* Expand / Collapse All Toggle (only in structured mode) */}
            {globalViewMode === "structured" && (
              <Button
                variant="outline"
                size="sm"
                onClick={() => setAllExpanded((v) => !v)}
                className="text-xs h-9 px-3 gap-1.5 font-medium rounded-xl border-ink/10"
                title={allExpanded ? "Collapse all sections" : "Expand all sections"}
              >
                {allExpanded ? (
                  <>
                    <ChevronUp className="h-3.5 w-3.5" />
                    Collapse all
                  </>
                ) : (
                  <>
                    <ChevronDown className="h-3.5 w-3.5" />
                    Expand all
                  </>
                )}
              </Button>
            )}

            {/* Copy All */}
            <Button
              variant="outline"
              size="sm"
              onClick={handleCopyAll}
              className="text-xs h-9 px-3 gap-1.5 font-medium rounded-xl border-ink/10"
              title="Copy complete JSON to clipboard"
            >
              {copiedAll ? (
                <Check className="h-3.5 w-3.5 text-success-500" />
              ) : (
                <Copy className="h-3.5 w-3.5" />
              )}
              {copiedAll ? "Copied" : "Copy all"}
            </Button>

            {/* Download JSON */}
            <Button
              variant="outline"
              size="sm"
              onClick={() => downloadConfigJson(cfg)}
              className="text-xs h-9 px-3 gap-1.5 font-medium rounded-xl border-ink/10"
              title="Download configuration as JSON"
            >
              <Download className="h-3.5 w-3.5" />
              Download
            </Button>

            {/* Refresh Button */}
            <Button
              variant="ghost"
              size="icon_sm"
              onClick={handleRefresh}
              className="h-9 w-9 rounded-xl text-muted-foreground hover:text-foreground"
              title="Refresh configuration"
            >
              <RefreshCw className={cn("h-4 w-4", isFetching && "animate-spin text-primary")} />
            </Button>
          </div>
        </div>

        {/* Category Filter Chips */}
        <div className="flex items-center gap-1.5 overflow-x-auto pt-1 pb-0.5 no-scrollbar">
          <button
            type="button"
            onClick={() => setSelectedSection("all")}
            className={cn(
              "px-3 py-1 rounded-lg text-xs font-medium shrink-0 transition-all border",
              selectedSection === "all"
                ? "bg-primary text-white border-primary shadow-xs"
                : "bg-ink/[0.03] text-muted-foreground border-ink/10 hover:border-ink/20 hover:text-foreground",
            )}
          >
            All ({sectionKeys.length})
          </button>
          {sectionKeys.map((key) => {
            const meta = getServiceMeta(key);
            const isSelected = selectedSection === key;
            return (
              <button
                key={key}
                type="button"
                onClick={() => setSelectedSection(key)}
                className={cn(
                  "px-3 py-1 rounded-lg text-xs font-medium shrink-0 transition-all border inline-flex items-center gap-1.5",
                  isSelected
                    ? "bg-primary text-white border-primary shadow-xs"
                    : "bg-ink/[0.03] text-muted-foreground border-ink/10 hover:border-ink/20 hover:text-foreground",
                )}
              >
                <span className="capitalize">{meta.title}</span>
              </button>
            );
          })}
        </div>
      </div>

      {/* 3. Content Display: Global JSON or Structured Cards */}
      {globalViewMode === "json" ? (
        <CodeBlock data={cfg} title="application-config" searchQuery={searchQuery} />
      ) : visibleSections.length === 0 ? (
        <div className="p-12 text-center rounded-2xl border border-dashed border-ink/15 bg-card/30 space-y-3">
          <div className="w-10 h-10 rounded-full bg-ink/5 flex items-center justify-center mx-auto text-muted-foreground">
            <Search className="h-5 w-5" />
          </div>
          <div className="space-y-1">
            <h4 className="text-sm font-semibold text-foreground">
              No configuration parameters found
            </h4>
            <p className="text-xs text-muted-foreground max-w-sm mx-auto">
              No matching settings found for &ldquo;{searchQuery}&rdquo;. Try another keyword or clear filters.
            </p>
          </div>
          <Button
            variant="outline"
            size="sm"
            onClick={() => {
              setSearchQuery("");
              setSelectedSection("all");
            }}
          >
            Reset filters
          </Button>
        </div>
      ) : (
        <div className="space-y-4">
          {visibleSections.map((key) => (
            <ServiceConfigCard
              key={key}
              serviceKey={key}
              data={cfg[key]}
              searchQuery={searchQuery}
              defaultExpanded={allExpanded}
            />
          ))}
        </div>
      )}
    </div>
  );
};

export const Route = createFileRoute("/keystore/config")({
  component: KeystoreConfig,
});


