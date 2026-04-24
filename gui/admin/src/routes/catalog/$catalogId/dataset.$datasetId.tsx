import { createFileRoute, Link } from "@tanstack/react-router";
import {
    useGetDatasetById,
    getGetDatasetByIdQueryOptions,
    useAddPolicyToDataset,
} from "shared/src/data/orval/datasets/datasets";
import {
    useGetDistributionsByDatasetId,
    getGetDistributionsByDatasetIdQueryOptions,
} from "shared/src/data/orval/distributions/distributions";
import { DataTable } from "shared/src/components/DataTable";
import { FormatDate } from "shared/src/components/ui/format-date";
import { ArrowRight, Plus } from "lucide-react";
import { useCreateOdrlPolicy, useGetPoliciesByEntityId } from "shared/src/data/orval/odrl-policies/odrl-policies";
import { OdrlPolicyInfo } from "shared/src/data/orval/model/odrlPolicyInfo";
import { Button } from "shared/src/components/ui/button.tsx";
import { formatUrn } from "shared/src/lib/utils";
import { PageLayout } from "shared/src/components/layout/PageLayout";
import { PageHeader } from "shared/src/components/layout/PageHeader";
import { InfoGrid } from "shared/src/components/layout/InfoGrid";
import { PageSection } from "shared/src/components/layout/PageSection";
import { InfoList } from "shared/src/components/ui/info-list";
import { Badge } from "shared/src/components/ui/badge";
import {
    Drawer,
    DrawerContent,
    DrawerHeader,
    DrawerTitle,
    DrawerTrigger,
} from "shared/src/components/ui/drawer";
import { PolicyWrapperNew } from "shared/src/components/PolicyWrapperNew.tsx";
import { PolicyWrapperShow } from "shared/src/components/PolicyWrapperShow.tsx";
import { useContext, useState } from "react";
import { GlobalInfoContext, GlobalInfoContextType } from "shared/src/context/GlobalInfoContext.tsx";
import Heading from "shared/src/components/ui/heading";
import DistributionItem from "shared/components/ui/distribution-item";


function RouteComponent() {
    const { catalogId, datasetId } = Route.useParams();
    const { data: datasetData } = useGetDatasetById(datasetId);
    const { data: distributionsData } = useGetDistributionsByDatasetId(datasetId);
    const { data: policiesData, refetch: refetchPolicies } = useGetPoliciesByEntityId(datasetId);
    const [open, setOpen] = useState(false);
    const { mutateAsync: createPolicyAsync } = useCreateOdrlPolicy();
    // const { api_gateway } = useContext<GlobalInfoContextType | null>(GlobalInfoContext)!; // No longer needed

    const dataset = datasetData?.status === 200 ? datasetData.data : undefined;
    const distributions = distributionsData?.status === 200 ? distributionsData.data : [];
    const policies = policiesData?.status === 200 ? policiesData.data : [];

    const onSubmit = async (data: OdrlPolicyInfo, description?: string) => {
        await createPolicyAsync({
            data: {
                entityId: datasetId,
                entityType: "Dataset",
                odrlOffer: data,
                description,
            }
        });
        setOpen(false);
        refetchPolicies();
    };

    if (!dataset) return null;

    return (
        <PageLayout>
            <div className="grid grid-cols-3 gap-12 max-h-[85vh]">
                <div className="rounded-md border border-background-200/60 bg-background-200/5 p-4 ">
                    <Heading level="h2">{dataset.dctTitle}</Heading>
                    <p className="text-sm mb-2"> {dataset.dctDescription
                        ? dataset.dctDescription : `Dataset with data about ${dataset.dctTitle}`} </p>
                    <InfoList
                        items={[

                            {
                                label: "Issued",
                                // @ts-ignore
                                value: { type: "custom", content: <FormatDate date={dataset?.dctIssued!} /> },
                            },
                        ]}
                    />
                    <div className="h-3"></div>
                    <div className="border-t border-white/10"></div>
                    <div className="h-4"></div>
                    <p className="text-pink-300"> Part of catalog slug</p>
                    <p className="text-pink-300"> Part of organization slug</p>
                </div>
                <div className="container-policies-distributions max-h-[85vh] overflow-y-scroll pr-4 col-span-2">
              
                        <Heading level="h4" className="text-left flex gap-3">
                            Policies

                            <div className=" mt-0.5">
                                <Drawer direction={"right"} open={open} onOpenChange={(open) => setOpen(open)}>
                                    <DrawerTrigger asChild>
                                        <Button
                                            variant="outline"
                                            size="sm"
                                            className="h-6 text-[10px] uppercase tracking-wide px-2 gap-1"
                                        >
                                            <Plus className="h-3 w-3" />
                                            Add Policy
                                        </Button>
                                    </DrawerTrigger>
                                    <DrawerContent>
                                        <DrawerHeader className="px-8 border-b border-white/10 pb-4 mb-4">
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
                                        <PolicyWrapperNew onSubmit={onSubmit} />
                                    </DrawerContent>
                                </Drawer>
                            </div>
                        </Heading>

                        <div className="grid grid-cols-2 gap-3">
                            {policies &&
                                policies.map((policy) => (
                                    <PolicyWrapperShow
                                        key={policy.id}
                                        policy={policy}
                                        datasetId={dataset.id!}
                                        catalogId={undefined}
                                        datasetName={dataset.dctTitle}
                                        showOfferAccess
                                    />
                                ))}
                        </div>
                        <div className="h-6"></div>
                        <Heading level="h4" className="text-left">
                            Distributions
                        </Heading>
                        <div className="grid grid-cols-2 gap-3">
                            {distributions && distributions.map((distribution) => (
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
                    </div>
                </div>

            {/* 
                 <PageHeader
                    title="Dataset with id"
                    badge={
                        <Badge variant="info" size="lg">
                            {formatUrn(dataset.id!)}
                        </Badge>
                    }
                />  */}

            {/* <InfoGrid>
                <PageSection>
                    <InfoList
                        items={[
                            { label: "Dataset title", value: dataset.dctTitle },
                            { label: "Dataset description", value: dataset.dctDescription },
                            {
                                label: "Catalog creation date",
                                value: { type: "custom", content: <FormatDate date={dataset.dctIssued!} /> },
                            },
                        ]}
                    />
                </PageSection>
            </InfoGrid> */}

            {/* <PageSection title="Distributions">
                <DataTable
                    className="text-sm"
                    data={distributions ?? []}
                    keyExtractor={(d) => d.id!}
                    columns={[
                        {
                            header: "Distribution Id",
                            cell: (d) => <Badge variant="info">{formatUrn(d.id!)}</Badge>,
                        },
                        {
                            header: "Distribution Title",
                            accessorKey: "dctTitle",
                            cell: (d) => d.dctTitle ?? "undefined",
                        },
                        {
                            header: "Created at",
                            cell: (d) => <FormatDate date={d.dctIssued!} />,
                        },
                        {
                            header: "Associated Connector",
                            cell: (d) => (
                                <div className="flex gap-2">
                                    <Link
                                        to="/catalog/$catalogId/distribution-connector/$distributionId"
                                        params={{
                                            catalogId: catalogId,
                                            distributionId: d.id!,
                                        }}
                                    >
                                        <Button variant="link" size="sm" className="h-auto p-0 text-xs">
                                            See connector instance
                                            <ArrowRight className="ml-1 h-3 w-3" />
                                        </Button>
                                    </Link>
                                </div>
                            ),
                        },
                        {
                            header: "Associated Data service",
                            cell: (d) => (
                                <div className="flex gap-2">
                                    <Link
                                        to="/catalog/$catalogId/data-service/$dataserviceId"
                                        params={{
                                            catalogId: catalogId,
                                            dataserviceId: d.dcatAccessService!,
                                        }}
                                    >
                                        <Button
                                            variant="link"
                                            size="sm"
                                            className="h-auto p-0 text-xs text-muted-foreground hover:text-foreground"
                                        >
                                            See dataservice
                                            <ArrowRight className="ml-1 h-3 w-3" />
                                        </Button>
                                    </Link>
                                </div>
                            ),
                        },
                    ]}
                />
            </PageSection> */}

            {/* <PageSection
                title="ODRL Policies"
                className="mt-10"
                action={
                    <Drawer direction={"right"} open={open} onOpenChange={(open) => setOpen(open)}>
                        <DrawerTrigger asChild>
                            <Button
                                variant="outline"
                                size="sm"
                                className="h-6 text-[10px] uppercase tracking-wide px-2 gap-1"
                            >
                                <Plus className="h-3 w-3" />
                                Add Policy
                            </Button>
                        </DrawerTrigger>
                        <DrawerContent>
                            <DrawerHeader className="px-8 border-b border-white/10 pb-4 mb-4">
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
                            <PolicyWrapperNew onSubmit={onSubmit} />
                        </DrawerContent>
                    </Drawer>
                }
            >
                <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                    {policies &&
                        policies.map((policy) => (
                            <PolicyWrapperShow
                                key={policy.id}
                                policy={policy}
                                datasetId={dataset.id!}
                                catalogId={undefined}
                                datasetName={dataset.dctTitle}
                                showOfferAccess
                            />
                        ))}
                </div>
            </PageSection> */}
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
