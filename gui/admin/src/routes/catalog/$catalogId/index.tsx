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
import { useGetAllParticipants } from "shared/data/orval/participants/participants";
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
import DatasetItem from "shared/components/ui/dataset-item";



const RouteComponent = () => {
    const { catalogId } = Route.useParams();
    const { data: catalogData } = useGetCatalogById(catalogId);
    const { data: datasetsData } = useGetDatasetsByCatalogId(catalogId);
    const { data: dataservicesData } = useGetDataServicesByCatalogId(catalogId);

    
    const catalog = catalogData?.status === 200 ? catalogData.data : undefined;
    const datasets = datasetsData?.status === 200 ? datasetsData.data : [];
    const dataservices = dataservicesData?.status === 200 ? dataservicesData.data : [];

    if (!catalog) return null;
    const { data: participants } = useGetAllParticipants();

    console.log(datasets, "datasets")

    const myAgent = Array.isArray(participants?.data)
        ? participants.data.find(
            (p) => p.is_me && p.participant_type === "Agent"
        )
        : undefined;

    const myAgentSlug =
        myAgent?.participant_slug?.toString() || "Unknown Participant";

        {console.log(datasets.map((d: any) => d.id), "datasets ids in catalog")}

    return (
        <PageLayout>
            {/* <PageHeader
                title="My Catalog"
                badge={
                    <Badge variant="info" size="lg">
                        {formatUrn(catalogId)}
                    </Badge>
                }
            /> */}
            <div className="grid grid-cols-3 gap-12">

                <div className="rounded-md border border-background-200/60 bg-background-200/5 p-4 max-h-[60vh] ">
                    <Heading level="h2"> My catalog</Heading>
                    <p className="text-sm mb-2">Description of the catalog. This is the catalog of {myAgentSlug}</p>
                    <InfoList
                        items={[
                            { label: "Catalog title", value: catalog.dctTitle },

                            { label: "Catalog homepage", value: catalog.foafHomePage },
                            {
                                label: "Catalog creation date",
                                value: { type: "custom", content: <FormatDate date={catalog.dctIssued} /> },
                            },

                        ]}
                    />
                    <div className="catalog-participant-container flex gap-2 justify-start">
                        <img className="rounded-full bg-violet-600 h-6 aspect-square"></img>
                        <Heading level="h4" className='capitalize'> {myAgentSlug} </Heading>
                    </div>
                    <div className="h-3"></div>
                    <div className="border-t border-white/10"></div>
                    <div className="h-4"></div>
                    {dataservices.map((ds) => (
                        <>
                            <Heading level="h4" className="text-left">
                                Dataservice
                            </Heading>
                            <InfoList
                                items={[
                                    {
                                        label: "Dataservice ID",
                                        value: { type: "urn", value: ds.id! },
                                    },

                                    {
                                        label: "Dataservice creation date",
                                        value: { type: "custom", content: <FormatDate date={ds.dctIssued} /> },
                                    },
                                    {
                                        label: "Endpoint",
                                        value: "Dataservice URL",
                                    },
                                ]}
                            />
                        </>
                    ))}
                </div>
                <div className="col-span-2">
                    <Heading level="h4" className="text-left">
                        Datasets
                    </Heading>
                    <div className="grid grid-cols-2 gap-3">
                        {(datasets.length > 0) ? datasets.map((dataset) => (
                            <DatasetItem
                                key={dataset.id}
                                title={dataset.dctTitle!}
                                description={dataset.dctDescription!}
                                date={dataset.dctIssued!}
                                prevRoute={catalog.id!}
                                datasetId={dataset.id!}
                                ownDataset={true}
                            />
                        )) : <p className="text-muted-foreground italic text-sm">No datasets to show</p>}
                    </div>
                </div>

            </div>

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
                            header: "Description",
                            accessorKey: "dctDescription",
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
