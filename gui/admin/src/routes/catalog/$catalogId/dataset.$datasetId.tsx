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

import React, { useState, useMemo } from "react";
import { createFileRoute, Link } from "@tanstack/react-router";
import {
  useGetDatasetById,
  getGetDatasetByIdQueryOptions,
} from "shared/src/data/orval/datasets/datasets";
import {
  useGetDistributionsByDatasetId,
  getGetDistributionsByDatasetIdQueryOptions,
} from "shared/src/data/orval/distributions/distributions";
import {
  useCreateOdrlPolicy,
  useGetPoliciesByEntityId,
} from "shared/src/data/orval/odrl-policies/odrl-policies";
import { useGetCatalogById } from "shared/src/data/orval/catalogs/catalogs";
import { OdrlPolicyInfo } from "shared/src/data/orval/model/odrlPolicyInfo";
import { DistributionDto } from "shared/src/data/orval/model";
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
  Card,
  CardHeader,
  CardTitle,
  CardDescription,
  CardContent,
} from "shared/src/components/ui/card";
import { Badge } from "shared/src/components/ui/badge";
import { Button } from "shared/src/components/ui/button";
import { Input } from "shared/src/components/ui/input";
import { DataTable, type Column } from "shared/src/components/DataTable";
import { FormatDate } from "shared/src/components/ui/format-date";
import { formatUrn } from "shared/src/lib/utils";
import {
  Drawer,
  DrawerContent,
  DrawerHeader,
  DrawerTitle,
  DrawerTrigger,
} from "shared/src/components/ui/drawer";
import { PolicyWrapperNew } from "shared/src/components/PolicyWrapperNew";
import { PolicyWrapperShow } from "shared/src/components/PolicyWrapperShow";
import DistributionItem from "shared/src/components/ui/distribution-item";
import Avatar from "shared/src/components/ui/avatar-img";
import Heading from "shared/src/components/ui/heading";
import {
  ArrowLeft,
  ArrowRight,
  Calendar,
  Check,
  Code2,
  Copy,
  Database,
  FileCode,
  Globe,
  Layers,
  LayoutGrid,
  Plus,
  RefreshCw,
  Server,
  ShieldCheck,
  Table as TableIcon,
  User,
} from "lucide-react";
import { useMyself } from "shared/src/data/useMyself";

type DatasetTab = "distributions" | "policies" | "metadata";

function RouteComponent() {
  const { catalogId, datasetId } = Route.useParams();
  const [activeTab, setActiveTab] = useState<DatasetTab>("distributions");
  const [viewMode, setViewMode] = useState<"grid" | "table">("grid");
  const [searchQuery, setSearchQuery] = useState("");
  const [copiedId, setCopiedId] = useState(false);
  const [drawerOpen, setDrawerOpen] = useState(false);

  const { data: catalogData, refetch: refetchCatalog } = useGetCatalogById(catalogId);
  const { data: datasetData, refetch: refetchDataset } = useGetDatasetById(datasetId);
  const { data: distributionsData, refetch: refetchDistributions } =
    useGetDistributionsByDatasetId(datasetId);
  const { data: policiesData, refetch: refetchPolicies } = useGetPoliciesByEntityId(datasetId);
  const { mutateAsync: createPolicyAsync } = useCreateOdrlPolicy();

  const dataset = datasetData?.status === 200 ? datasetData.data : undefined;
  const distributions = distributionsData?.status === 200 ? distributionsData.data : [];
  const policies = policiesData?.status === 200 ? policiesData.data : [];
  const catalog = catalogData?.status === 200 ? catalogData.data : undefined;

  const myAgent = useMyself();
  const myAgentSlug = myAgent?.participant_nick || "Local Agent";

  const handleCopyId = () => {
    if (!dataset?.id) return;
    navigator.clipboard.writeText(dataset.id);
    setCopiedId(true);
    setTimeout(() => setCopiedId(false), 2000);
  };

  const handleRefresh = () => {
    refetchCatalog();
    refetchDataset();
    refetchDistributions();
    refetchPolicies();
  };

  const onSubmitPolicy = async (data: OdrlPolicyInfo, description?: string) => {
    await createPolicyAsync({
      data: {
        entityId: datasetId,
        entityType: "Dataset",
        odrlOffer: data,
        description,
      },
    });
    setDrawerOpen(false);
    refetchPolicies();
  };

  const filteredDistributions = useMemo(() => {
    if (!searchQuery.trim()) return distributions;
    const q = searchQuery.toLowerCase();
    return distributions.filter(
      (d) =>
        d.dctTitle?.toLowerCase().includes(q) ||
        d.dctDescription?.toLowerCase().includes(q) ||
        d.dctFormat?.toLowerCase().includes(q) ||
        d.id?.toLowerCase().includes(q),
    );
  }, [distributions, searchQuery]);

  const filteredPolicies = useMemo(() => {
    if (!searchQuery.trim()) return policies;
    const q = searchQuery.toLowerCase();
    return policies.filter(
      (p) =>
        p.description?.toLowerCase().includes(q) ||
        p.id?.toLowerCase().includes(q) ||
        p.entity?.toLowerCase().includes(q),
    );
  }, [policies, searchQuery]);

  const distributionColumns: Column<DistributionDto>[] = [
    {
      header: "Title & Identifier",
      cell: (d) => (
        <div>
          <p className="font-semibold text-foreground">{d.dctTitle || "Unnamed Distribution"}</p>
          <span className="font-mono text-xs text-muted-foreground">{formatUrn(d.id || "")}</span>
        </div>
      ),
    },
    {
      header: "Format / MIME",
      cell: (d) => (
        <Badge variant="code" className="font-mono text-xs">
          {d.dctFormat || "application/json"}
        </Badge>
      ),
    },
    {
      header: "Access Data Service",
      cell: (d) =>
        d.dcatAccessService ? (
          <Link
            to="/catalog/$catalogId/data-service/$dataServiceId"
            params={{
              catalogId,
              dataServiceId: d.dcatAccessService,
            }}
            className="text-emerald-700 dark:text-emerald-400 hover:underline font-mono text-xs flex items-center gap-1"
          >
            <Server className="h-3 w-3" />
            <span>{formatUrn(d.dcatAccessService)}</span>
          </Link>
        ) : (
          <span className="text-muted-foreground text-xs italic">Default Gateway</span>
        ),
    },
    {
      header: "Issued",
      cell: (d) => (
        <span className="text-xs text-muted-foreground">
          <FormatDate date={d.dctIssued} />
        </span>
      ),
    },
    {
      header: "Action",
      cell: (d) => (
        <Link
          to="/catalog/$catalogId/distribution-connector/$distributionId"
          params={{
            catalogId,
            distributionId: d.id!,
          }}
          className="inline-flex items-center gap-1 text-primary hover:underline font-mono text-xs font-semibold group"
        >
          <span>Connector Instance</span>
          <ArrowRight className="h-3.5 w-3.5 transition-transform group-hover:translate-x-0.5" />
        </Link>
      ),
    },
  ];

  if (!dataset) return null;

  return (
    <PageLayout>
      <InnerSidebarLayout>
        {/* Secondary Left Sidebar */}
        <InnerSidebar>
          <Card className="border-ink/10 bg-background-800/40">
            <CardHeader className="pb-3 space-y-2">
              <div className="flex items-center justify-between">
                <Badge variant="outline" className="border-brand-sky/30 text-brand-sky text-xs">
                  Dataset Offering
                </Badge>
                <Button
                  variant="ghost"
                  size="icon"
                  className="h-7 w-7 text-muted-foreground hover:text-foreground"
                  onClick={handleCopyId}
                  title="Copy Dataset URN"
                >
                  {copiedId ? (
                    <Check className="h-3.5 w-3.5 text-emerald-700 dark:text-emerald-400" />
                  ) : (
                    <Copy className="h-3.5 w-3.5" />
                  )}
                </Button>
              </div>

              <CardTitle className="text-lg font-semibold leading-tight">
                {dataset.dctTitle}
              </CardTitle>

              <CardDescription className="text-xs line-clamp-3">
                {dataset.dctDescription || "No description provided for this dataset."}
              </CardDescription>
            </CardHeader>

            <CardContent className="space-y-3 pt-0 text-xs">
              <div className="p-2 rounded-lg bg-sunken/20 border border-ink/5 space-y-1 font-mono text-xs">
                <span className="text-muted-foreground block">URN:</span>
                <span className="text-brand-sky break-all">{formatUrn(dataset.id || "")}</span>
              </div>

              {dataset.dctConformsTo && (
                <div className="flex justify-between items-center text-xs">
                  <span className="text-muted-foreground flex items-center gap-1">
                    <Globe className="h-3.5 w-3.5 opacity-60" />
                    Conforms To:
                  </span>
                  <span className="font-mono text-foreground/80 truncate max-w-[150px]">
                    {dataset.dctConformsTo}
                  </span>
                </div>
              )}

              {dataset.dctCreator && (
                <div className="flex justify-between items-center text-xs">
                  <span className="text-muted-foreground flex items-center gap-1">
                    <User className="h-3.5 w-3.5 opacity-60" />
                    Creator DID:
                  </span>
                  <span className="font-mono text-foreground/80 truncate max-w-[150px]">
                    {dataset.dctCreator}
                  </span>
                </div>
              )}

              {dataset.dctIssued && (
                <div className="flex justify-between items-center text-xs">
                  <span className="text-muted-foreground flex items-center gap-1">
                    <Calendar className="h-3.5 w-3.5 opacity-60" />
                    Issued:
                  </span>
                  <FormatDate date={dataset.dctIssued} />
                </div>
              )}

              <div className="pt-2 border-t border-ink/10 space-y-2">
                <span className="text-xs text-muted-foreground font-semibold uppercase tracking-wider block">
                  Publisher Participant
                </span>
                <div className="flex items-center gap-2 p-2 rounded-lg bg-ink/5 border border-ink/5">
                  <Avatar />
                  <div className="min-w-0">
                    <p className="font-semibold text-xs capitalize truncate">{myAgentSlug}</p>
                    <p className="text-xs text-muted-foreground">Local Dataspace Agent</p>
                  </div>
                </div>
              </div>

              <div className="pt-2 border-t border-ink/10 space-y-1">
                <span className="text-xs text-muted-foreground font-semibold uppercase tracking-wider block">
                  Parent Catalog
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
              </div>
            </CardContent>
          </Card>

          {/* Navigation section */}
          <InnerSidebarSection title="Views & Resources">
            <InnerSidebarNav>
              <InnerSidebarNavItem
                icon={Layers}
                label="Distributions"
                count={distributions.length}
                active={activeTab === "distributions"}
                onClick={() => setActiveTab("distributions")}
              />
              <InnerSidebarNavItem
                icon={ShieldCheck}
                label="Access Policies"
                count={policies.length}
                active={activeTab === "policies"}
                onClick={() => setActiveTab("policies")}
              />
              <InnerSidebarNavItem
                icon={FileCode}
                label="DCAT Metadata"
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
            <Link
              to="/catalog/$catalogId"
              params={{ catalogId }}
              className="inline-flex items-center gap-1.5 text-xs text-muted-foreground hover:text-foreground transition-colors"
            >
              <ArrowLeft className="h-3.5 w-3.5" />
              <span>Back to Catalog</span>
            </Link>

            <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
              <div>
                <div className="flex items-center gap-2 flex-wrap">
                  <Heading level="h2" className="!mb-0">
                    {dataset.dctTitle}
                  </Heading>
                  <Badge
                    variant="outline"
                    className="font-mono text-xs border-brand-sky/40 text-brand-sky"
                  >
                    {formatUrn(dataset.id || "")}
                  </Badge>
                </div>
                <p className="text-xs text-muted-foreground mt-1">
                  Catalog:{" "}
                  <span className="text-foreground font-medium">
                    {catalog?.dctTitle || catalogId}
                  </span>
                </p>
              </div>

              <div className="flex items-center gap-2 flex-wrap">
                <Button
                  variant="outline"
                  size="sm"
                  onClick={handleRefresh}
                  className="gap-1.5 text-xs"
                >
                  <RefreshCw className="h-3.5 w-3.5" />
                  Refresh
                </Button>

                <Drawer direction="right" open={drawerOpen} onOpenChange={setDrawerOpen}>
                  <DrawerTrigger asChild>
                    <Button size="sm" className="gap-1.5 text-xs">
                      <Plus className="h-3.5 w-3.5" />
                      Add Policy
                    </Button>
                  </DrawerTrigger>
                  <DrawerContent>
                    <DrawerHeader className="px-8 border-b border-ink/10 pb-4 mb-4">
                      <DrawerTitle className="flex flex-col gap-1">
                        <span className="text-lg font-semibold">New ODRL Policy</span>
                        <div className="flex items-center text-sm font-normal text-muted-foreground">
                          for Dataset
                          <Badge variant="info" size="sm" className="ml-2 font-mono">
                            {formatUrn(dataset.id!)}
                          </Badge>
                        </div>
                      </DrawerTitle>
                    </DrawerHeader>
                    <PolicyWrapperNew onSubmit={onSubmitPolicy} />
                  </DrawerContent>
                </Drawer>
              </div>
            </div>
          </div>

          {/* Search & View Controls */}
          {activeTab !== "metadata" && (
            <div className="flex flex-col sm:flex-row items-center justify-between gap-3">
              <Input
                placeholder={`Search ${activeTab}...`}
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                className="max-w-sm text-xs"
              />

              {activeTab === "distributions" && (
                <div className="flex items-center gap-1 bg-ink/5 border border-ink/10 rounded-lg p-1">
                  <Button
                    variant={viewMode === "grid" ? "default" : "ghost"}
                    size="icon"
                    className="h-7 w-7"
                    onClick={() => setViewMode("grid")}
                    title="Grid View"
                  >
                    <LayoutGrid className="h-3.5 w-3.5" />
                  </Button>
                  <Button
                    variant={viewMode === "table" ? "default" : "ghost"}
                    size="icon"
                    className="h-7 w-7"
                    onClick={() => setViewMode("table")}
                    title="Table View"
                  >
                    <TableIcon className="h-3.5 w-3.5" />
                  </Button>
                </div>
              )}
            </div>
          )}

          {/* TAB 1: DISTRIBUTIONS */}
          {activeTab === "distributions" && (
            <div className="space-y-4">
              {filteredDistributions.length === 0 ? (
                <Card className="border-dashed border-ink/10 p-8 text-center bg-transparent">
                  <Layers className="h-8 w-8 text-muted-foreground/50 mx-auto mb-2" />
                  <p className="text-sm font-medium text-foreground">No distributions found</p>
                  <p className="text-xs text-muted-foreground mt-1">
                    This dataset does not have any active data distributions configured yet.
                  </p>
                </Card>
              ) : viewMode === "grid" ? (
                <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                  {filteredDistributions.map((distribution) => (
                    <DistributionItem
                      key={distribution.id}
                      title={distribution.dctTitle}
                      description={distribution.dctDescription}
                      date={distribution.dctIssued}
                      ownDataset={true}
                      prevRoute={catalogId}
                      distribuionId={distribution.id}
                      dataserviceId={distribution.dcatAccessService}
                    />
                  ))}
                </div>
              ) : (
                <DataTable
                  searchable={false}
                  data={filteredDistributions}
                  columns={distributionColumns}
                  keyExtractor={(d) => d.id || ""}
                />
              )}
            </div>
          )}

          {/* TAB 2: ACCESS POLICIES */}
          {activeTab === "policies" && (
            <div className="space-y-4">
              {filteredPolicies.length === 0 ? (
                <Card className="border-dashed border-ink/10 p-8 text-center bg-transparent">
                  <ShieldCheck className="h-8 w-8 text-muted-foreground/50 mx-auto mb-2" />
                  <p className="text-sm font-medium text-foreground">No policies attached</p>
                  <p className="text-xs text-muted-foreground mt-1">
                    Attach an ODRL policy rule to define usage and access constraints for consumers.
                  </p>
                  <Button
                    size="sm"
                    onClick={() => setDrawerOpen(true)}
                    className="mt-4 gap-1 text-xs"
                  >
                    <Plus className="h-3.5 w-3.5" />
                    Attach New Policy
                  </Button>
                </Card>
              ) : (
                <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                  {filteredPolicies.map((policy) => (
                    <PolicyWrapperShow
                      key={policy.id}
                      policy={policy}
                      datasetId={dataset.id!}
                      catalogId={catalogId}
                      datasetName={dataset.dctTitle}
                      showOfferAccess
                    />
                  ))}
                </div>
              )}
            </div>
          )}

          {/* TAB 3: DCAT METADATA */}
          {activeTab === "metadata" && (
            <div className="space-y-6">
              <Card className="border-ink/10 bg-background-800/40">
                <CardHeader className="pb-3">
                  <CardTitle className="text-sm font-semibold flex items-center gap-2">
                    <FileCode className="h-4 w-4 text-brand-sky" />
                    DCAT Specification Properties
                  </CardTitle>
                </CardHeader>
                <CardContent className="pt-0">
                  <div className="rounded-lg border border-ink/10 divide-y divide-ink/5 text-xs">
                    <div className="flex flex-col sm:flex-row sm:items-center justify-between p-3 gap-1">
                      <span className="text-muted-foreground">Identifier (@id)</span>
                      <span className="font-mono text-brand-sky text-xs break-all">
                        {dataset.id}
                      </span>
                    </div>
                    <div className="flex flex-col sm:flex-row sm:items-center justify-between p-3 gap-1">
                      <span className="text-muted-foreground">Title (dct:title)</span>
                      <span className="font-medium text-foreground">{dataset.dctTitle}</span>
                    </div>
                    <div className="flex flex-col sm:flex-row sm:items-center justify-between p-3 gap-1">
                      <span className="text-muted-foreground">Description (dct:description)</span>
                      <span className="text-foreground max-w-xl text-right">
                        {dataset.dctDescription || "—"}
                      </span>
                    </div>
                    <div className="flex flex-col sm:flex-row sm:items-center justify-between p-3 gap-1">
                      <span className="text-muted-foreground">
                        Profile Standard (dct:conformsTo)
                      </span>
                      <span className="font-mono text-foreground/80">
                        {dataset.dctConformsTo || "—"}
                      </span>
                    </div>
                    <div className="flex flex-col sm:flex-row sm:items-center justify-between p-3 gap-1">
                      <span className="text-muted-foreground">
                        Creator / Publisher DID (dct:creator)
                      </span>
                      <span className="font-mono text-foreground/80">
                        {dataset.dctCreator || "—"}
                      </span>
                    </div>
                    <div className="flex flex-col sm:flex-row sm:items-center justify-between p-3 gap-1">
                      <span className="text-muted-foreground">Issued Date (dct:issued)</span>
                      <FormatDate date={dataset.dctIssued} />
                    </div>
                    <div className="flex flex-col sm:flex-row sm:items-center justify-between p-3 gap-1">
                      <span className="text-muted-foreground">Modified Date (dct:modified)</span>
                      <FormatDate date={dataset.dctModified} />
                    </div>
                    <div className="flex flex-col sm:flex-row sm:items-center justify-between p-3 gap-1">
                      <span className="text-muted-foreground">Parent Catalog Reference</span>
                      <span className="font-mono text-xs text-muted-foreground">
                        {dataset.catalogId || catalogId}
                      </span>
                    </div>
                  </div>
                </CardContent>
              </Card>

              {/* Raw JSON Payload */}
              <details className="rounded-xl border border-ink/10 bg-background-800/40 p-4 text-xs">
                <summary className="cursor-pointer font-medium text-muted-foreground hover:text-foreground flex items-center gap-2">
                  <Code2 className="h-4 w-4" />
                  Inspect Raw Dataset Specification (JSON)
                </summary>
                <pre className="mt-3 p-4 rounded-lg bg-sunken/40 font-mono text-xs text-brand-sky overflow-x-auto border border-ink/5">
                  {JSON.stringify(dataset, null, 2)}
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
 * Route for displaying dataset details.
 */
export const Route = createFileRoute("/catalog/$catalogId/dataset/$datasetId")({
  component: RouteComponent,
  pendingComponent: () => <div>Loading...</div>,
  loader: async ({ context: { queryClient }, params: { datasetId } }) => {
    await queryClient.ensureQueryData(getGetDatasetByIdQueryOptions(datasetId));
    return queryClient.ensureQueryData(getGetDistributionsByDatasetIdQueryOptions(datasetId));
  },
});
