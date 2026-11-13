import { createFileRoute, Link, useNavigate } from "@tanstack/react-router";
import React, { useState, useMemo } from "react";
import DatasetItem from "shared/src/components/ui/dataset-item";
import { CountBadge } from "@eunomia/ui";
import { PageLayout } from "@eunomia/ui";
import { PageHeader } from "@eunomia/ui";
import { Skeleton } from "@eunomia/ui";
import { Badge } from "@eunomia/ui";
import { Button } from "@eunomia/ui";
import { Input } from "@eunomia/ui";
import { DataTable, type Column } from "@eunomia/ui";
import { FieldGrid, Identifier } from "@eunomia/ui";
import { FormatDate } from "@eunomia/ui";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@eunomia/ui";
import {
  ExternalLink,
  Eye,
  LayoutGrid,
  Pencil,
  Plus,
  Table as TableIcon,
  Trash2,
  Upload,
} from "lucide-react";

import { useGetCatalogById, useGetMainCatalogs } from "shared/src/data/orval/catalogs/catalogs";
import { useGetDatasetsByCatalogId } from "shared/src/data/orval/datasets/datasets";
import {
  CatalogViews,
  type CatalogDoc,
  type DataServiceView,
  type DatasetView,
} from "shared/src/data/catalog";
import {
  useGetDataServicesByCatalogId,
  useGetMainDataService,
} from "shared/src/data/orval/data-services/data-services";
import { useMyself } from "shared/src/data/useMyself";
import { CatalogSpecificationPanel } from "shared/src/components/catalog/CatalogSpecificationPanel";
import { PublicationBadge } from "shared/src/components/catalog/Publication";
import {
  CatalogResourceEditor,
  type EditorTarget,
} from "shared/src/components/catalog/CatalogResourceEditor";
import {
  DeleteResourceDialog,
  type DeleteTarget,
} from "shared/src/components/catalog/DeleteResourceDialog";

export const Route = createFileRoute("/my-catalog/")({
  component: RouteComponent,
});

type CatalogTab = "datasets" | "dataservices" | "specification";
type ViewMode = "grid" | "table";

const VIEW_MODES: { mode: ViewMode; icon: typeof LayoutGrid; title: string }[] = [
  { mode: "grid", icon: LayoutGrid, title: "Cards" },
  { mode: "table", icon: TableIcon, title: "Table" },
];

/** How many of `key` a dataset document embeds. */
const countOf = (row: DatasetView, key: string): number => {
  const value = (row as unknown as Record<string, unknown>)[key];
  return Array.isArray(value) ? value.length : 0;
};

function RouteComponent() {
  const navigate = useNavigate();
  const [activeTab, setActiveTab] = useState<CatalogTab>("datasets");
  const [viewMode, setViewMode] = useState<ViewMode>("grid");
  const [searchQuery, setSearchQuery] = useState("");
  const [editing, setEditing] = useState<EditorTarget | null>(null);
  const [deleting, setDeleting] = useState<DeleteTarget | null>(null);

  const { data: mainCatalogData, isLoading: isCatalogLoading } = useGetMainCatalogs();
  const mainCatalog =
    mainCatalogData?.status === 200
      ? CatalogViews.catalog(mainCatalogData.data as CatalogDoc)
      : undefined;
  const catalogId = mainCatalog?.id;

  const { data: catalogData } = useGetCatalogById(catalogId ?? "");
  const { data: datasetsData } = useGetDatasetsByCatalogId(catalogId ?? "");
  const { data: dataservicesData } = useGetDataServicesByCatalogId(catalogId ?? "");

  const catalog =
    catalogData?.status === 200 ? CatalogViews.catalog(catalogData.data as CatalogDoc) : undefined;
  const datasets =
    datasetsData?.status === 200 ? CatalogViews.list(datasetsData.data, CatalogViews.dataset) : [];
  const dataservices =
    dataservicesData?.status === 200
      ? CatalogViews.list(dataservicesData.data, CatalogViews.dataService)
      : [];

  const myAgent = useMyself();
  const { data: mainServiceData } = useGetMainDataService();
  const mainServiceId =
    mainServiceData?.status === 200
      ? CatalogViews.dataService(mainServiceData.data as CatalogDoc).id
      : "";
  // The main data service is the connector's first, created with the main catalog.
  const mainDs = dataservices.find((ds) => ds.id === mainServiceId) ?? dataservices[0];

  const filteredDatasets = useMemo(() => {
    if (!searchQuery.trim()) return datasets;
    const q = searchQuery.toLowerCase();
    return datasets.filter(
      (d) =>
        (d.title && d.title.toLowerCase().includes(q)) ||
        (d.description && d.description.toLowerCase().includes(q)) ||
        d.keywords.some((k) => k.toLowerCase().includes(q)) ||
        (d.id && d.id.toLowerCase().includes(q)),
    );
  }, [datasets, searchQuery]);

  if (isCatalogLoading || !catalog) {
    return (
      <PageLayout>
        <PageHeader eyebrow="My catalog" title={<Skeleton className="h-7 w-64" />} />
        <Skeleton className="h-16 w-full rounded-lg" />
        <Skeleton className="h-48 w-full rounded-lg" />
      </PageLayout>
    );
  }

  const openDataset = (datasetId: string) =>
    navigate({
      to: "/catalog/$catalogId/dataset/$datasetId",
      params: { catalogId: catalog.id ?? "", datasetId },
    });

  const datasetColumns: Column<DatasetView>[] = [
    {
      header: "Dataset",
      accessorKey: "title",
      cell: (row) => (
        <div className="flex min-w-0 flex-col">
          <span className="truncate font-medium text-foreground">
            {row.title || "Untitled dataset"}
          </span>
          {row.description && (
            <span className="line-clamp-1 text-xs text-muted-foreground">{row.description}</span>
          )}
        </div>
      ),
    },
    { header: "Identifier", accessorKey: "id", cell: (row) => <Identifier value={row.id} /> },
    {
      header: "Distributions",
      className: "w-28 tabular-nums",
      cell: (row) => countOf(row, "distribution"),
    },
    {
      header: "Policies",
      className: "w-24 tabular-nums",
      cell: (row) => countOf(row, "hasPolicy"),
    },
    { header: "State", className: "w-28", cell: (row) => <PublicationBadge draft={row.draft} /> },
    {
      header: "Issued",
      accessorKey: "issued",
      className: "w-36 whitespace-nowrap",
      cell: (row) => <FormatDate date={row.issued} />,
    },
    {
      header: "",
      className: "w-20 text-right",
      cell: (row) => (
        <span
          className="inline-flex items-center justify-end gap-0.5"
          onClick={(e: React.MouseEvent) => e.stopPropagation()}
        >
          <Button
            variant="ghost"
            size="icon_sm"
            className="h-7 w-7"
            title="Edit dataset"
            onClick={() => setEditing({ kind: "dataset", id: row.id })}
          >
            <Pencil />
          </Button>
          <Button
            variant="ghost"
            size="icon_sm"
            className="h-7 w-7 text-danger-700 dark:text-danger-300"
            title="Delete dataset"
            onClick={() => setDeleting({ kind: "dataset", id: row.id, title: row.title })}
          >
            <Trash2 />
          </Button>
        </span>
      ),
    },
  ];

  const dataServiceColumns: Column<DataServiceView>[] = [
    {
      header: "Data service",
      accessorKey: "title",
      cell: (ds) => (
        <span className="inline-flex items-center gap-2">
          <span className="font-medium text-foreground">{ds.title || "Data service"}</span>
          {ds.id === mainDs?.id && (
            <Badge variant="info" size="sm">
              Primary
            </Badge>
          )}
        </span>
      ),
    },
    {
      header: "Endpoint",
      cell: (ds) => <span className="break-all font-mono text-xs">{ds.endpointURL || "—"}</span>,
    },
    { header: "Identifier", accessorKey: "id", cell: (ds) => <Identifier value={ds.id} /> },
    {
      header: "Issued",
      accessorKey: "issued",
      className: "w-36 whitespace-nowrap",
      cell: (ds) => <FormatDate date={ds.issued} />,
    },
    {
      header: "",
      className: "w-20 text-right",
      cell: (ds) => (
        <span
          className="inline-flex items-center justify-end gap-0.5"
          onClick={(e: React.MouseEvent) => e.stopPropagation()}
        >
          <Button
            variant="ghost"
            size="icon_sm"
            className="h-7 w-7"
            title="Edit data service"
            onClick={() => setEditing({ kind: "dataService", id: ds.id })}
          >
            <Pencil />
          </Button>
          {ds.id !== mainDs?.id && (
            <Button
              variant="ghost"
              size="icon_sm"
              className="h-7 w-7 text-danger-700 dark:text-danger-300"
              title="Delete data service"
              onClick={() => setDeleting({ kind: "dataService", id: ds.id, title: ds.title })}
            >
              <Trash2 />
            </Button>
          )}
        </span>
      ),
    },
  ];

  return (
    <PageLayout>
      <PageHeader
        eyebrow="My catalog"
        title={catalog.title || `${myAgent?.participant_nick || "Local agent"}'s catalog`}
        description={catalog.description}
        actions={
          <>
            <Link to="/catalog/$catalogId" params={{ catalogId: catalog.id! }}>
              <Button size="sm" variant="outline">
                <Eye /> View as consumer
              </Button>
            </Link>
            <Link to="/my-catalog/import">
              <Button size="sm" variant="outline">
                <Upload /> Import DCAT
              </Button>
            </Link>
            <Link to="/my-catalog/new">
              <Button size="sm">
                <Plus /> New dataset
              </Button>
            </Link>
          </>
        }
      />

      <FieldGrid
        columns={4}
        fields={[
          { label: "Publisher", value: myAgent?.participant_nick },
          { label: "Issued", value: <FormatDate date={catalog.issued} /> },
          { label: "Catalog", value: <Identifier value={catalog.id} /> },
          {
            label: "DSP endpoint",
            value: mainDs?.endpointURL && (
              <span className="break-all font-mono text-xs">{mainDs.endpointURL}</span>
            ),
          },
          {
            label: "Homepage",
            span: 4,
            hidden: !catalog.homepage,
            value: (
              <a
                href={catalog.homepage}
                target="_blank"
                rel="noreferrer"
                className="inline-flex items-center gap-1 text-brand-sky hover:underline"
              >
                {catalog.homepage}
                <ExternalLink className="h-3 w-3" />
              </a>
            ),
          },
        ]}
      />

      <Tabs value={activeTab} onValueChange={(tab) => setActiveTab(tab as CatalogTab)}>
        <TabsList>
          <TabsTrigger value="datasets">
            Datasets <CountBadge n={datasets.length} showZero />
          </TabsTrigger>
          <TabsTrigger value="dataservices">
            Data services <CountBadge n={dataservices.length} showZero />
          </TabsTrigger>
          <TabsTrigger value="specification">Specification</TabsTrigger>
        </TabsList>

        <TabsContent value="datasets" className="space-y-3 pt-3">
          {datasets.length === 0 ? (
            <p className="rounded-lg border border-dashed border-ink/15 py-8 text-center text-sm text-ink/60">
              This catalog has no datasets yet.
            </p>
          ) : (
            <>
              <div className="flex items-center justify-between gap-2">
                <Input
                  placeholder="Search datasets..."
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  onClear={() => setSearchQuery("")}
                  className="h-8 w-full text-xs sm:w-72"
                />
                <div className="flex shrink-0 items-center rounded-md border border-ink/10 p-0.5">
                  {VIEW_MODES.map(({ mode, icon: Icon, title }) => (
                    <button
                      key={mode}
                      type="button"
                      title={title}
                      aria-pressed={viewMode === mode}
                      onClick={() => setViewMode(mode)}
                      className={`rounded p-1.5 transition-colors ${
                        viewMode === mode
                          ? "bg-ink/10 text-ink"
                          : "text-muted-foreground hover:text-foreground"
                      }`}
                    >
                      <Icon className="h-3.5 w-3.5" />
                    </button>
                  ))}
                </div>
              </div>

              {filteredDatasets.length === 0 ? (
                <p className="rounded-lg border border-dashed border-ink/15 py-8 text-center text-sm text-ink/60">
                  No dataset matches "{searchQuery}".
                </p>
              ) : viewMode === "grid" ? (
                <div className="grid grid-cols-1 gap-3 md:grid-cols-2 2xl:grid-cols-3">
                  {filteredDatasets.map((dataset) => (
                    <DatasetItem
                      key={dataset.id}
                      title={dataset.title ?? ""}
                      description={dataset.description ?? ""}
                      date={dataset.issued ?? ""}
                      prevRoute={catalog.id ?? ""}
                      datasetId={dataset.id ?? ""}
                      ownDataset={true}
                      dataset={dataset}
                      onEdit={() => setEditing({ kind: "dataset", id: dataset.id })}
                      onDelete={() =>
                        setDeleting({ kind: "dataset", id: dataset.id, title: dataset.title })
                      }
                    />
                  ))}
                </div>
              ) : (
                <DataTable
                  columns={datasetColumns}
                  data={filteredDatasets}
                  keyExtractor={(d) => d.id}
                  onRowClick={(d) => openDataset(d.id)}
                  searchable={false}
                  pageSize={10}
                />
              )}
            </>
          )}
        </TabsContent>

        <TabsContent value="dataservices" className="pt-3">
          <DataTable
            columns={dataServiceColumns}
            data={dataservices}
            keyExtractor={(ds) => ds.id}
            onRowClick={(ds) =>
              navigate({
                to: "/catalog/$catalogId/data-service/$dataServiceId",
                params: { catalogId: catalog.id!, dataServiceId: ds.id },
              })
            }
            searchable={false}
            emptyMessage="No data service is registered, so consumers have no protocol endpoint."
            pageSize={10}
          />
        </TabsContent>

        <TabsContent value="specification" className="pt-3">
          <CatalogSpecificationPanel catalogId={catalog.id!} />
        </TabsContent>
      </Tabs>

      <CatalogResourceEditor
        target={editing}
        onClose={() => setEditing(null)}
        catalogId={catalog.id}
      />
      <DeleteResourceDialog target={deleting} onClose={() => setDeleting(null)} />
    </PageLayout>
  );
}
