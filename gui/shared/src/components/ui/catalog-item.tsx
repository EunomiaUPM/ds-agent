import React, { useState } from "react";
import { CatalogRpc, DspViews, dspIssued, dspTitle } from "../../data/catalog";
import avatarImg from "../../../public/avatar.png";
import { Avatar } from "@eunomia/ui";
import { Heading } from "@eunomia/ui";
import { Link, useNavigate } from "@tanstack/react-router";
import { FormatDate } from "@eunomia/ui";
import { Lock, Database, Server, Calendar, ArrowUpRight } from "lucide-react";
import { Popover, PopoverContent, PopoverTrigger } from "@eunomia/ui";
import { WizardDialog } from "@eunomia/ui";
import { useRef } from "react";
import { Badge } from "@eunomia/ui";
import { useRpcSetupCatalogRequest } from "shared/src/data/orval/catalog-rp-c/catalog-rp-c";
import { ConnectPeerForm } from "shared/src/components/ConnectPeerForm";
import type { PeerConnection } from "shared/src/data/peerConnection";
import type { FederatedParticipant } from "shared/src/data/useFederatedCatalog";
import { Card, CardHeader, CardFooter } from "@eunomia/ui";

interface CatalogItemProps {
  date?: string | undefined;
  /** Known only once connected; `?` until then. */
  datasetNumber?: number;
  organizationName: string;
  id: string | null;
  title?: string;
  /** This user's connection with the peer. */
  connection?: PeerConnection;
  /** The peer as the federated catalog lists it; needed to connect. */
  participant?: FederatedParticipant;
  /** Shows the tutorial step on the connection form. */
  showTutorial?: boolean;
  onConnectClose?: () => void;
  ownCatalog?: boolean;
}

const CatalogItem: React.FC<CatalogItemProps> = ({
  date,
  datasetNumber,
  organizationName,
  id,
  title,
  connection = "none",
  participant,
  showTutorial = false,
  onConnectClose,
  ownCatalog = false,
}) => {
  const navigate = useNavigate();
  const connected = ownCatalog || connection === "approved";
  const canConnect =
    id !== null &&
    !ownCatalog &&
    (connection === "none" || connection === "rejected") &&
    !!participant;

  const unavailableCatalogClasses =
    id === null ? "opacity-65 grayscale cursor-not-allowed" : "cursor-pointer";

  const labelConnectRef = useRef<HTMLElement | null>(null);
  const [wizardConnectOpen, setWizardConnectOpen] = useState(false);

  const { mutate, data } = useRpcSetupCatalogRequest();

  // Only with a grant of this user: asking without one would get a grant on its own, with no
  // visibility chosen. The own catalog is local: there is no peer to ask through DSP.
  React.useEffect(() => {
    if (id && !ownCatalog && connection === "approved") {
      mutate({
        data: CatalogRpc.catalog(id),
      });
    }
  }, [id, ownCatalog, connection, mutate]);

  const liveCatalog = data?.status === 200 ? CatalogRpc.response(data.data) : undefined;
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

  // open the wizard only after the form has been opened and the title anchor is mounted
  React.useEffect(() => {
    let t: any;
    if (openForm && showTutorial) {
      t = setTimeout(() => setWizardConnectOpen(true), 50);
    } else {
      setWizardConnectOpen(false);
    }
    return () => clearTimeout(t);
  }, [openForm, showTutorial]);

  const handleOpenChange = (isOpen: boolean) => {
    setOpenForm(isOpen);
    if (!isOpen && onConnectClose) {
      onConnectClose();
    }
  };

  if (canConnect) {
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
          <div
            ref={(el) => (labelConnectRef.current = el as any)}
            className="flex gap-2 items-center mb-3"
          >
            <Lock className="h-4 w-4" />
            <Heading level="h5" className="!mb-0">
              Connect with {organizationName}
            </Heading>
          </div>
          {showTutorial && (
            <WizardDialog
              open={wizardConnectOpen}
              onClose={() => setWizardConnectOpen(false)}
              anchorRef={labelConnectRef}
              sectionTitle="Connection with Participant Tutorial"
              step="2 of 2"
              align="left"
              title="Connection with Dataspace Participant required"
              content={
                <>
                  You can only access the catalog of a participant once you are connected to them.
                  Choose who else can see the connection and click <strong>"Connect"</strong>.
                </>
              }
            />
          )}
          <ConnectPeerForm
            participant={participant!}
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
  } else if (id !== null && (connection === "approved" || connection === "pending")) {
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
          ) : id === null ? null : connection === "approved" ? (
            <Badge variant="status" state="active" className="text-xs">
              Authenticated
            </Badge>
          ) : connection === "pending" ? (
            <Badge variant="status" state="process" className="text-xs">
              Awaiting response
            </Badge>
          ) : connection === "rejected" ? (
            <Badge variant="status" state="danger" className="text-xs">
              Rejected
            </Badge>
          ) : (
            <Badge variant="status" state="error" className="text-xs">
              Auth Required
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
