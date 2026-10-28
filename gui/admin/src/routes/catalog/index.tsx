import { createFileRoute, Link } from "@tanstack/react-router";
import { ArrowRight } from "lucide-react";
import CatalogItem from "shared/src/components/ui/catalog-item";
import Heading from "shared/src/components/ui/heading.tsx";
import { Button } from "shared/src/components/ui/button.tsx";
import { PageLayout } from "shared/src/components/layout/PageLayout";
import { useFederatedCatalog } from "shared/src/data/useFederatedCatalog";
import { usePeerConnections } from "shared/src/data/peerConnection";
import { Card, CardContent } from "shared/components/ui/card";
import logoImg from "./../../../../shared/src/img/eunomia_logo_lg_light.svg";
import logoImgDark from "./../../../../shared/src/img/eunomia_logo_lg_dark.svg";
import { useTheme } from "shared/src/hooks/useTheme";
import { useGetMyself } from "shared/src/data/orval/participants/participants";

const RouteComponent = () => {
  const { resolvedTheme } = useTheme();
  const federated = useFederatedCatalog();
  const connections = usePeerConnections();

  // Until this connector is known, its own card would pass for a peer.
  const myself = useGetMyself();
  const myAgent = myself.data?.status === 200 ? myself.data.data : undefined;

  if (federated.state === "loading" || connections.isLoading || myself.isLoading) {
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
  // Whether this user is connected to any peer yet; if not, the first one to connect with is
  // highlighted.
  const onboardedWithKnownProvider = agents.some(
    (prov) => connections.statusOf(prov.participant_id) === "approved",
  );
  const firstPeer = onboardedWithKnownProvider
    ? undefined
    : agents.find(
        (p) => !isOwn(p.participant_id) && connections.statusOf(p.participant_id) === "none",
      );

  return (
    <PageLayout>
      <div className="bg-violet-700/40 flex justify-center items-center h-48">
        <Heading level="h2">Browse public catalogs and your connections' catalogs </Heading>
      </div>
      <p className="h-4 my-2 text-sm text-muted-foreground">
        {firstPeer && "Pick a participant to connect with and browse its catalog."}
      </p>
      <div className="grid grid-cols-3 gap-5">
        {agents.map((p) => {
          const highlighted = p === firstPeer;

          return (
            <div
              key={p.participant_id}
              // Same width and corners as the card, so the highlight fits it.
              className={`max-w-lg rounded-xl ${
                highlighted ? "ring-2 ring-secondary-400 shadow-md animate-pulse" : ""
              }`}
            >
              <CatalogItem
                organizationName={p.participant_nick ?? "Unknown"}
                id={p.participant_id ?? null}
                connection={connections.statusOf(p.participant_id)}
                participant={p}
                pendingGrantId={connections.pendingGrantOf(p.participant_id)?.id}
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
