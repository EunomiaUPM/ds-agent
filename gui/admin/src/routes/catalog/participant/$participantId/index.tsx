import { createFileRoute, Link, redirect, useNavigate } from "@tanstack/react-router";
import { getGetMyselfQueryOptions } from "shared/src/data/orval/participants/participants";
import { useRpcSetupCatalogRequest } from "shared/src/data/orval/catalog-rp-c/catalog-rp-c";
import { CatalogRpc, DspViews, dspIssued, dspTitle, text } from "shared/src/data/catalog";
import { PageLayout } from "@eunomia/ui";
import { PageHeader } from "@eunomia/ui";
import { FieldGrid, Identifier } from "@eunomia/ui";
import { FormatDate } from "@eunomia/ui";
import { encodePathId, formatUrn, pathIdParams } from "@eunomia/ui";
import React, { useEffect, useState, useMemo } from "react";
import { Skeleton } from "@eunomia/ui";
import DatasetItem from "shared/src/components/ui/dataset-item";
import { CountBadge } from "@eunomia/ui";
import { useFederatedCatalog } from "shared/data/useFederatedCatalog";
import { Button } from "@eunomia/ui";
import { Input } from "@eunomia/ui";
import { Card, CardTitle, CardDescription } from "@eunomia/ui";
import { ExternalLink, Loader2, ShieldCheck } from "lucide-react";
import { coversDataset, grantedDatasets, usePeerConnections } from "shared/src/data/peerConnection";
import { ConnectPeerForm } from "shared/src/components/ConnectPeerForm";

function RouteComponent() {
  const { participantId } = Route.useParams();
  const federated = useFederatedCatalog();
  const { mutate, data, isPending, error } = useRpcSetupCatalogRequest();
  const [searchQuery, setSearchQuery] = useState("");

  const navigate = useNavigate();
  const connections = usePeerConnections();
  const connection = connections.statusOf(participantId);
  const grant = connections.approvedGrantOf(participantId);
  const pendingGrant = connections.pendingGrantOf(participantId);
  const granted = grant?.granted ?? [];
  const [widening, setWidening] = useState<{ id: string; title: string } | null>(null);

  const participant =
    federated.state === "ok"
      ? federated.agents.find((p: any) => p.participant_id === participantId)
      : undefined;

  // Only with a grant of this user that covers the catalog; reading never asks for more access.
  useEffect(() => {
    if (connection !== "catalog") return;
    mutate({
      data: CatalogRpc.catalog(participantId),
    });
  }, [participantId, connection, mutate]);

  const catalog = data?.status === 200 ? CatalogRpc.response(data.data) : undefined;
  const datasets = DspViews.datasets(catalog).map((node) => ({ ...DspViews.resource(node), node }));
  const service = DspViews.service(catalog);
  const dataservice = service && {
    "@id": String(service["@id"] ?? ""),
    title: dspTitle(service),
    endpointURL: text(service.endpointURL),
  };

  const filteredDatasets = useMemo(() => {
    if (!searchQuery.trim()) return datasets;
    const q = searchQuery.toLowerCase();
    return datasets.filter(
      (d) =>
        d.title.toLowerCase().includes(q) ||
        d.description.toLowerCase().includes(q) ||
        d.id.toLowerCase().includes(q),
    );
  }, [datasets, searchQuery]);

  if (federated.state === "loading" || connections.isLoading || isPending) {
    return (
      <PageLayout>
        <PageHeader eyebrow="Peer catalog" title={<Skeleton className="h-7 w-64" />} />
        <Skeleton className="h-16 w-full rounded-lg" />
        <div className="grid grid-cols-1 gap-3 md:grid-cols-2 2xl:grid-cols-3">
          <Skeleton className="h-36 w-full rounded-xl" />
          <Skeleton className="h-36 w-full rounded-xl" />
        </div>
      </PageLayout>
    );
  }

  if (federated.state === "no-authority") {
    return (
      <PageLayout>
        <Card className="p-8 text-center max-w-lg mx-auto">
          <CardTitle>No Authority Found</CardTitle>
          <CardDescription className="mt-2">
            Connect to an authority node to browse federated catalogs.
          </CardDescription>
        </Card>
      </PageLayout>
    );
  }

  if (error || federated.state === "error") {
    return (
      <PageLayout>
        <Card className="p-8 text-center max-w-lg mx-auto border-destructive/40">
          <CardTitle className="text-destructive">Catalog Discovery Failed</CardTitle>
          <CardDescription className="mt-2 text-muted-foreground">
            {error?.message || "Failed to resolve federated catalog for this participant."}
          </CardDescription>
          <Button variant="outline" className="mt-4" onClick={() => window.location.reload()}>
            Retry
          </Button>
        </Card>
      </PageLayout>
    );
  }

  if (connection !== "catalog") {
    const nick = participant?.participant_nick || "this participant";
    return (
      <PageLayout>
        <PageHeader title={`${participant?.participant_nick || "Participant"}'s Catalog`} />
        <Card className="p-6 max-w-lg">
          <CardTitle className="mb-4">Request access to {nick}</CardTitle>
          {connection === "datasets" && (
            <div className="mb-4 space-y-1">
              <CardDescription>
                You can open these datasets. Ask below to browse the whole catalog.
              </CardDescription>
              {grantedDatasets(granted).map((id) => (
                <Link
                  key={id}
                  to={`/catalog/participant/${encodePathId(participantId)}/dataset/${id}`}
                  className="block font-mono text-xs underline"
                >
                  {formatUrn(id)}
                </Link>
              ))}
            </div>
          )}
          {connection === "pending" && participant && pendingGrant ? (
            <ConnectPeerForm
              key={pendingGrant.id}
              participant={participant}
              pending={pendingGrant.id}
              onCancel={() => navigate({ to: "/catalog" })}
            />
          ) : connection === "pending" ? (
            <CardDescription className="flex items-center gap-2">
              <Loader2 className="h-4 w-4 animate-spin" />
              Waiting for {nick} to accept your request; the catalog opens once it does.
            </CardDescription>
          ) : participant ? (
            <ConnectPeerForm
              participant={participant}
              widen={grant ? { grantId: grant.id } : undefined}
              onCancel={() => navigate({ to: "/catalog" })}
            />
          ) : (
            <CardDescription>
              This participant is not in the federated catalog of your authority.
            </CardDescription>
          )}
        </Card>
      </PageLayout>
    );
  }

  if (!catalog) return null;

  const nick = participant?.participant_nick || "Participant";
  const homepage = DspViews.homepage(catalog);

  return (
    <PageLayout>
      <PageHeader eyebrow="Peer catalog" title={dspTitle(catalog) || `${nick}'s catalog`} />

      <FieldGrid
        columns={4}
        fields={[
          { label: "Participant", value: <span className="capitalize">{nick}</span> },
          { label: "Issued", value: <FormatDate date={dspIssued(catalog)} /> },
          {
            label: "Data service",
            value: dataservice && <Identifier value={dataservice["@id"]} />,
          },
          {
            label: "DSP endpoint",
            value: dataservice?.endpointURL && (
              <span className="break-all font-mono text-xs">{dataservice.endpointURL}</span>
            ),
          },
          {
            label: "Homepage",
            span: 4,
            hidden: !homepage,
            value: (
              <a
                href={homepage}
                target="_blank"
                rel="noreferrer"
                className="inline-flex items-center gap-1 text-brand-sky hover:underline"
              >
                {homepage}
                <ExternalLink className="h-3 w-3" />
              </a>
            ),
          },
        ]}
      />

      <div className="flex items-center justify-between gap-3 pt-2">
        <h2 className="flex items-center gap-2 text-xs font-semibold uppercase tracking-wide text-ink/80">
          Offered datasets <CountBadge n={filteredDatasets.length} showZero />
        </h2>
        <Input
          placeholder="Filter datasets..."
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          onClear={() => setSearchQuery("")}
          className="h-8 w-full text-xs sm:w-72"
        />
      </div>

      {widening && participant && grant && (
        <Card className="p-4">
          <ConnectPeerForm
            participant={participant}
            widen={{ grantId: grant.id, datasetId: widening.id, title: widening.title }}
            onSent={() => setWidening(null)}
            onCancel={() => setWidening(null)}
          />
        </Card>
      )}

      {filteredDatasets.length > 0 ? (
        <div className="grid grid-cols-1 gap-3 md:grid-cols-2 2xl:grid-cols-3">
          {filteredDatasets.map((dataset) => (
            <div key={dataset.id} className="flex flex-col gap-2">
              <DatasetItem
                title={dataset.title}
                description={dataset.description}
                date={dataset.issued}
                prevRoute={participantId}
                datasetId={dataset.id}
                ownDataset={false}
                dataset={dataset.node}
              />
              {!coversDataset(granted, dataset.id) && (
                <Button
                  size="sm"
                  variant="outline"
                  className="self-start text-xs"
                  onClick={() => setWidening({ id: dataset.id, title: dataset.title })}
                >
                  <ShieldCheck className="mr-1 h-3.5 w-3.5" />
                  Request access
                </Button>
              )}
            </div>
          ))}
        </div>
      ) : (
        <p className="rounded-lg border border-dashed border-ink/15 py-8 text-center text-sm text-ink/60">
          {searchQuery
            ? `No dataset matches "${searchQuery}".`
            : "This participant offers no datasets in this catalog."}
        </p>
      )}
    </PageLayout>
  );
}

export const Route = createFileRoute("/catalog/participant/$participantId/")({
  params: pathIdParams("participantId"),
  component: RouteComponent,
  // This connector is not among its own peers: its catalog is the local one.
  beforeLoad: async ({ params, context: { queryClient } }) => {
    const myself = await queryClient.ensureQueryData(getGetMyselfQueryOptions());
    if (myself.status === 200 && myself.data.participant_id === params.participantId) {
      throw redirect({ to: "/my-catalog" });
    }
  },
});
