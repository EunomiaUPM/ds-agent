import React, { useState } from "react";
import { CatalogRpc, DspViews, dspIssued, dspTitle } from "../../data/catalog";
import avatarImg from "../../../public/avatar.png";
import { Avatar } from "@eunomia/ui";
import { Heading } from "@eunomia/ui";
import { Link, useNavigate } from "@tanstack/react-router";
import { FormatDate } from "@eunomia/ui";
import { Lock, Database, Server, Calendar, ArrowUpRight } from "lucide-react";
import { Popover, PopoverContent, PopoverTrigger } from "@eunomia/ui";
import { Badge } from "@eunomia/ui";
import { useRpcSetupCatalogRequest } from "shared/src/data/orval/catalog-rp-c/catalog-rp-c";
import { ConnectPeerForm } from "shared/src/components/ConnectPeerForm";
import { accessLabel, type PeerConnection } from "shared/src/data/peerConnection";
import type { SentGrant } from "shared/src/data/orval/model/sentGrant";
import type { FederatedParticipant } from "shared/src/data/useFederatedCatalog";
import { Card, CardHeader, CardFooter } from "@eunomia/ui";

interface CatalogItemProps {
  date?: string | undefined;
  /** Known only with access to the catalog; `?` until then. */
  datasetNumber?: number;
  organizationName: string;
  id: string | null;
  title?: string;
  /** What this user can read of the peer. */
  connection?: PeerConnection;
  /** This user's latest approved grant with the peer, widened instead of asking for a new one. */
  approvedGrant?: SentGrant;
  /** The peer as the federated catalog lists it; needed to ask for access. */
  participant?: FederatedParticipant;
  /** This user's grant with the peer still on its way, resumed from the card. */
  pendingGrantId?: string;
  ownCatalog?: boolean;
}

const CatalogItem: React.FC<CatalogItemProps> = ({
  date,
  datasetNumber,
  organizationName,
  id,
  title,
  connection = "none",
  approvedGrant,
  participant,
  pendingGrantId,
  ownCatalog = false,
}) => {
  const navigate = useNavigate();
  const connected = ownCatalog || connection === "catalog";
  const canConnect =
    id !== null &&
    !ownCatalog &&
    (connection === "none" || (connection === "pending" && !!pendingGrantId)) &&
    !!participant;

  const unavailableCatalogClasses =
    id === null ? "opacity-65 grayscale cursor-not-allowed" : "cursor-pointer";

  const { mutate, data } = useRpcSetupCatalogRequest();

  // Only with a grant of this user that covers the catalog: reading never asks for more access.
  // The own catalog is local: there is no peer to ask through DSP.
  React.useEffect(() => {
    if (id && !ownCatalog && connection === "catalog") {
      mutate({
        data: CatalogRpc.catalog(id),
      });
    }
  }, [id, ownCatalog, connection, mutate]);

  const liveCatalog =
    connection === "catalog" && data?.status === 200 ? CatalogRpc.response(data.data) : undefined;
  const liveTitle = liveCatalog ? dspTitle(liveCatalog) : undefined;
  const liveDate = liveCatalog ? dspIssued(liveCatalog) : undefined;
  const liveDatasetNr = liveCatalog ? DspViews.datasets(liveCatalog).length : undefined;

  const displayTitle = liveTitle || title;
  const displayDate = liveDate || date;
  const displayDatasetNr = liveDatasetNr ?? datasetNumber;

  const headingText = displayTitle ? displayTitle : `${organizationName}'s Catalog`;
  const headingNode = (
    <div className="group/title flex items-center justify-between gap-2">
      <Heading
        level="h4"
        className="capitalize !mb-0 underline-offset-2 group-hover/title:underline text-base font-semibold"
      >
        {headingText}
      </Heading>
      <ArrowUpRight className="h-4 w-4 text-muted-foreground group-hover/title:text-brand-sky transition-colors flex-shrink-0" />
    </div>
  );

  let headingLink: React.ReactNode;
  const [openForm, setOpenForm] = useState(false);
  // Fixed when the form opens: a grant sent from it turns the connection pending meanwhile.
  const [resumed, setResumed] = useState<string | undefined>();

  const handleOpenChange = (isOpen: boolean) => {
    if (isOpen) setResumed(connection === "pending" ? pendingGrantId : undefined);
    setOpenForm(isOpen);
  };

  // Kept open once the grant is sent: the form still has to show what the peer asks for.
  if (canConnect || (openForm && !!participant)) {
    headingLink = (
      <Popover open={openForm} onOpenChange={handleOpenChange}>
        <PopoverTrigger asChild>
          <button type="button" className="p-0 m-0 text-left w-full">
            {headingNode}
          </button>
        </PopoverTrigger>
        <PopoverContent
          align="start"
          className="w-96 rounded-xl border border-ink/15 bg-background-800/95 shadow-2xl backdrop-blur-xl"
        >
          <div className="flex gap-2 items-center mb-3">
            <Lock className="h-4 w-4" />
            <Heading level="h5" className="!mb-0">
              Request access to {organizationName}
            </Heading>
          </div>
          <ConnectPeerForm
            participant={participant!}
            widen={approvedGrant && !resumed ? { grantId: approvedGrant.id } : undefined}
            pending={resumed}
            onCancel={() => handleOpenChange(false)}
            onSent={() => {
              handleOpenChange(false);
              navigate({
                to: "/catalog/participant/$participantId",
                params: { participantId: id! },
              });
            }}
          />
        </PopoverContent>
      </Popover>
    );
  } else if (ownCatalog) {
    headingLink = <Link to={"/my-catalog"}>{headingNode}</Link>;
  } else if (id !== null && connection !== "none") {
    headingLink = (
      <Link to={"/catalog/participant/$participantId"} params={{ participantId: id }}>
        {headingNode}
      </Link>
    );
  } else {
    headingLink = headingNode;
  }

  return (
    <Card
      variant="interactive"
      className={`h-full flex flex-col justify-between max-w-lg transition-all duration-200 ${unavailableCatalogClasses} ${
        connected && !ownCatalog ? "border-emerald-500/30 hover:border-emerald-500/60" : ""
      }`}
    >
      <CardHeader className="pb-3 space-y-3">
        <div className="flex items-center justify-between gap-2">
          <div className="flex items-center gap-2 min-w-0">
            <Avatar src={avatarImg} />
            <span className="font-semibold text-sm text-foreground capitalize truncate">
              {organizationName}
            </span>
          </div>

          {ownCatalog ? (
            <Badge variant="role" dsrole="Provider" className="text-xs">
              My Catalog
            </Badge>
          ) : id === null ? null : connection === "catalog" ? (
            <Badge variant="status" state="success" className="text-xs">
              {accessLabel(approvedGrant?.granted ?? [])}
            </Badge>
          ) : connection === "datasets" ? (
            <Badge variant="status" state="active" className="text-xs">
              {accessLabel(approvedGrant?.granted ?? [])}
            </Badge>
          ) : connection === "pending" ? (
            <Badge variant="status" state="pending" className="text-xs">
              Awaiting response
            </Badge>
          ) : (
            <Badge variant="status" state="error" className="text-xs">
              No access
            </Badge>
          )}
        </div>

        <div className="pt-1">
          {headingLink}
          <p className="mt-1.5 text-xs text-muted-foreground line-clamp-2 leading-relaxed">
            This is the catalog of{" "}
            <span className="capitalize text-foreground/90 font-medium">{organizationName}</span>.
            Browse published datasets, distributions, and associated dataservices.
          </p>
        </div>
      </CardHeader>

      <CardFooter className="pt-3 flex items-center justify-between gap-3 text-xs border-t border-ink/5 bg-background-800/20 rounded-b-xl">
        <div className="flex items-center gap-1 text-muted-foreground text-xs">
          <Calendar className="h-3 w-3 opacity-60" />
          <FormatDate date={displayDate} />
        </div>

        <div className="flex items-center gap-2">
          <span className="inline-flex items-center gap-1 whitespace-nowrap px-2 py-0.5 rounded-md bg-ink/5 text-xs text-muted-foreground">
            <Server className="h-3 w-3 text-brand-sky" /> 1 service
          </span>
          {/* Unknown until the catalog is asked for: no count rather than a guess. */}
          {displayDatasetNr !== undefined && (
            <span className="inline-flex items-center gap-1 whitespace-nowrap px-2 py-0.5 rounded-md bg-ink/5 text-xs text-muted-foreground">
              <Database className="h-3 w-3 text-emerald-700 dark:text-emerald-400" />
              {displayDatasetNr} {displayDatasetNr === 1 ? "dataset" : "datasets"}
            </span>
          )}
        </div>
      </CardFooter>
    </Card>
  );
};

export default CatalogItem;
