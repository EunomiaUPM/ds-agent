import React, { useState } from "react";
import avatarImg from "../../../public/avatar.png";
import Avatar from "./avatar-img";
import Heading from "shared/src/components/ui/heading";
import { Link } from "@tanstack/react-router";
import { FormatDate } from "shared/src/components/ui/format-date";
import { CheckCircle2, Lock, Database, Server, Calendar, ArrowUpRight } from "lucide-react";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
  DialogClose,
} from "shared/src/components/ui/dialog";
import { Button } from "shared/src/components/ui/button";
import WizardDialog from "shared/src/components/WizardDialog";
import { useRef } from "react";
import { useGetAllParticipants } from "shared/src/data/orval/participants/participants";
import { Badge } from "shared/src/components/ui/badge";
import { useRpcSetupCatalogRequest } from "shared/src/data/orval/catalog-rp-c/catalog-rp-c";
import {
  Card,
  CardHeader,
  CardTitle,
  CardContent,
  CardFooter,
} from "shared/src/components/ui/card";

interface CatalogItemProps {
  date?: string | undefined;
  datasetNumber: number;
  organizationName: string;
  id: string | null;
  title?: string;
  isAuthenticated?: boolean;
  unauthRedirect?: { url: string; slug: string } | null;
  onUnauthDialogClose?: () => void;
  ownCatalog?: boolean;
}

const CatalogItem: React.FC<CatalogItemProps> = ({
  date,
  datasetNumber,
  organizationName,
  id,
  title,
  isAuthenticated,
  unauthRedirect,
  onUnauthDialogClose,
  ownCatalog = false,
}) => {
  const unavailableCatalogClasses =
    id === null ? "opacity-65 grayscale cursor-not-allowed" : "cursor-pointer";

  const highlightButtonClasses = unauthRedirect
    ? "animate-pulse bg-secondary-600 hover:bg-secondary-500 ring-2 ring-secondary-400"
    : "";

  const labelConnectRef = useRef<HTMLElement | null>(null);
  const [wizardConnectOpen, setWizardConnectOpen] = useState(false);

  const { mutate, data } = useRpcSetupCatalogRequest();

  React.useEffect(() => {
    if (id) {
      mutate({
        data: {
          associatedAgentPeer: id,
          filter: [],
          noCache: true,
        },
      });
    }
  }, [id, mutate]);

  const liveCatalog = data?.status === 200 ? data.data : undefined;
  const liveTitle = liveCatalog?.response?.title;
  const liveDate = liveCatalog?.response?.issued;
  const liveDatasetNr = liveCatalog?.response?.dataset?.length;

  const displayTitle = liveTitle || title;
  const displayDate = liveDate || date;
  const displayDatasetNr = liveDatasetNr || datasetNumber;

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
  const [openDialog, setOpenDialog] = useState(false);

  //verify if user is onboarded with any provider to decide whether we show the wizard or we dont
  const { data: participantsResponse } = useGetAllParticipants();
  const localParticipants = participantsResponse?.status === 200 ? participantsResponse.data : [];

  let isOnboardedWithKnownProvider = localParticipants.some(
    (lp) => lp.participant_type !== "Authority",
  );

  // open the wizard only after the dialog has been opened and the title anchor is mounted
  React.useEffect(() => {
    let t: any;
    if (openDialog) {
      // schedule on next tick so the Dialog content mounts and labelConnectRef is set
      t = setTimeout(() => setWizardConnectOpen(true), 50);
    } else {
      setWizardConnectOpen(false);
    }
    return () => clearTimeout(t);
  }, [openDialog]);

  const handleOpenChange = (isOpen: boolean) => {
    setOpenDialog(isOpen);
    if (!isOpen && onUnauthDialogClose) {
      onUnauthDialogClose();
    }
  };

  if (unauthRedirect) {
    headingLink = (
      <>
        <button
          type="button"
          onClick={() => setOpenDialog(true)}
          className="p-0 m-0 text-left w-full"
        >
          {headingNode}
        </button>

        <Dialog open={openDialog} onOpenChange={handleOpenChange}>
          <DialogContent>
            <DialogHeader>
              <DialogTitle
                ref={(el) => (labelConnectRef.current = el as any)}
                className="flex gap-2 items-center"
              >
                <Lock className="h-5 w-5"></Lock>
                <Heading level="h4" className="!mb-0">
                  Access required
                </Heading>
              </DialogTitle>
              <DialogDescription>
                {isOnboardedWithKnownProvider ? (
                  ""
                ) : (
                  <WizardDialog
                    open={wizardConnectOpen}
                    onClose={() => setWizardConnectOpen(false)}
                    anchorRef={labelConnectRef}
                    sectionTitle="Connection with Participant Tutorial"
                    step="2 of 3"
                    align="left"
                    title="Connection with Dataspace Participant required"
                    content={
                      <>
                        You can only access the catalog of a participant if you are connected to
                        them. Click on the button <strong>"Request connection"</strong> to connect
                        with the owner of the catalog.
                      </>
                    }
                  />
                )}
                You don't have permission to access this catalog. <br /> First, you need to connect
                with <strong>{organizationName}</strong>.
              </DialogDescription>
            </DialogHeader>
            <DialogFooter>
              <DialogClose asChild>
                <Button variant="ghost">Keep browsing</Button>
              </DialogClose>
              <Link
                to="/connections/sent/new"
                search={{ url: unauthRedirect.url, nick: unauthRedirect.slug }}
              >
                <Button className={highlightButtonClasses}>Request connection</Button>
              </Link>
            </DialogFooter>
          </DialogContent>
        </Dialog>
      </>
    );
  } else if (id !== null) {
    headingLink = (
      <Link
        to={"/catalog/participant/$id/"}
        params={{
          id: id!,
        }}
      >
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
        isAuthenticated && !ownCatalog ? "border-emerald-500/30 hover:border-emerald-500/60" : ""
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

          {isAuthenticated ? (
            !ownCatalog ? (
              <Badge variant="status" state="active" className="text-xs">
                Authenticated
              </Badge>
            ) : (
              <Badge variant="role" dsrole="Provider" className="text-xs">
                My Catalog
              </Badge>
            )
          ) : id !== null ? (
            <Badge variant="status" state="error" className="text-xs">
              Auth Required
            </Badge>
          ) : null}
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
          <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-md bg-ink/5 text-xs font-mono text-muted-foreground">
            <Server className="h-3 w-3 text-brand-sky" /> 1 Service
          </span>
          <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-md bg-ink/5 text-xs font-mono text-muted-foreground">
            <Database className="h-3 w-3 text-emerald-700 dark:text-emerald-400" /> {displayDatasetNr} Datasets
          </span>
        </div>
      </CardFooter>
    </Card>
  );
};

export default CatalogItem;
