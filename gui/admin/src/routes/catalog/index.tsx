import { createFileRoute, Link } from "@tanstack/react-router";
import { ArrowRight } from "lucide-react";
import CatalogItem from "shared/src/components/ui/catalog-item";
import Heading from "shared/src/components/ui/heading.tsx";
import { Button } from "shared/src/components/ui/button.tsx";
import { PageLayout } from "shared/src/components/layout/PageLayout";
import { useFederatedCatalog } from "shared/src/data/useFederatedCatalog";
import { useGetAllParticipants } from "shared/src/data/orval/participants/participants";
import WizardDialog from "shared/src/components/WizardDialog";
import { useRef, useState } from "react";

const RouteComponent = () => {
  const federated = useFederatedCatalog();
  const { data: participantsResponse } = useGetAllParticipants();
  const localParticipants =
    participantsResponse?.status === 200 ? participantsResponse.data : [];

  const labelCatalogRef = useRef<HTMLElement | null>(null);
  // first wizard URL state
  const [wizardCatalogOpen, setWizardCatalogOpen] = useState(true);



  if (federated.state === "loading") {
    return (
      <PageLayout>
        <div>Loading...</div>
      </PageLayout>
    );
  }

  if (federated.state === "no-authority") {
    return (
      <PageLayout>
        <div className="bg-violet-700/40 flex flex-col justify-center items-center h-48 gap-2">
          <Heading level="h2">You haven't joined a Dataspace yet</Heading>
          <p className="text-sm text-muted-foreground">
            Add an authority to discover other participants and browse their catalogs.
          </p>
          <Link to="/authority/new">
            <Button>
              Join a Dataspace
              <ArrowRight />
            </Button>
          </Link>
        </div>
      </PageLayout>
    );
  }

  if (federated.state === "error") {
    return (
      <PageLayout>
        <div className="flex items-center justify-center h-full text-red-500">
          Error loading federated catalog.
        </div>
      </PageLayout>
    );
  }

  const { agents } = federated;
  console.log(localParticipants, "localparticipants")
  console.log(agents, "agents")

  //variable that tells if user is onboarded with any provider or not.
  // true = they are / false = they're not
  const onboardedWithKnownProvider = agents.some((prov) =>
  localParticipants.some(
    (lp) => lp.participant_id === prov.participant_id && !lp.is_me && lp.participant_type !== "Authority"
  )
);

  return (
    <PageLayout>
      <div className="bg-violet-700/40 flex justify-center items-center h-48">
        <Heading level="h2">Browse public catalogs and your connections' catalogs </Heading>
      </div>
   {!onboardedWithKnownProvider &&
      <WizardDialog
        open={wizardCatalogOpen}
        onClose={() => setWizardCatalogOpen(false)}
        anchorRef={labelCatalogRef}
        align="left"
        step={"1 of 3"}
        sectionTitle="Connection with Participant Tutorial"
        title="Catalog browser"
        content={
          <>
            At this section you can browse the catalogs of the dataspace participants.
            <br />

          </>}
      />
        }
        <div ref={(el) => (labelCatalogRef.current = el as any) }className="h-4" />
        <div className="grid grid-cols-3 gap-5">

          {agents.map((p) => {
            const isOnboarded = localParticipants.some(
              (lp) => lp.participant_id === p.participant_id && !lp.is_me,
            );
            const unauthRedirect = isOnboarded
              ? null
              : { url: p.base_url, slug: p.participant_slug };
            return (
              <div className={unauthRedirect 
              ? "ring-2 ring-secondary-400 shadow-md animate-pulse rounded-md w-full max-w-lg" 
              : ""}
              onClick={() => setWizardCatalogOpen(false)}>
                <CatalogItem
                  key={p.participant_id}
                  date={""}
                  datasetNumber={0}
                  organizationName={p.participant_slug ?? "Unknown"}
                  id={p.participant_id ?? null}
                  isAuthenticated={isOnboarded}
                  unauthRedirect={unauthRedirect}
                />
              </div>
            );
          })}
          <CatalogItem
            date={""}
            datasetNumber={17}
            organizationName={"Another participant"}
            id={null}
            title={"Meteorology Stations in Madrid Catalog"}
          />
          <CatalogItem
            date={""}
            datasetNumber={23}
            organizationName={"Another participant"}
            id={null}
            title={"Parking Ocupation in Ávila Catalog"}
          />
          <CatalogItem
            date={""}
            datasetNumber={31}
            organizationName={"Another participant"}
            id={null}
            title={"Population Growth in Spain 2026 Catalog"}
          />
        </div>
        <div className="h-4" />
    </PageLayout>
  );
};

export const Route = createFileRoute("/catalog/")({
  component: RouteComponent,
  pendingComponent: () => <div>Loading...</div>,
});
