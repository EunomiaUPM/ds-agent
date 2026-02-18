import { createFileRoute, Link } from "@tanstack/react-router";
import { formatUrn } from "shared/src/lib/utils";
import dayjs from "dayjs";
import { Badge } from "shared/src/components/ui/badge";
import Heading from "shared/src/components/ui/heading";
import { PageLayout } from "shared/src/components/layout/PageLayout";
import { PageHeader } from "shared/src/components/layout/PageHeader";
import { PageSection } from "shared/src/components/layout/PageSection";
import { InfoGrid } from "shared/src/components/layout/InfoGrid";

import { DataTable } from "shared/src/components/DataTable";
import { FormatDate } from "shared/src/components/ui/format-date";

import { ArrowRight, Plus } from "lucide-react";

import {
  useGetCatalogById,
} from "shared/src/data/orval/catalogs/catalogs";
import {
  useGetDatasetsByCatalogId,
} from "shared/src/data/orval/datasets/datasets";
import {
  useGetDataServicesByCatalogId,
} from "shared/src/data/orval/data-services/data-services";
import { Button } from "shared/src/components/ui/button";
// Icons
import { InfoList } from "shared/src/components/ui/info-list";
import {
  Drawer,
  DrawerBody,
  DrawerClose,
  DrawerContent,
  DrawerFooter,
  DrawerHeader,
  DrawerTitle,
  DrawerTrigger,
} from "shared/src/components/ui/drawer";

const RouteComponent = () => {
  const { catalogId } = Route.useParams();
  const { data: catalogData } = useGetCatalogById(catalogId);
  const { data: datasetsData } = useGetDatasetsByCatalogId(catalogId);
  const { data: dataservicesData } = useGetDataServicesByCatalogId(catalogId);

  const catalog = catalogData?.status === 200 ? catalogData.data : undefined;
  const datasets = datasetsData?.status === 200 ? datasetsData.data : [];
  const dataservices = dataservicesData?.status === 200 ? dataservicesData.data : [];

  if (!catalog) return null;

  return (
    <PageLayout>
      <PageHeader
        title="Catalog info"
        badge={
          <Badge variant="info" size="lg">
            {formatUrn(catalogId)}
          </Badge>
        }
      />
      <InfoGrid>
        <PageSection title="Catalog details:">
          <InfoList
            items={[
              { label: "Catalog title", value: catalog.dctTitle },
              {
                label: "Catalog participant ID",
                value: { type: "urn", value: catalog.dspaceParticipantId },
              },
              { label: "Catalog homepage", value: catalog.foafHomePage },
              {
                label: "Catalog creation date",
                value: { type: "custom", content: <FormatDate date={catalog.dctIssued} /> },
              },
            ]}
          />
        </PageSection>
      </InfoGrid>

      <PageSection title="Datasets">
        <DataTable
          className="text-sm"
          data={datasets ?? []}
          keyExtractor={(d) => d.id!}
          columns={[
            {
              header: "Dataset ID",
              cell: (d) => <Badge variant="info">{formatUrn(d.id!)}</Badge>,
            },
            {
              header: "Title",
              accessorKey: "dctTitle",
            },
            {
              header: "Provider ID",
              cell: (d) => <Badge variant="info">{formatUrn(catalog.dspaceParticipantId!)}</Badge>,
            },
            {
              header: "Created at",
              cell: (d) => <FormatDate date={d.dctIssued!} />,
            },
            {
              header: "Actions",
              cell: (d) => (
                <Drawer direction={"right"}>
                  <DrawerTrigger>
                    <Button variant="outline" size="sm">
                      <Plus />
                      Offer dataset
                    </Button>
                  </DrawerTrigger>
                  <DrawerContent>
                    <DrawerHeader>
                      <DrawerTitle>
                        <Heading level="h5" className="text-current">
                          New Contract Negotiation Offer
                        </Heading>
                      </DrawerTitle>
                    </DrawerHeader>
                    <DrawerBody>
                      offer drawer...
                    </DrawerBody>
                    <DrawerFooter>
                      <DrawerClose className="flex justify-start gap-4">
                        <Button variant="ghost" className="w-40">
                          Cancel
                        </Button>
                      </DrawerClose>
                    </DrawerFooter>
                  </DrawerContent>
                </Drawer>
              ),
            },
            {
              header: "Link",
              cell: (d) => (
                <Link
                  to="/catalog/$catalogId/dataset/$datasetId"
                  params={{
                    catalogId: catalog.id!,
                    datasetId: d.id!,
                  }}
                >
                  <Button variant="link">
                    See dataset
                    <ArrowRight />
                  </Button>
                </Link>
              ),
            },
          ]}
        />
      </PageSection>
      <PageSection title="Dataservices">
        <DataTable
          className="text-sm"
          data={dataservices ?? []}
          keyExtractor={(ds) => ds.id!}
          columns={[
            {
              header: "Dataservice Id",
              cell: (ds) => <Badge variant="info">{formatUrn(ds.id!)}</Badge>,
            },
            {
              header: "Created at",
              cell: (ds) => <FormatDate date={ds.dctIssued!} />,
            },
            {
              header: "Link",
              cell: (ds) => (
                <Link
                  to="/catalog/$catalogId/data-service/$dataserviceId"
                  params={{
                    catalogId: catalog.id!,
                    dataserviceId: ds.id!,
                  }}
                >
                  <Button variant="link">
                    See dataservice
                    <ArrowRight />
                  </Button>
                </Link>
              ),
            },
          ]}
        />
      </PageSection>
    </PageLayout>
  );
};

/**
 * Route for displaying catalog details.
 */
export const Route = createFileRoute("/catalog/$catalogId/")({
  component: RouteComponent,
  pendingComponent: () => <div>Loading...</div>,
});
