import { createFileRoute } from "@tanstack/react-router";
import Heading from "shared/src/components/ui/heading";
import CatalogItem from "shared/src/components/ui/catalog-item";
import DatasetItem from "shared/src/components/ui/dataset-item";
import DistributionItem from "shared/src/components/ui/distribution-item";
import { Separator } from "shared/src/components/ui/separator";
import { PageLayout } from "shared/src/components/layout/PageLayout";
import { PageHeader } from "shared/src/components/layout/PageHeader";
import { PageSection } from "shared/src/components/layout/PageSection";
import { Skeleton } from "shared/src/components/ui/skeleton";
import { InfoList } from "shared/src/components/ui/info-list";
import AvatarImg from "shared/src/components/ui/avatar-img";

import { useGetCatalogById } from "shared/src/data/orval/catalogs/catalogs";
import { useGetDatasetsByCatalogId } from "shared/src/data/orval/datasets/datasets";
import { useGetDataServicesByCatalogId } from "shared/src/data/orval/data-services/data-services";
import { useGetMainCatalogs } from "shared/src/data/orval/catalogs/catalogs";
import { useGetAllParticipants } from "shared/data/orval/participants/participants";
import { useEffect } from "react";
import { formatUrn } from "shared/src/lib/utils";
import { DataTable } from "shared/src/components/DataTable";
import { FormatDate } from "shared/src/components/ui/format-date";
import { Badge } from "shared/src/components/ui/badge";
import { Link } from "@tanstack/react-router";
import { Button } from "shared/src/components/ui/button";
import { ArrowRight } from "lucide-react";

export const Route = createFileRoute("/my-catalog/")({
  component: RouteComponent,
});

function RouteComponent() {
  const { data: mainCatalogData } = useGetMainCatalogs();
  const mainCatalog = mainCatalogData?.status === 200 ? mainCatalogData.data : undefined;

  const catalogId = mainCatalog?.id;

  const { data: catalogData } = useGetCatalogById(catalogId ?? "");
  const { data: datasetsData } = useGetDatasetsByCatalogId(catalogId ?? "");
  const { data: dataservicesData } = useGetDataServicesByCatalogId(catalogId ?? "");

  const catalog = catalogData?.status === 200 ? catalogData.data : undefined;
  const datasets = datasetsData?.status === 200 ? datasetsData.data : [];
  const dataservices = dataservicesData?.status === 200 ? dataservicesData.data : [];

  // call participants hook unconditionally so hooks count is stable across renders
  const { data: participants } = useGetAllParticipants();

  // if we don't have the resolved catalog yet, show nothing (prevents crashes)
  if (!catalog) return null;

  const myAgent = Array.isArray(participants?.data)
    ? participants.data.find((p) => p.is_me && p.participant_type === "Agent")
    : undefined;

  const myAgentSlug = myAgent?.participant_slug;

  // pick preferred data service: marked main or first available
  const mainDs = (dataservices || []).find((ds) => ds.dspaceMainDataService) ?? dataservices[0];
  const hasDataservice = !!mainDs;

  return (
    <PageLayout>

      <div className="grid grid-cols-3 gap-12">
        <div className="rounded-md border border-background-200/60 bg-background-200/5 p-4 max-h-[70vh] ">
          <Heading level="h2" className="capitalize">
            {" "}
            {catalog.dctTitle ? catalog.dctTitle : `${myAgentSlug}'s Catalog`}
          </Heading>

          <Badge variant="detail" size="lg" className="uppercase text-blue-300 font-semibold mb-3">
            My own catalog
          </Badge>
          <p className="text-sm mb-2">
            Description of the catalog. This is the catalog of{" "}
            <span className="capitalize">{myAgentSlug}</span>
          </p>
          <InfoList
            items={[
              { label: "Catalog title", value: catalog.dctTitle },

              { label: "Catalog homepage", value: catalog.foafHomePage },
              {
                label: "Catalog creation date",
                value: { type: "custom", content: <FormatDate date={catalog.dctIssued} /> },
              },
              {
                label: "Organization",
                value: {
                  type: "custom",
                  content: (
                    <div className={`catalog-participant-container flex gap-2 justify-start `}>
                      <AvatarImg sizeClass="h-7" />
                      <Heading level="h4" className="capitalize">
                        {" "}
                        {myAgentSlug}{" "}
                      </Heading>
                    </div>
                  ),
                },
              },
            ]}
          />

          <div className="h-1"></div>
          <div className="border-t border-white/10"></div>
          <div className="h-2"></div>

          <>
            <Heading level="h4" className="text-left">
              Dataservice
            </Heading>
            <InfoList
              items={
                hasDataservice
                  ? [
                    {
                      label: "Dataservice ID",
                      value: { type: "urn", value: mainDs.id! },
                    },
                    {
                      label: "Dataservice creation date",
                      value: {
                        type: "custom",
                        content: <FormatDate date={mainDs.dctIssued} />,
                      },
                    },
                    {
                      label: "Endpoint",
                      value: mainDs.dcatEndpointUrl ?? "No endpoint provided",
                    },
                  ]
                  : [
                    { label: "Dataservice", value: "No dataservice registered for this catalog" },
                  ]
              }
            />
          </>
        </div>
        <div className="col-span-2">
          <Heading level="h4" className="text-left">
            Datasets
          </Heading>
          <div className="grid grid-cols-2 gap-3">
            {datasets.length > 0 ? (
              datasets.map((dataset) => (
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
              ))
            ) : (
              <p className="text-muted-foreground italic text-sm">No datasets to show</p>
            )}
          </div>
        </div>
      </div>

    </PageLayout>

  );
}
