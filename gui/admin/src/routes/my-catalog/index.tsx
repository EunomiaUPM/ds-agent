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

import { useGetCatalogs, useGetMainCatalogs } from "shared/data/orval/catalogs/catalogs";
import { formatUrn } from "shared/src/lib/utils";
import { DataTable } from "shared/src/components/DataTable";
import { FormatDate } from "shared/src/components/ui/format-date";
import { Badge } from "shared/src/components/ui/badge";
import { Link } from "@tanstack/react-router";
import { Button } from "shared/src/components/ui/button.tsx";
import { ArrowRight } from "lucide-react";

export const Route = createFileRoute("/my-catalog/")({
  component: RouteComponent,
});

function RouteComponent() {
      const { data: mainCatalog } = useGetMainCatalogs();
  const { data: catalogs } = useGetCatalogs();

//   const { mutate, data, isPending, error } = useRpcSetupCatalogRequest();
//     useEffect(() => {
//       mutate({
//         data: {
//           associatedAgentPeer: participantId,
//           filter: [],
//             noCache: true
//         },
//       });
//     }, [participantId, mutate]);
//   const catalog = data?.status === 200 ? data.data : undefined;

//     {console.log(catalog, "others catalogs")}
//   if (!catalog) return null;



// if (isPending) {
//     return (
//       <PageLayout>
       
//         <div>Loading...</div>
//       </PageLayout>
//     );
//   }
//   if (error) {
//     return (
//       <div className="flex items-center justify-center h-full text-red-500">
//         Error loading catalog: {error.message}
//       </div>
//     );
//   }
  
  if (!mainCatalog?.data || mainCatalog.status !== 200) return null;
    return (
        <div>
          <Heading level="h2" className="mb-4">My Catalog</Heading>
            {/* <Separator orientation='vertical'></Separator> */}
            <PageSection title="Catalogs">
        <DataTable
          className="text-sm"
          data={Array.isArray(catalogs?.data) ? catalogs.data : []}
          keyExtractor={(c) => c.id!}
          columns={[
            {
              header: "Title",
              accessorKey: "dctTitle",
              cell: (c) => <p className="text-18">{c.dctTitle}</p>,
            },
            {
              header: "Created at",
              cell: (c) => <FormatDate date={c.dctIssued} />,
            },
            {
              header: "Catalog ID",
              cell: (c) => <Badge variant="info">{formatUrn(c.id)}</Badge>,
            },
            {
              header: "Provider ID",
              cell: (c) => <Badge variant="info">{formatUrn(c.dspaceParticipantId)}</Badge>,
            },
            {
              header: "Link",
              cell: (c) => (
                <Link to="/catalog/$catalogId" params={{ catalogId: c.id }}>
                  <Button variant={"link"}>
                    See catalog
                    <ArrowRight />
                  </Button>
                </Link>
              ),
            },
          ]}
        />
      </PageSection>
            <div className="h-5"></div>
            <div className="grid grid-cols-3 gap-3">
                {/* <CatalogItem ></CatalogItem>
                <CatalogItem></CatalogItem>
                <CatalogItem></CatalogItem>
                <CatalogItem></CatalogItem> */}
            </div>
            <div className="wrapper opacity-15">
            <div className="h-5"></div>
            <div className="card-organization-container flex-col bg-brand-sky/15 border rounded-md border-white/20 flex flex-col px-3 pt-2 pb-3 max-w-[250px]">
                <p className="text-xs uppercase">Organization</p>
                <div className="h-2"></div>
                <div className="card-organization-info flex gap-3">
                    <div>
                        <img className="rounded-full bg-violet-600 h-12 aspect-square"></img>
                    </div>
                    <div className="card-organization-text">
                        <Heading level="h4" className="mb-1"> ECOSTARTS</Heading>
                        <p className='text-sm'>ESG Certification Services</p>
                    </div>
                </div>
            </div>
            <div className="divider">
                <div className="h-6"></div>
                <div className="border-t border-white"></div>
                <div className="h-6"></div>
            </div>
            <div className="flex flex-wrap gap-3">
                <DatasetItem />
                <DatasetItem />
                <DatasetItem />
                <DatasetItem />
                <DatasetItem />
                <DatasetItem />
                <DatasetItem />
                <DatasetItem />
            </div>
             <div className="divider">
                <div className="h-6"></div>
                <div className="border-t border-white"></div>
                <div className="h-6"></div>
            </div>
            <div className="flex flex-wrap gap-3">
                <DistributionItem/>
                       <DistributionItem/>
                              <DistributionItem/>
            </div>
        </div>
        </div>
    )
}
