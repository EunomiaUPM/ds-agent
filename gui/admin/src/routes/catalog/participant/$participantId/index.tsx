import { createFileRoute, Link } from "@tanstack/react-router";
import { useRpcSetupCatalogRequest } from "shared/src/data/orval/catalog-rp-c/catalog-rp-c";
import { PageLayout } from "shared/src/components/layout/PageLayout";
import { PageHeader } from "shared/src/components/layout/PageHeader";
import { FormatDate } from "shared/src/components/ui/format-date";
import { formatUrn } from "shared/src/lib/utils";
import React, { useEffect, useState, useMemo } from "react";
import { Skeleton } from "shared/src/components/ui/skeleton";
import Heading from "shared/src/components/ui/heading";
import DatasetItem from "shared/src/components/ui/dataset-item";
import Avatar from "shared/components/ui/avatar-img";
import { useFederatedCatalog } from "shared/data/useFederatedCatalog";
import { Badge } from "shared/src/components/ui/badge";
import { Button } from "shared/src/components/ui/button";
import { Input } from "shared/src/components/ui/input";
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
import {
  Database,
  Server,
  ArrowLeft,
  Calendar,
  Layers,
  ArrowRight,
  CheckCircle2,
  ShieldCheck,
  Search,
} from "lucide-react";
import { useMyself } from "shared/src/data/useMyself";

function RouteComponent() {
  const { participantId } = Route.useParams();
  const federated = useFederatedCatalog();
  const { mutate, data, isPending, error } = useRpcSetupCatalogRequest();
  const [searchQuery, setSearchQuery] = useState("");
  const [activeTab, setActiveTab] = useState<"datasets" | "dataservice">("datasets");

  const myAgent = useMyself();

  const participant =
    federated.state === "ok"
      ? federated.agents.find((p: any) => p.participant_id === participantId)
      : undefined;

  useEffect(() => {
    mutate({
      data: {
        associatedAgentPeer: participantId,
        filter: [],
        noCache: true,
      },
    });
  }, [participantId, mutate]);

  const catalog = data?.status === 200 ? data.data : undefined;
  const datasets = Array.isArray(catalog?.response?.dataset) ? catalog.response.dataset : [];
  const dataservice = catalog?.response?.service as any;

  const filteredDatasets = useMemo(() => {
    if (!searchQuery.trim()) return datasets;
    const q = searchQuery.toLowerCase();
    return datasets.filter(
      (d: any) =>
        (d.title && d.title.toLowerCase().includes(q)) ||
        (d.dctDescription && d.dctDescription.toLowerCase().includes(q)) ||
        (d["@id"] && d["@id"].toLowerCase().includes(q)),
    );
  }, [datasets, searchQuery]);

  if (federated.state === "loading" || isPending) {
    return (
      <PageLayout>
        <PageHeader
          title="Participant Catalog"
          badge={<Skeleton className="h-6 w-36 rounded-full" />}
        />
        <InnerSidebarLayout>
          <InnerSidebar>
            <div className="rounded-xl border border-ink/10 bg-card/60 p-4 space-y-4">
              <Skeleton className="h-10 w-full" />
              <Skeleton className="h-20 w-full" />
            </div>
          </InnerSidebar>
          <InnerBody>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <Skeleton className="h-44 w-full rounded-xl" />
              <Skeleton className="h-44 w-full rounded-xl" />
            </div>
          </InnerBody>
        </InnerSidebarLayout>
      </PageLayout>
    );
  }

  if (federated.state === "no-authority") {
    return (
      <PageLayout>
        <Card className="p-8 text-center max-w-lg mx-auto">
          <CardTitle>No Authority Found</CardTitle>
          <CardDescription className="mt-2">
            Connect to an authority node to browse federated catalogs.
          </CardDescription>
        </Card>
      </PageLayout>
    );
  }

  if (error || federated.state === "error") {
    return (
      <PageLayout>
        <Card className="p-8 text-center max-w-lg mx-auto border-destructive/40">
          <CardTitle className="text-destructive">Catalog Discovery Failed</CardTitle>
          <CardDescription className="mt-2 text-muted-foreground">
            {error?.message || "Failed to resolve federated catalog for this participant."}
          </CardDescription>
          <Button variant="outline" className="mt-4" onClick={() => window.location.reload()}>
            Retry
          </Button>
        </Card>
      </PageLayout>
    );
  }

  if (!catalog) return null;

  const catalogTitle =
    catalog.response?.title || `${participant?.participant_nick || "Participant"}'s Catalog`;
  const isOwnCatalog = participant?.participant_id === myAgent?.participant_id;

  return (
    <PageLayout>
      <PageHeader
        title={catalogTitle}
        badge={
          isOwnCatalog ? (
            <Badge variant="role" dsrole="Provider">
              My Own Catalog
            </Badge>
          ) : (
            <Badge variant="status" state="active">
              Peer Catalog
            </Badge>
          )
        }
      >
        <div className="pt-2">
          <Link to="/catalog">
            <Button
              variant="ghost"
              size="sm"
              className="h-7 text-xs px-2 gap-1 text-muted-foreground hover:text-foreground"
            >
              <ArrowLeft className="h-3.5 w-3.5" />
              <span>Back to Catalogs</span>
            </Button>
          </Link>
        </div>
      </PageHeader>

      <InnerSidebarLayout>
        {/* Secondary Sidebar (Sidebar Sec) */}
        <InnerSidebar>
          {/* Participant Profile Section */}
          <InnerSidebarSection>
            <div className="flex items-center gap-3">
              <Avatar sizeClass="h-10 w-10" />
              <div className="min-w-0 flex-1">
                <h3 className="font-semibold text-foreground text-sm truncate capitalize">
                  {participant?.participant_nick || "Peer Participant"}
                </h3>
                <p className="text-xs text-muted-foreground truncate font-mono">
                  {participantId ? `#${participantId.slice(-10)}` : ""}
                </p>
              </div>
            </div>

            <p className="text-xs text-muted-foreground leading-relaxed">
              Federated catalog offered by this peer participant within the sovereign dataspace
              network.
            </p>

            <div className="pt-2 border-t border-ink/5 space-y-2 text-xs">
              <div className="flex justify-between items-center text-xs">
                <span className="text-muted-foreground flex items-center gap-1">
                  <Calendar className="h-3 w-3 opacity-60" />
                  Issued Date:
                </span>
                <FormatDate date={catalog.response?.issued} />
              </div>

              {catalog.response?.homepage && (
                <div className="flex justify-between items-center text-xs">
                  <span className="text-muted-foreground">Homepage:</span>
                  <a
                    href={catalog.response.homepage}
                    target="_blank"
                    rel="noreferrer"
                    className="text-brand-sky hover:underline truncate max-w-[150px]"
                  >
                    {catalog.response.homepage}
                  </a>
                </div>
              )}
            </div>
          </InnerSidebarSection>

          {/* Navigation */}
          <InnerSidebarSection title="Navigation">
            <InnerSidebarNav>
              <InnerSidebarNavItem
                active={activeTab === "datasets"}
                onClick={() => setActiveTab("datasets")}
                icon={Database}
                label="Offered Datasets"
                count={datasets.length}
              />
              {dataservice && (
                <InnerSidebarNavItem
                  active={activeTab === "dataservice"}
                  onClick={() => setActiveTab("dataservice")}
                  icon={Server}
                  label="Data Service"
                  count={1}
                />
              )}
            </InnerSidebarNav>
          </InnerSidebarSection>

          {/* Dataservice Quick Info */}
          {dataservice && (
            <InnerSidebarSection title="Dataservice Endpoint">
              <div className="space-y-1.5 text-xs">
                <div className="flex items-center justify-between">
                  <span className="font-medium text-foreground text-xs truncate">
                    {dataservice.title || "DCAT Dataservice"}
                  </span>
                  <Badge variant="status" state="active" className="text-xs">
                    Active
                  </Badge>
                </div>
                <div className="rounded bg-background-800/60 p-2 text-xs font-mono text-muted-foreground break-all border border-ink/5">
                  {dataservice.endpointURL || "No endpoint URL registered"}
                </div>
              </div>
            </InnerSidebarSection>
          )}
        </InnerSidebar>

        {/* Main Content Body (Body) */}
        <InnerBody>
          {activeTab === "datasets" && (
            <div className="flex flex-col gap-4">
              {/* Header Toolbar */}
              <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-3 pb-3 border-b border-ink/10">
                <div className="flex items-center gap-2">
                  <Heading level="h4" className="!mb-0 text-lg font-semibold">
                    Offered Datasets
                  </Heading>
                  <Badge variant="code" className="text-xs">
                    {filteredDatasets.length}{" "}
                    {filteredDatasets.length === 1 ? "dataset" : "datasets"}
                  </Badge>
                </div>

                <Input
                  placeholder="Filter datasets..."
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  onClear={() => setSearchQuery("")}
                  className="h-8 text-xs w-full sm:w-60"
                />
              </div>

              {filteredDatasets.length > 0 ? (
                <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                  {filteredDatasets.map((dataset: any) => (
                    <DatasetItem
                      key={dataset["@id"]!}
                      title={dataset.title!}
                      description={dataset.dctDescription!}
                      date={dataset.issued!}
                      prevRoute={participantId}
                      datasetId={dataset["@id"]!}
                      ownDataset={false}
                      dataset={dataset}
                    />
                  ))}
                </div>
              ) : (
                <Card className="p-8 text-center flex flex-col items-center justify-center gap-3 border-dashed">
                  <Database className="h-10 w-10 text-muted-foreground/50" />
                  <div className="space-y-1">
                    <h4 className="font-semibold text-foreground text-sm">No datasets available</h4>
                    <p className="text-xs text-muted-foreground max-w-sm">
                      {searchQuery
                        ? `No dataset matches "${searchQuery}".`
                        : "This participant currently has no public datasets in this catalog."}
                    </p>
                  </div>
                  {searchQuery && (
                    <Button
                      size="sm"
                      variant="outline"
                      onClick={() => setSearchQuery("")}
                      className="text-xs mt-2"
                    >
                      Clear Filter
                    </Button>
                  )}
                </Card>
              )}
            </div>
          )}

          {activeTab === "dataservice" && dataservice && (
            <div className="flex flex-col gap-4">
              <div className="pb-3 border-b border-ink/10">
                <Heading level="h4" className="!mb-0 text-lg font-semibold">
                  Dataservice Details
                </Heading>
              </div>

              <Card className="p-6 space-y-4">
                <div className="flex items-center gap-3">
                  <div className="p-2.5 rounded-lg bg-primary/10 text-brand-sky">
                    <Server className="h-6 w-6" />
                  </div>
                  <div>
                    <h4 className="font-semibold text-foreground text-base">
                      {dataservice.title || "Federated Dataservice"}
                    </h4>
                    <p className="text-xs text-muted-foreground font-mono">
                      {formatUrn(dataservice["@id"] || "")}
                    </p>
                  </div>
                </div>

                <div className="rounded-lg border border-ink/10 bg-background-800/40 p-4 space-y-3 text-xs">
                  <div className="flex justify-between items-center">
                    <span className="text-muted-foreground font-medium">Endpoint URL:</span>
                    <span className="font-mono text-brand-sky text-xs select-all">
                      {dataservice.endpointURL || "N/A"}
                    </span>
                  </div>
                  <div className="flex justify-between items-center">
                    <span className="text-muted-foreground font-medium">Service Type:</span>
                    <Badge variant="outline" className="text-xs font-mono">
                      DCAT-AP DataService
                    </Badge>
                  </div>
                </div>
              </Card>
            </div>
          )}
        </InnerBody>
      </InnerSidebarLayout>
    </PageLayout>
  );
}

export const Route = createFileRoute("/catalog/participant/$participantId/")({
  component: RouteComponent,
});
