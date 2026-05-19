import React, { useState } from "react";
import avatarImg from "../../../public/avatar.png";
import Avatar from "./avatar-img";
import Heading from "shared/src/components/ui/heading";
import { Link } from "@tanstack/react-router";
import { FormatDate } from "shared/src/components/ui/format-date";
import { CheckCircle2, Lock } from "lucide-react";
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
    <Heading level="h4" className="capitalize mb-3 underline-offset-2 hover:underline">
      {headingText}
    </Heading>
  );

  let headingLink: React.ReactNode;
  const [openDialog, setOpenDialog] = useState(false);

  //verify if user is onboarded with any provider to decide whether we show the wizard or we dont
  const { data: participantsResponse } = useGetAllParticipants();
  const localParticipants = participantsResponse?.status === 200 ? participantsResponse.data : [];

  let isOnboardedWithKnownProvider = localParticipants.some(
    (lp) => lp.participant_type !== "Authority" && lp.is_me === false,
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
                to="/providers/new"
                search={{ url: unauthRedirect.url, slug: unauthRedirect.slug }}
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
    <div
      className={`catalog-card h-full bg-background-200/15  hover:bg-background-200/30 transition-all border rounded-md flex flex-col p-4 gap-3 justify-between max-w-lg ${unavailableCatalogClasses} ${isAuthenticated ? "border-emerald-500/40" : "border-white/10"} ${ownCatalog && isAuthenticated ? "border-white/10" : ""}`}
    >
      <div className="catalog-top">
        <div className="catalog-dates-container flex gap-3 text-sm tracking-wide items-start justify-between">
          <div className="catalog-dates-created flex gap-1 mb-2">
            <p className="text-sm">Created at:</p>
            <FormatDate date={displayDate} />
          </div>
          {isAuthenticated ? (
            !ownCatalog ? (
              <span className="flex items-center gap-1 text-xs text-emerald-400 font-medium">
                <CheckCircle2 className="h-3.5 w-3.5" />
                Authenticated
              </span>
            ) : (
              <Badge
                variant="detail"
                size="default"
                className="uppercase text-blue-300 font-semibold mb-3"
              >
                My own catalog
              </Badge>
            )
          ) : id !== null ? (
            <span className="flex items-center gap-1 text-xs text-muted-foreground font-medium text-red-400">
              <Lock className="h-3.5 w-3.5" />
              Auth required
            </span>
          ) : null}
        </div>
        <div className="catalog-text-container">
          {headingLink}
          <p className="mb-2 line-clamp-3 text-sm">
            This is the catalog of <span className="capitalize">{organizationName}</span>, who is
            also part of this dataspace. Click on the catalog name to see the datasets and
            dataservice they offer.
          </p>
        </div>
      </div>
      <div className="catalog-bottom">
        <div className="catalog-participant-container flex gap-2 justify-start items-center mb-2">
          <Avatar src={avatarImg} />
          <Heading level="h5" className="capitalize !mb-0">
            {organizationName}
          </Heading>
        </div>
        <div className="catalog-items-container flex justify-end gap-2 text-sm italic">
          <p> 1 Dataservice </p>
          <p> {displayDatasetNr} Datasets </p>
        </div>
      </div>
    </div>
  );
};

export default CatalogItem;
