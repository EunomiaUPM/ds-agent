import { createFileRoute, Link } from "@tanstack/react-router";
import { ArrowRight } from "lucide-react";
import CatalogItem from "shared/src/components/ui/catalog-item";
import { PageHeader } from "@eunomia/ui";
import { Heading } from "@eunomia/ui";
import { Button } from "@eunomia/ui";
import { PageLayout } from "@eunomia/ui";
import { useFederatedCatalog } from "shared/src/data/useFederatedCatalog";
import { usePeerConnections } from "shared/src/data/peerConnection";
import { WizardDialog } from "@eunomia/ui";
import { useRef, useState } from "react";
import { Card, CardContent } from "@eunomia/ui";
import { eunomiaLogoLight as logoImg } from "@eunomia/ui";
import { eunomiaLogoDark as logoImgDark } from "@eunomia/ui";
import { useTheme } from "@eunomia/ui";
import { useMyself } from "shared/src/data/useMyself";

const RouteComponent = () => {
  const { resolvedTheme } = useTheme();
  const federated = useFederatedCatalog();
  const connections = usePeerConnections();

  const myAgent = useMyself();

  const labelCatalogRef = useRef<HTMLElement | null>(null);
  // first wizard URL state
  const [wizardCatalogOpen, setWizardCatalogOpen] = useState(true);

  if (federated.state === "loading" || connections.isLoading) {
    return (
      <PageLayout>
        <div>Loading...</div>
      </PageLayout>
    );
  }

  if (federated.state === "no-authority") {
    return (
      <PageLayout className="overlayContainer">
        <Card className="overlayContent h-fit">
          <div className="contentLogo">
            <img src={resolvedTheme === "dark" ? logoImg : logoImgDark} alt="Eunomia Logo" />
          </div>
          <CardContent className="contentMessage">
            <div className="messageGroup">
              <Heading level="h2">You haven't joined a Dataspace yet</Heading>
              <p className="text-sm text-muted-foreground">
                Add an authority to discover other participants and browse their catalogs.
              </p>
            </div>
          </CardContent>
          <Link to="/authority/new" className="contentFooter">
            <Button>
              Join a Dataspace
              <ArrowRight />
            </Button>
          </Link>
        </Card>
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

  const isOwn = (participantId: string) => participantId === myAgent?.participant_id;
  // Whether this user is connected to any peer yet; if not, the tutorial guides the first one.
  const onboardedWithKnownProvider = agents.some(
    (prov) => connections.statusOf(prov.participant_id) === "approved",
  );
  const firstPeer = agents.find((p) => !isOwn(p.participant_id));

  return (
    <PageLayout>
      <PageHeader
        title="Catalog Browser"
        description="Public catalogs of the dataspace and the catalogs of the participants you are connected to."
      />
      {!onboardedWithKnownProvider && (
        <WizardDialog
          open={wizardCatalogOpen}
          onClose={() => setWizardCatalogOpen(false)}
          anchorRef={labelCatalogRef}
          align="left"
          step="1 of 2"
          sectionTitle="Connection with Participant Tutorial"
          title="Catalog browser"
          content={
            <>
              At this section you can browse the catalogs of the dataspace participants.
              <br />
            </>
          }
        />
      )}
      {/* The tutorial bubble sits above its anchor: leave it room instead of covering the header. */}
      {!onboardedWithKnownProvider && wizardCatalogOpen && <div className="h-36" />}
      <div ref={(el) => (labelCatalogRef.current = el as any)} className="h-4" />
      <div className="grid grid-cols-3 gap-5">
        {agents.map((p) => {
          const connection = connections.statusOf(p.participant_id);
          // With no connection yet, the first peer is highlighted (tutorial).
          const firstAgentWithUnauth =
            !onboardedWithKnownProvider && p === firstPeer && connection === "none";

          return (
            <div
              key={p.participant_id}
              // Same width and corners as the card, so the highlight fits it.
              className={`max-w-lg rounded-xl ${
                firstAgentWithUnauth ? "ring-2 ring-secondary-400 shadow-md animate-pulse" : ""
              }`}
              onClick={() => setWizardCatalogOpen(false)}
            >
              <CatalogItem
                organizationName={p.participant_nick ?? "Unknown"}
                id={p.participant_id ?? null}
                connection={connection}
                participant={p}
                showTutorial={!onboardedWithKnownProvider}
                onConnectClose={() => {
                  if (firstAgentWithUnauth) {
                    setTimeout(() => setWizardCatalogOpen(true), 50);
                  }
                }}
                ownCatalog={isOwn(p.participant_id)}
              />
            </div>
          );
        })}
        <CatalogItem
          date={"5/19/2022"}
          datasetNumber={17}
          organizationName={"Another participant"}
          id={null}
          title={"Meteorology Stations in Madrid Catalog"}
        />
        <CatalogItem
          date={"12/08/2025"}
          datasetNumber={23}
          organizationName={"Another participant"}
          id={null}
          title={"Parking Ocupation in Ávila Catalog"}
        />
        <CatalogItem
          date={"2/2/2024"}
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
