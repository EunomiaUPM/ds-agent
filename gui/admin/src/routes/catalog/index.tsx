import { createFileRoute, Link } from "@tanstack/react-router";
import { ArrowRight, Network } from "lucide-react";
import CatalogItem from "shared/src/components/ui/catalog-item";
import { PageHeader } from "@eunomia/ui";
import { Button } from "@eunomia/ui";
import { PageLayout } from "@eunomia/ui";
import { useFederatedCatalog } from "shared/src/data/useFederatedCatalog";
import { usePeerConnections } from "shared/src/data/peerConnection";
import { useGetMyself } from "shared/src/data/orval/participants/participants";

const TITLE = "Catalog Browser";
const DESCRIPTION =
  "Public catalogs of the dataspace and the catalogs of the participants you have access to.";

const RouteComponent = () => {
  const federated = useFederatedCatalog();
  const connections = usePeerConnections();

  // Until this connector is known, its own card would pass for a peer.
  const myself = useGetMyself();
  const myAgent = myself.data?.status === 200 ? myself.data.data : undefined;

  if (federated.state === "loading" || connections.isLoading || myself.isLoading) {
    return (
      <PageLayout>
        <PageHeader title={TITLE} description={DESCRIPTION} />
        <p className="text-sm text-muted-foreground">Loading...</p>
      </PageLayout>
    );
  }

  if (federated.state === "no-authority") {
    return (
      <PageLayout>
        <PageHeader title={TITLE} description={DESCRIPTION} />
        <div className="flex flex-col items-center gap-2 rounded-lg border border-dashed border-ink/15 px-6 py-10 text-center">
          <Network className="h-8 w-8 text-muted-foreground/50" />
          <h2 className="text-sm font-semibold text-ink/90">You haven't joined a dataspace yet</h2>
          <p className="max-w-sm text-xs text-muted-foreground">
            Add an authority to discover other participants and browse their catalogs.
          </p>
          <Link to="/authority/new" className="mt-2">
            <Button size="sm">
              Join a dataspace
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

  const isOwn = (participantId: string) => participantId === myAgent?.participant_id;
  // Whether this user can read any peer's catalog yet; if not, the first one to ask is highlighted.
  const onboardedWithKnownProvider = agents.some(
    (prov) => connections.statusOf(prov.participant_id) === "catalog",
  );
  const firstPeer = onboardedWithKnownProvider
    ? undefined
    : agents.find(
        (p) => !isOwn(p.participant_id) && connections.statusOf(p.participant_id) === "none",
      );

  return (
    <PageLayout>
      <PageHeader title={TITLE} description={DESCRIPTION} />
      {firstPeer && (
        <p className="text-sm text-muted-foreground">
          Pick a participant to request access and browse its catalog.
        </p>
      )}
      {agents.length === 0 ? (
        <div className="rounded-lg border border-dashed border-ink/15 px-6 py-10 text-center text-sm text-muted-foreground">
          The authority lists no participants yet.
        </div>
      ) : (
        <div className="grid grid-cols-1 gap-3 md:grid-cols-2 xl:grid-cols-3">
          {agents.map((p) => {
            const highlighted = p === firstPeer;

            return (
              <div
                key={p.participant_id}
                // Same corners as the card, so the highlight fits it.
                className={`rounded-xl ${
                  highlighted ? "ring-2 ring-secondary-400 shadow-md animate-pulse" : ""
                }`}
              >
                <CatalogItem
                  organizationName={p.participant_nick ?? "Unknown"}
                  id={p.participant_id ?? null}
                  connection={connections.statusOf(p.participant_id)}
                  approvedGrant={connections.approvedGrantOf(p.participant_id)}
                  participant={p}
                  pendingGrantId={connections.pendingGrantOf(p.participant_id)?.id}
                  ownCatalog={isOwn(p.participant_id)}
                />
              </div>
            );
          })}
        </div>
      )}
    </PageLayout>
  );
};

export const Route = createFileRoute("/catalog/")({
  component: RouteComponent,
  pendingComponent: () => <div>Loading...</div>,
});
