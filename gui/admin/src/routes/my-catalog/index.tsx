import { createFileRoute } from '@tanstack/react-router';
import Heading from 'shared/src/components/ui/heading';
import CatalogItem from "shared/src/components/ui/catalog-item";
import DatasetItem from "shared/src/components/ui/dataset-item";
import DistributionItem from "shared/src/components/ui/distribution-item"
import { Separator } from "shared/src/components/ui/separator";

export const Route = createFileRoute('/my-catalog/')({
    component: RouteComponent,
})

function RouteComponent() {
    return (
        <div>
          
            {/* <Separator orientation='vertical'></Separator> */}
            <div className="h-5"></div>
            <div className="grid grid-cols-3 gap-3">
                <CatalogItem></CatalogItem>
                <CatalogItem></CatalogItem>
                <CatalogItem></CatalogItem>
                <CatalogItem></CatalogItem>
            </div>
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
    )
}
