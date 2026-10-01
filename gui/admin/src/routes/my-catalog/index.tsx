import { createFileRoute, Link } from "@tanstack/react-router";
import React, { useState, useMemo } from "react";
import Heading from "shared/src/components/ui/heading";
import DatasetItem from "shared/src/components/ui/dataset-item";
import { PageLayout } from "shared/src/components/layout/PageLayout";
import { PageHeader } from "shared/src/components/layout/PageHeader";
import { Skeleton } from "shared/src/components/ui/skeleton";
import AvatarImg from "shared/src/components/ui/avatar-img";
import {
  InnerSidebarLayout,
  InnerSidebar,
  InnerBody,
  InnerSidebarSection,
  InnerSidebarNav,
  InnerSidebarNavItem,
} from "shared/src/components/layout/InnerSidebarLayout";
import {
  Card,
  CardHeader,
  CardTitle,
  CardDescription,
  CardContent,
  CardFooter,
} from "shared/src/components/ui/card";
import { Badge } from "shared/src/components/ui/badge";
import { Button } from "shared/src/components/ui/button";
import { Input } from "shared/src/components/ui/input";
import { DataTable, type Column } from "shared/src/components/DataTable";
import { FormatDate } from "shared/src/components/ui/format-date";
import { formatUrn } from "shared/src/lib/utils";
import {
  Database,
  Server,
  ShieldCheck,
  FileCode,
  Plus,
  RefreshCw,
  LayoutGrid,
  Table as TableIcon,
  ArrowRight,
  ExternalLink,
  Copy,
  Check,
  CheckCircle2,
  Info,
  Globe,
  Calendar,
  Layers,
} from "lucide-react";

import { useGetCatalogById, useGetMainCatalogs } from "shared/src/data/orval/catalogs/catalogs";
import { useGetDatasetsByCatalogId } from "shared/src/data/orval/datasets/datasets";
import { useGetDataServicesByCatalogId } from "shared/src/data/orval/data-services/data-services";
import { useMyself } from "shared/src/data/useMyself";

export const Route = createFileRoute("/my-catalog/")({
  component: RouteComponent,
});

type CatalogTab = "datasets" | "dataservices" | "policies" | "metadata";

function RouteComponent() {
  const [activeTab, setActiveTab] = useState<CatalogTab>("datasets");
  const [viewMode, setViewMode] = useState<"grid" | "table">("grid");
  const [searchQuery, setSearchQuery] = useState("");
  const [copiedId, setCopiedId] = useState(false);

  const {
    data: mainCatalogData,
    isLoading: isCatalogLoading,
    refetch: refetchMainCatalog,
  } = useGetMainCatalogs();
  const mainCatalog = mainCatalogData?.status === 200 ? mainCatalogData.data : undefined;
  const catalogId = mainCatalog?.id;

  const { data: catalogData, refetch: refetchCatalog } = useGetCatalogById(catalogId ?? "");
  const { data: datasetsData, refetch: refetchDatasets } = useGetDatasetsByCatalogId(
    catalogId ?? "",
  );
  const { data: dataservicesData, refetch: refetchDataservices } = useGetDataServicesByCatalogId(
    catalogId ?? "",
  );

  const catalog = catalogData?.status === 200 ? catalogData.data : undefined;
  const datasets = datasetsData?.status === 200 ? datasetsData.data : [];
  const dataservices = dataservicesData?.status === 200 ? dataservicesData.data : [];

  const handleRefresh = () => {
    refetchMainCatalog();
    refetchCatalog();
    refetchDatasets();
    refetchDataservices();
  };

  const myAgent = useMyself();

  const myAgentSlug = myAgent?.participant_nick || "Local Agent";
  const mainDs = (dataservices || []).find((ds) => ds.dspaceMainDataService) ?? dataservices[0];
  const hasDataservice = !!mainDs;

  // Filter datasets for live search
  const filteredDatasets = useMemo(() => {
    if (!searchQuery.trim()) return datasets;
    const q = searchQuery.toLowerCase();
    return datasets.filter(
      (d) =>
        (d.dctTitle && d.dctTitle.toLowerCase().includes(q)) ||
        (d.dctDescription && d.dctDescription.toLowerCase().includes(q)) ||
        (d.id && d.id.toLowerCase().includes(q)),
    );
  }, [datasets, searchQuery]);

  const copyCatalogId = () => {
    if (catalog?.id) {
      navigator.clipboard.writeText(catalog.id);
      setCopiedId(true);
      setTimeout(() => setCopiedId(false), 2000);
    }
  };

  // Table columns definition for datasets
  const datasetColumns: Column<any>[] = [
    {
      header: "Dataset Title",
      accessorKey: "dctTitle",
      cell: (row) => (
        <div className="flex flex-col gap-0.5">
          <Link
            to="/catalog/$catalogId/dataset/$datasetId"
            params={{
              catalogId: catalog?.id ?? "",
              datasetId: row.id ?? "",
            }}
            className="font-semibold text-foreground hover:text-brand-sky underline-offset-2 hover:underline inline-flex items-center gap-1.5"
          >
            <span>{row.dctTitle || "Untitled Dataset"}</span>
            <ArrowRight className="h-3 w-3 opacity-60 flex-shrink-0" />
          </Link>
          <span className="text-xs text-muted-foreground line-clamp-1">
            {row.dctDescription || "No description provided"}
          </span>
        </div>
      ),
    },
    {
      header: "Identifier",
      accessorKey: "id",
      cell: (row) => (
        <Badge variant="code" className="text-xs font-mono">
          {formatUrn(row.id || "")}
        </Badge>
      ),
    },
    {
      header: "Distributions",
      cell: (row) => (
        <Badge
          variant="outline"
          className="text-xs font-mono text-sky-700 dark:text-sky-300 border-sky-500/20 bg-sky-500/10"
        >
          {Array.isArray((row as any).distribution) ? (row as any).distribution.length : 0} dists
        </Badge>
      ),
    },
    {
      header: "Policies",
      cell: (row) => (
        <Badge
          variant="outline"
          className="text-xs font-mono text-purple-700 dark:text-purple-300 border-purple-500/20 bg-purple-500/10"
        >
          {Array.isArray((row as any).hasPolicy) ? (row as any).hasPolicy.length : 0} rules
        </Badge>
      ),
    },
    {
      header: "Issued",
      accessorKey: "dctIssued",
      cell: (row) => <FormatDate date={row.dctIssued} />,
    },
    {
      header: "Action",
      cell: (row) => (
        <Link
          to="/catalog/$catalogId/dataset/$datasetId"
          params={{
            catalogId: catalog?.id ?? "",
            datasetId: row.id ?? "",
          }}
        >
          <Button variant="ghost" size="sm" className="h-7 text-xs px-2 gap-1 text-primary">
            <span>Inspect</span>
            <ArrowRight className="h-3 w-3" />
          </Button>
        </Link>
      ),
    },
  ];

  // Loading Skeleton State
  if (isCatalogLoading || !catalog) {
    return (
      <PageLayout>
        <PageHeader
          title="Myself's Catalog"
          badge={<Skeleton className="h-6 w-32 rounded-full" />}
        />
        <InnerSidebarLayout>
          <InnerSidebar>
            <div className="rounded-xl border border-ink/10 bg-card/60 p-4 space-y-4">
              <Skeleton className="h-10 w-full" />
              <Skeleton className="h-20 w-full" />
              <Skeleton className="h-8 w-full" />
            </div>
            <div className="rounded-xl border border-ink/10 bg-card/60 p-4 space-y-2">
              <Skeleton className="h-8 w-full" />
              <Skeleton className="h-8 w-full" />
              <Skeleton className="h-8 w-full" />
            </div>
          </InnerSidebar>
          <InnerBody>
            <div className="rounded-xl border border-ink/10 bg-card/60 p-6 space-y-4">
              <Skeleton className="h-10 w-64" />
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                <Skeleton className="h-44 w-full rounded-xl" />
                <Skeleton className="h-44 w-full rounded-xl" />
              </div>
            </div>
          </InnerBody>
        </InnerSidebarLayout>
      </PageLayout>
    );
  }

  return (
    <PageLayout>
      {/* Top Header */}
      <PageHeader
        title="Myself's Catalog"
        badge={
          <div className="flex items-center gap-2">
            <Badge variant="role" dsrole="Provider" className="text-xs uppercase">
              Local Node Catalog
            </Badge>
            <span className="hidden sm:inline-flex items-center gap-1 text-emerald-700 dark:text-emerald-400 text-xs font-mono">
              <CheckCircle2 className="h-3.5 w-3.5" /> Synchronized
            </span>
          </div>
        }
      />

      {/* Dual-Pane Layout: sidebar sec - body */}
      <InnerSidebarLayout>
        {/* Secondary Sidebar (Sidebar Sec) */}
        <InnerSidebar>
          {/* 1. Catalog Identity & Profile */}
          <InnerSidebarSection>
            <div className="flex items-start gap-3">
              <AvatarImg sizeClass="h-10 w-10" />
              <div className="min-w-0 flex-1">
                <h3 className="font-semibold text-foreground text-sm truncate capitalize">
                  {catalog.dctTitle || `${myAgentSlug}'s Catalog`}
                </h3>
                <p className="text-xs text-muted-foreground truncate font-mono">
                  {myAgent?.participant_nick || myAgentSlug}
                </p>
              </div>
            </div>

            <p className="text-xs text-muted-foreground leading-relaxed">
              Sovereign DCAT-AP catalog repository published by the local participant node in this
              dataspace.
            </p>

            <div className="pt-2 border-t border-ink/5 space-y-2 text-xs">
              <div className="flex justify-between items-center text-xs">
                <span className="text-muted-foreground flex items-center gap-1">
                  <Calendar className="h-3 w-3 opacity-60" />
                  Issued Date:
                </span>
                <FormatDate date={catalog.dctIssued} />
              </div>

              {catalog.foafHomePage && (
                <div className="flex justify-between items-center text-xs">
                  <span className="text-muted-foreground flex items-center gap-1">
                    <Globe className="h-3 w-3 opacity-60" />
                    Homepage:
                  </span>
                  <a
                    href={catalog.foafHomePage}
                    target="_blank"
                    rel="noreferrer"
                    className="text-brand-sky hover:underline truncate max-w-[150px] inline-flex items-center gap-1"
                  >
                    <span>{catalog.foafHomePage}</span>
                    <ExternalLink className="h-2.5 w-2.5" />
                  </a>
                </div>
              )}

              <div className="flex justify-between items-center text-xs">
                <span className="text-muted-foreground flex items-center gap-1">
                  <Info className="h-3 w-3 opacity-60" />
                  Catalog ID:
                </span>
                <button
                  type="button"
                  onClick={copyCatalogId}
                  className="font-mono text-muted-foreground hover:text-foreground inline-flex items-center gap-1 bg-ink/5 px-1.5 py-0.5 rounded border border-ink/5 transition-colors"
                  title="Click to copy full ID"
                >
                  <span>{catalog.id ? `#${catalog.id.slice(-8)}` : "—"}</span>
                  {copiedId ? (
                    <Check className="h-2.5 w-2.5 text-emerald-700 dark:text-emerald-400" />
                  ) : (
                    <Copy className="h-2.5 w-2.5 opacity-60" />
                  )}
                </button>
              </div>
            </div>
          </InnerSidebarSection>

          {/* 2. Secondary Navigation Tabs */}
          <InnerSidebarSection title="Navigation">
            <InnerSidebarNav>
              <InnerSidebarNavItem
                active={activeTab === "datasets"}
                onClick={() => setActiveTab("datasets")}
                icon={Database}
                label="Datasets"
                count={datasets.length}
              />
              <InnerSidebarNavItem
                active={activeTab === "dataservices"}
                onClick={() => setActiveTab("dataservices")}
                icon={Server}
                label="Data Services"
                count={dataservices.length}
              />
              <InnerSidebarNavItem
                active={activeTab === "policies"}
                onClick={() => setActiveTab("policies")}
                icon={ShieldCheck}
                label="Catalog Policies"
              />
              <InnerSidebarNavItem
                active={activeTab === "metadata"}
                onClick={() => setActiveTab("metadata")}
                icon={FileCode}
                label="DCAT-AP Specification"
              />
            </InnerSidebarNav>
          </InnerSidebarSection>

          {/* 3. Primary Data Service Card */}
          {hasDataservice && (
            <InnerSidebarSection title="Active Data Service">
              <div className="space-y-2">
                <div className="flex items-center justify-between">
                  <span className="text-xs font-medium text-foreground truncate">
                    {mainDs.dctTitle || "Dataspace Data Service"}
                  </span>
                  <Badge variant="status" state="active" className="text-xs">
                    Online
                  </Badge>
                </div>
                <div className="rounded-lg bg-background-800/60 p-2 text-xs font-mono text-muted-foreground break-all border border-ink/5">
                  {mainDs.dcatEndpointUrl || "No endpoint URL registered"}
                </div>
              </div>
            </InnerSidebarSection>
          )}

          {/* 4. Quick Actions */}
          <InnerSidebarSection title="Quick Actions">
            <div className="flex flex-col gap-2">
              <Link to="/my-catalog/new">
                <Button
                  size="sm"
                  variant="default"
                  className="w-full justify-start gap-2 text-xs font-medium"
                >
                  <Plus className="h-3.5 w-3.5" />
                  New Dataset Offering
                </Button>
              </Link>
              <Link to="/catalog/$catalogId" params={{ catalogId: catalog.id! }}>
                <Button size="sm" variant="default" className="w-full justify-start gap-2 text-xs">
                  <Layers className="h-3.5 w-3.5" />
                  Manage Offerings
                </Button>
              </Link>
              <Button
                size="sm"
                variant="outline"
                onClick={handleRefresh}
                className="w-full justify-start gap-2 text-xs"
              >
                <RefreshCw className="h-3.5 w-3.5" />
                Refresh Catalog Data
              </Button>
            </div>
          </InnerSidebarSection>
        </InnerSidebar>

        {/* Main Content Body (Body) */}
        <InnerBody>
          {/* TAB 1: DATASETS */}
          {activeTab === "datasets" && (
            <div className="flex flex-col gap-4">
              {/* Body Toolbar */}
              <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-3 pb-3 border-b border-ink/10">
                <div className="flex items-center gap-2">
                  <Heading level="h4" className="!mb-0 text-lg font-semibold">
                    Published Datasets
                  </Heading>
                  <Badge variant="code" className="text-xs">
                    {filteredDatasets.length} {filteredDatasets.length === 1 ? "item" : "items"}
                  </Badge>
                </div>

                <div className="flex items-center gap-2 w-full sm:w-auto">
                  {/* Search filter */}
                  <Input
                    placeholder="Search datasets..."
                    value={searchQuery}
                    onChange={(e) => setSearchQuery(e.target.value)}
                    onClear={() => setSearchQuery("")}
                    className="h-8 text-xs w-full sm:w-60"
                  />

                  {/* New Dataset Action */}
                  <Link to="/my-catalog/new">
                    <Button
                      size="sm"
                      variant="default"
                      className="h-8 text-xs gap-1.5 flex-shrink-0"
                    >
                      <Plus className="h-3.5 w-3.5" />
                      New Dataset
                    </Button>
                  </Link>

                  {/* View Mode Toggle */}
                  <div className="flex items-center border border-ink/10 rounded-lg p-0.5 bg-background-800/40 flex-shrink-0">
                    <button
                      type="button"
                      onClick={() => setViewMode("grid")}
                      className={`p-1.5 rounded-md text-xs transition-colors ${
                        viewMode === "grid"
                          ? "bg-primary/30 text-brand-sky shadow-sm"
                          : "text-muted-foreground hover:text-foreground"
                      }`}
                      title="Grid Cards View"
                    >
                      <LayoutGrid className="h-4 w-4" />
                    </button>
                    <button
                      type="button"
                      onClick={() => setViewMode("table")}
                      className={`p-1.5 rounded-md text-xs transition-colors ${
                        viewMode === "table"
                          ? "bg-primary/30 text-brand-sky shadow-sm"
                          : "text-muted-foreground hover:text-foreground"
                      }`}
                      title="Table View"
                    >
                      <TableIcon className="h-4 w-4" />
                    </button>
                  </div>
                </div>
              </div>

              {/* View Presentation */}
              {viewMode === "grid" ? (
                filteredDatasets.length > 0 ? (
                  <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                    {filteredDatasets.map((dataset) => (
                      <DatasetItem
                        key={dataset.id}
                        title={dataset.dctTitle ?? ""}
                        description={dataset.dctDescription ?? ""}
                        date={dataset.dctIssued ?? ""}
                        prevRoute={catalog.id ?? ""}
                        datasetId={dataset.id ?? ""}
                        ownDataset={true}
                        dataset={dataset}
                      />
                    ))}
                  </div>
                ) : (
                  <Card className="p-8 text-center flex flex-col items-center justify-center gap-3 border-dashed">
                    <Database className="h-10 w-10 text-muted-foreground/50" />
                    <div className="space-y-1">
                      <h4 className="font-semibold text-foreground text-sm">No datasets found</h4>
                      <p className="text-xs text-muted-foreground max-w-sm">
                        {searchQuery
                          ? `No dataset matches "${searchQuery}". Try clearing your search query.`
                          : "This catalog does not have any datasets registered yet."}
                      </p>
                    </div>
                    {searchQuery && (
                      <Button
                        size="sm"
                        variant="outline"
                        onClick={() => setSearchQuery("")}
                        className="text-xs mt-2"
                      >
                        Clear Search
                      </Button>
                    )}
                  </Card>
                )
              ) : (
                <DataTable
                  columns={datasetColumns}
                  data={filteredDatasets}
                  keyExtractor={(d) => d.id || String(d.dctTitle) || Math.random().toString()}
                  searchable={false}
                  pageSize={10}
                />
              )}
            </div>
          )}

          {/* TAB 2: DATA SERVICES */}
          {activeTab === "dataservices" && (
            <div className="flex flex-col gap-4">
              <div className="flex items-center justify-between pb-3 border-b border-ink/10">
                <Heading level="h4" className="!mb-0 text-lg font-semibold">
                  Registered Data Services
                </Heading>
                <Badge variant="code" className="text-xs">
                  {dataservices.length} {dataservices.length === 1 ? "service" : "services"}
                </Badge>
              </div>

              {dataservices.length > 0 ? (
                <div className="grid grid-cols-1 gap-4">
                  {dataservices.map((ds) => (
                    <Card key={ds.id} className="flex flex-col justify-between">
                      <CardHeader className="pb-3 space-y-2">
                        <div className="flex items-center justify-between">
                          <div className="flex items-center gap-2">
                            <Server className="h-5 w-5 text-brand-sky" />
                            <CardTitle className="text-base font-semibold">
                              {ds.dctTitle || "Data Service"}
                            </CardTitle>
                          </div>
                          {ds.dspaceMainDataService && (
                            <Badge variant="status" state="active">
                              Primary Service
                            </Badge>
                          )}
                        </div>
                        <CardDescription>
                          {ds.dctDescription ||
                            "Data service endpoint configured for this catalog."}
                        </CardDescription>
                      </CardHeader>

                      <CardContent className="space-y-3">
                        <div className="rounded-lg border border-ink/10 bg-background-800/40 p-3 space-y-2 text-xs">
                          <div className="flex justify-between items-center">
                            <span className="text-muted-foreground font-medium">Endpoint URL:</span>
                            <span className="font-mono text-brand-sky text-xs select-all">
                              {ds.dcatEndpointUrl || "N/A"}
                            </span>
                          </div>
                          <div className="flex justify-between items-center">
                            <span className="text-muted-foreground font-medium">Service ID:</span>
                            <span className="font-mono text-muted-foreground text-xs">
                              {formatUrn(ds.id || "")}
                            </span>
                          </div>
                          <div className="flex justify-between items-center">
                            <span className="text-muted-foreground font-medium">Issued Date:</span>
                            <FormatDate date={ds.dctIssued} />
                          </div>
                        </div>
                      </CardContent>

                      <CardFooter className="pt-3 border-t border-ink/5 flex justify-end">
                        <Link
                          to="/catalog/$catalogId/data-service/$dataServiceId"
                          params={{
                            catalogId: catalog.id!,
                            dataServiceId: ds.id!,
                          }}
                        >
                          <Button variant="outline" size="sm" className="gap-1.5 text-xs">
                            <span>Inspect Data Service</span>
                            <ArrowRight className="h-3.5 w-3.5" />
                          </Button>
                        </Link>
                      </CardFooter>
                    </Card>
                  ))}
                </div>
              ) : (
                <Card className="p-8 text-center flex flex-col items-center justify-center gap-3 border-dashed">
                  <Server className="h-10 w-10 text-muted-foreground/50" />
                  <div className="space-y-1">
                    <h4 className="font-semibold text-foreground text-sm">
                      No data services registered
                    </h4>
                    <p className="text-xs text-muted-foreground max-w-sm">
                      Register a data service to provide consumers with protocol endpoints.
                    </p>
                  </div>
                </Card>
              )}
            </div>
          )}

          {/* TAB 3: POLICIES */}
          {activeTab === "policies" && (
            <div className="flex flex-col gap-4">
              <div className="flex items-center justify-between pb-3 border-b border-ink/10">
                <Heading level="h4" className="!mb-0 text-lg font-semibold">
                  Catalog ODRL Policies Overview
                </Heading>
                <Badge variant="code" className="text-xs">
                  ODRL 2.2 Compatible
                </Badge>
              </div>

              <Card className="p-6 space-y-4">
                <div className="flex items-start gap-3">
                  <div className="p-2 rounded-lg bg-purple-500/10 text-purple-700 dark:text-purple-400">
                    <ShieldCheck className="h-5 w-5" />
                  </div>
                  <div>
                    <h4 className="font-semibold text-foreground text-sm">
                      Dataspace Policy Engine
                    </h4>
                    <p className="text-xs text-muted-foreground leading-relaxed mt-1">
                      ODRL policies specify the permissions, duties, and prohibitions governing
                      access to datasets in this catalog during automated contract negotiations.
                    </p>
                  </div>
                </div>

                <div className="grid grid-cols-1 sm:grid-cols-3 gap-3 pt-2">
                  <div className="rounded-lg border border-ink/10 bg-background-800/40 p-3 space-y-1">
                    <span className="text-xs uppercase font-mono text-muted-foreground">
                      Total Datasets
                    </span>
                    <p className="text-xl font-bold text-foreground">{datasets.length}</p>
                  </div>
                  <div className="rounded-lg border border-ink/10 bg-background-800/40 p-3 space-y-1">
                    <span className="text-xs uppercase font-mono text-muted-foreground">
                      Active Rules
                    </span>
                    <p className="text-xl font-bold text-purple-700 dark:text-purple-400">
                      {datasets.reduce(
                        (acc, d) =>
                          acc +
                          (Array.isArray((d as any).hasPolicy) ? (d as any).hasPolicy.length : 0),
                        0,
                      )}
                    </p>
                  </div>
                  <div className="rounded-lg border border-ink/10 bg-background-800/40 p-3 space-y-1">
                    <span className="text-xs uppercase font-mono text-muted-foreground">
                      Enforcement
                    </span>
                    <p className="text-xl font-bold text-emerald-700 dark:text-emerald-400">Autonomous</p>
                  </div>
                </div>
              </Card>
            </div>
          )}

          {/* TAB 4: METADATA */}
          {activeTab === "metadata" && (
            <div className="flex flex-col gap-4">
              <div className="flex items-center justify-between pb-3 border-b border-ink/10">
                <Heading level="h4" className="!mb-0 text-lg font-semibold">
                  DCAT-AP Catalog Specification
                </Heading>
                <Button
                  size="sm"
                  variant="outline"
                  onClick={() => {
                    navigator.clipboard.writeText(JSON.stringify(catalog, null, 2));
                    setCopiedId(true);
                    setTimeout(() => setCopiedId(false), 2000);
                  }}
                  className="h-7 text-xs gap-1.5"
                >
                  {copiedId ? (
                    <Check className="h-3 w-3 text-emerald-700 dark:text-emerald-400" />
                  ) : (
                    <Copy className="h-3 w-3" />
                  )}
                  <span>{copiedId ? "Copied" : "Copy JSON"}</span>
                </Button>
              </div>

              <Card className="p-4 bg-background-800/80 border-ink/10">
                <pre className="text-xs font-mono text-muted-foreground overflow-x-auto max-h-[600px] leading-relaxed">
                  {JSON.stringify(catalog, null, 2)}
                </pre>
              </Card>
            </div>
          )}
        </InnerBody>
      </InnerSidebarLayout>
    </PageLayout>
  );
}
