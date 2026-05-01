import { createFileRoute } from "@tanstack/react-router";
import { ContractNegotiationActions } from "shared/src/components/actions/ContractNegotiationActions";
import { InfoList } from "shared/src/components/ui/info-list";
import { FormatDate } from "shared/src/components/ui/format-date";
import Heading from "../../../../../shared/src/components/ui/heading.tsx";
import {
  Drawer,
  DrawerBody,
  DrawerClose,
  DrawerContent,
  DrawerFooter,
  DrawerHeader,
  DrawerTitle,
  DrawerTrigger,
} from "@./../../shared/src/components/ui/drawer.tsx";
import { Button } from "shared/src/components/ui/button.tsx";
import CnProcessMessageComponent from "@./../../shared/src/components/CnProcessMessageComponent.tsx";
import { PageLayout } from "shared/src/components/layout/PageLayout";
import { PageSection } from "shared/src/components/layout/PageSection";
import { InfoGrid } from "shared/src/components/layout/InfoGrid";
import { useGetNegotiationProcessById } from "shared/data/orval/negotiations/negotiations.ts";
import { PageHeader } from "shared/components/layout/PageHeader.tsx";
import { Skeleton } from "shared/components/ui/skeleton.tsx";
import { GeneralErrorComponent } from "@/components/GeneralErrorComponent.tsx";
import { formatIdentifier } from "shared/lib/utils.ts";

const RouteComponent = () => {
  const { cnProcess } = Route.useParams();
  const { data: process, isLoading: isNegotiationProcessLoading } =
    useGetNegotiationProcessById(cnProcess);

  if (isNegotiationProcessLoading) {
    return (
      <PageLayout>
        <PageHeader
          title="Contract Negotiation Process"
          badge={<Skeleton className="h-8 w-48" />}
        />
        <div>Loading...</div>
      </PageLayout>
    );
  }

  // handle error
  if (!process || process.status !== 200) {
    return (
      <GeneralErrorComponent
        error={new Error("Contract negotiation process not found")}
        reset={() => {}}
      />
    );
  }

  return (
    <PageLayout>
      <InfoGrid className="mb-4">
        <PageSection title="Contract negotiation info">
          <InfoList
            items={[
              {
                label: "ProviderPid",
                value: {
                  type: "urn",
                  value: formatIdentifier(process.data.id),
                },
              },
              { label: "State", value: { type: "status", value: process.data.state } },
              {
                label: "Created at",
                value: { type: "custom", content: <FormatDate date={process.data.createdAt} /> },
              },
            ]}
          />
        </PageSection>
      </InfoGrid>
      <PageSection>
        {/* DRAWER */}
        <Drawer direction={"right"}>
          <DrawerTrigger>
            <Button variant={"secondary"}>See Contract Negotiation Messages</Button>
          </DrawerTrigger>
          <DrawerContent>
            <DrawerHeader>
              <DrawerTitle>
                <Heading level="h5" className="text-current">
                  Contract negotiation Messages
                </Heading>
              </DrawerTitle>
            </DrawerHeader>
            <DrawerBody>
              {/* New message subcomponent */}
              {process.data.messages?.map((message) => (
                <CnProcessMessageComponent key={message.id} message={message} />
              ))}
              {/* / New message subcomponent */}
            </DrawerBody>
            <DrawerFooter>
              <DrawerClose>
                <Button variant="ghost">Hide Messages</Button>
              </DrawerClose>
            </DrawerFooter>
          </DrawerContent>
        </Drawer>
      </PageSection>

      {/* ACTIONS */}
      <ContractNegotiationActions process={process.data} tiny={false} />
    </PageLayout>
  );
};

/**
 * Route for displaying contract negotiation process details.
 */
export const Route = createFileRoute("/contract-negotiation/$cnProcess/")({
  component: RouteComponent,
});
