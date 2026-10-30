import { createFileRoute, Link } from "@tanstack/react-router";
import { Plus } from "lucide-react";
import { PageLayout } from "shared/src/components/layout/PageLayout";
import { PageHeader } from "shared/src/components/layout/PageHeader";
import { PageSection } from "shared/src/components/layout/PageSection";
import { FieldGrid } from "shared/src/components/layout/FieldGrid";
import { FormatDate } from "shared/src/components/ui/format-date";
import { Button } from "shared/src/components/ui/button";
import { Badge } from "shared/src/components/ui/badge";
import { Identifier } from "shared/src/components/identity/Identifier";
import { ParticipantLabel } from "shared/src/components/identity/ParticipantLabel";
import DatasetItem from "shared/src/components/ui/dataset-item";
import { useGetCatalogById } from "shared/src/data/orval/catalogs/catalogs";
import { useGetDatasetsByCatalogId } from "shared/src/data/orval/datasets/datasets";
import { useGetDataServicesByCatalogId } from "shared/src/data/orval/data-services/data-services";
import { useMyself } from "shared/src/data/useMyself";
import { CatalogViews, type CatalogDoc } from "shared/src/data/catalog";

/** A catalog of this connector: its metadata, data services and datasets. */
const CatalogPage = () => {
  const { catalogId } = Route.useParams();
  const myAgent = useMyself();
  const { data: catalogData } = useGetCatalogById(catalogId);
  const { data: datasetsData } = useGetDatasetsByCatalogId(catalogId);
  const { data: dataservicesData } = useGetDataServicesByCatalogId(catalogId);

  const catalog =
    catalogData?.status === 200 ? CatalogViews.catalog(catalogData.data as CatalogDoc) : undefined;
  const datasets =
    datasetsData?.status === 200 ? CatalogViews.list(datasetsData.data, CatalogViews.dataset) : [];
  const dataservices =
    dataservicesData?.status === 200
      ? CatalogViews.list(dataservicesData.data, CatalogViews.dataService)
      : [];

  if (!catalog) return null;
  const own = !catalog.participantId || catalog.participantId === myAgent?.participant_id;

  return (
    <PageLayout>
      <PageHeader
        eyebrow="Catalog"
        title={catalog.title || `${myAgent?.participant_nick ?? "My"} catalog`}
        badge={own && <Badge variant="outline">Mine</Badge>}
        description={catalog.description || undefined}
        actions={
          own && (
            <Link to="/my-catalog/new">
              <Button>
                <Plus />
                New dataset
              </Button>
            </Link>
          )
        }
      />

      <PageSection title="Summary">
        <FieldGrid
          columns={4}
          fields={[
            { label: "Publisher", value: <ParticipantLabel id={catalog.participantId || myAgent?.participant_id} /> },
            { label: "Issued", value: <FormatDate date={catalog.issued || null} /> },
            {
              label: "Homepage",
              value: catalog.homepage ? (
                <a href={catalog.homepage} className="text-brand-sky hover:underline break-all">
                  {catalog.homepage}
                </a>
              ) : null,
            },
            { label: "Identifier", value: <Identifier value={catalog.id} tail={36} /> },
          ]}
        />
      </PageSection>

      <PageSection title="Data services">
        <FieldGrid
          columns={2}
          fields={dataservices.flatMap((ds) => [
            { label: "Data service", value: <Identifier value={ds.id} tail={36} /> },
            {
              label: "Endpoint",
              value: ds.endpointURL ? <span className="font-mono text-xs">{ds.endpointURL}</span> : null,
            },
          ])}
        />
      </PageSection>

      <PageSection title={`Datasets · ${datasets.length}`}>
        {datasets.length > 0 ? (
          <div className="grid grid-cols-1 gap-3 md:grid-cols-2 xl:grid-cols-3">
            {datasets.map((dataset) => (
              <DatasetItem
                key={dataset.id}
                title={dataset.title}
                description={dataset.description}
                date={dataset.issued}
                prevRoute={catalog.id}
                datasetId={dataset.id}
                ownDataset={own}
                dataset={dataset}
              />
            ))}
          </div>
        ) : (
          <p className="text-sm text-ink/50">This catalog has no datasets yet.</p>
        )}
      </PageSection>
    </PageLayout>
  );
};

export const Route = createFileRoute("/catalog/$catalogId/")({
  component: CatalogPage,
});
