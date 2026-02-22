import { createFileRoute } from "@tanstack/react-router";
import { useContext, useState } from "react";
import { GlobalInfoContext, GlobalInfoContextType } from "shared/src/context/GlobalInfoContext";
import {
  useBusinessGetPoliciesByDatasetId,
  useGetBusinessDatahubDataset,
  useGetBusinessPolicyTemplates,
} from "shared/src/data/business-queries";
import { AuthContext, AuthContextType } from "shared/src/context/AuthContext";
import { Button } from "shared/src/components/ui/button";
import { usePostBusinessNewPolicyInDataset } from "shared/src/data/business-mutations";
import { PolicyWrapperShow } from "shared/src/components/PolicyWrapperShow";
import { PolicyTemplateWrapperEdit } from "shared/src/components/PolicyTemplateWrapperEdit";
import Heading from "shared/src/components/ui/heading";
import { Plus } from "lucide-react";
import {
  Drawer,
  DrawerContent,
  DrawerDescription,
  DrawerHeader,
  DrawerTitle,
  DrawerTrigger,
} from "shared/src/components/ui/drawer";
import { InfoList } from "shared/src/components/ui/info-list";
import { Badge } from "shared/src/components/ui/badge";

function RouteComponent() {
  const { catalogId, datasetId } = Route.useParams();
  const { participant } = useContext<AuthContextType | null>(AuthContext)!;
  const [isOpen, setIsOpen] = useState<boolean>(false);
  const { data: dataset } = useGetBusinessDatahubDataset(datasetId);
  const { data: policies } = useBusinessGetPoliciesByDatasetId(catalogId, datasetId);
  const { data: policy_templates } = useGetBusinessPolicyTemplates() as {
    data: PolicyTemplate[];
  };
  const { mutateAsync: createPolicyAsync } = usePostBusinessNewPolicyInDataset();
  const { api_gateway } = useContext<GlobalInfoContextType | null>(GlobalInfoContext)!;

  const onSubmit = async (odrlContent: OdrlInfo, description?: string) => {
    await createPolicyAsync({
      api_gateway,
      datasetId,
      catalogId,
      content: {
        offer: odrlContent,
        description,
      },
    });
    setIsOpen(false);
  };

  return (
    <div className="space-y-4">
      <Heading level="h3" className="flex gap-2 items-center">
        Dataset
        <Badge variant="info" size="lg">
          {dataset.name}
        </Badge>
      </Heading>
      <InfoList
        className="text-sm w-2/3"
        items={dataset.custom_properties.map((property) => ({
          label: property[0],
          value: property[1],
          keyClassName: "text-sky-300",
          className: "text-gray-300/90",
        }))}
      />

      <div className=" flex flex-row  justify-start gap-3 items-center">
        <Heading level="h5" className="mb-0">
          ODRL Policies
        </Heading>

        <Drawer direction={"right"} open={isOpen} onOpenChange={(open) => setIsOpen(open)}>
          <DrawerTrigger asChild={true}>
            {participant?.participant_type == "Provider" && policy_templates && (
              <Button variant="default" size="sm" className="mb-1 ml-3">
                Add ODRL policy
                <Plus className="" />
              </Button>
            )}
          </DrawerTrigger>
          <DrawerContent>
            <DrawerHeader className="px-8">
              <DrawerTitle>
                <Heading level="h4" className="text-current mb-1 ">
                  New ODRL Policy
                </Heading>
                <p className="font-normal text-brand-sky">
                  {" "}
                  for Dataset
                  <Badge variant="info" size="sm" className="ml-2">
                    {" "}
                    {dataset.name}
                  </Badge>
                </p>
              </DrawerTitle>
              <DrawerDescription>
                Start by selecting a policy template, then edit the corresponding values.
              </DrawerDescription>
            </DrawerHeader>
            <div className="px-8 flex flex-col gap-4 overflow-y-scroll h-[80vh] pb-6">
              {policy_templates.map((policy_template) => (
                <PolicyTemplateWrapperEdit
                  key={policy_template.id}
                  policyTemplate={policy_template}
                  onSubmit={onSubmit}
                />
              ))}
            </div>
          </DrawerContent>
        </Drawer>
      </div>

      <div className="grid grid-cols-2 gap-4">
        {policies &&
          policies.map((policy) => (
            <PolicyWrapperShow
              key={policy["@id"]}
              participant={participant}
              policy={policy}
              catalogId={catalogId}
              datasetId={datasetId}
              datasetName={dataset.name}
            />
          ))}
      </div>
    </div>
  );
}

export const Route = createFileRoute("/datahub-catalog/$catalogId/dataset/$datasetId")({
  component: RouteComponent,
  pendingComponent: () => <div>Loading...</div>,
});
