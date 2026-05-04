import React, { useState } from "react";
import Heading from "shared/src/components/ui/heading";
import { Link } from "@tanstack/react-router";
import { FormatDate } from "shared/src/components/ui/format-date";
import { CheckCircle2, Lock } from "lucide-react";
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription, DialogFooter, DialogClose } from "shared/src/components/ui/dialog";
import { Button } from "shared/src/components/ui/button";

interface CatalogItemProps {
  date: string;
  datasetNumber: number;
  organizationName: string;
  id: string | null;
  title?: string;
  isAuthenticated?: boolean;
  unauthRedirect?: { url: string; slug: string } | null;
}

const CatalogItem: React.FC<CatalogItemProps> = ({
  date,
  datasetNumber,
  organizationName,
  id,
  title,
  isAuthenticated,
  unauthRedirect,
}) => {
  const unavailableCatalogClasses =
    id === null ? "opacity-65 grayscale cursor-not-allowed" : "cursor-pointer";

  const highlightRingClasses = !isAuthenticated ? "ring-2 ring-secondary-400 shadow-md animate-pulse" : ""

  const headingText = title ? title : `${organizationName}'s Catalog for Demo`;
  const headingNode = (
    <Heading level="h3" className="capitalize mb-3 underline-offset-2 hover:underline">
      {headingText}
    </Heading>
  );

  let headingLink: React.ReactNode;
  const [openDialog, setOpenDialog] = useState(false);

  if (unauthRedirect) {
    headingLink = (
      <>
        <button type="button" onClick={() => setOpenDialog(true)} className="p-0 m-0 text-left w-full">
          {headingNode}
        </button>

        <Dialog open={openDialog} onOpenChange={setOpenDialog}>
          <DialogContent>
            <DialogHeader>
              <DialogTitle>Access required</DialogTitle>
              <DialogDescription>
                You don't have permission to access this catalog. <br/> First, you need to connect with <strong>{organizationName}</strong>.
              </DialogDescription>
            </DialogHeader>
            <DialogFooter>
              <DialogClose asChild>
                <Button variant="ghost" onClick={() => setOpenDialog(false)}>Keep browsing</Button>
              </DialogClose>
              <Link to="/providers/new" search={{ url: unauthRedirect.url, slug: unauthRedirect.slug }}>
                <Button>Request connection</Button>
              </Link>
            </DialogFooter>
          </DialogContent>
        </Dialog>
      </>
    );
  } else if (id !== null) {
    headingLink = (
      <Link to="/catalog/participant/$participantId" params={{ participantId: id }}>
        {headingNode}
      </Link>
    );
  } else {
    headingLink = headingNode;
  }

  return (
    <div
      className={`catalog-card bg-background-200/15  hover:bg-background-200/30 transition-all border rounded-md flex flex-col p-4 gap-3 max-w-lg ${unavailableCatalogClasses} ${isAuthenticated ? "border-emerald-500/40" : "border-white/10"} `}
    >
      <div className="catalog-dates-container flex gap-3 text-sm tracking-wide items-center justify-between">
        <div className="catalog-dates-created flex gap-1">
          <p>Created at:</p>
          <FormatDate date={date} />
        </div>
        {isAuthenticated ? (
          <span className="flex items-center gap-1 text-xs text-emerald-400 font-medium">
            <CheckCircle2 className="h-3.5 w-3.5" />
            Authenticated
          </span>
        ) : id !== null ? (
          <span className="flex items-center gap-1 text-xs text-muted-foreground font-medium">
            <Lock className="h-3.5 w-3.5" />
            Auth required
          </span>
        ) : null}
      </div>
      <div className="catalog-text-container">
        {headingLink}
        <p className="mb-2 ">
          This is the catalog of <span className="capitalize">{organizationName}</span>, who is also
          part of this dataspace. Click on the catalog name to see the datasets and dataservice they
          offer.
        </p>
      </div>
      <div className="catalog-participant-container flex gap-2 justify-start">
        <div
          className={`rounded-full h-6 aspect-square ${organizationName === "provider" ? "bg-violet-700" : "bg-orange-500"}`}
        />
        <Heading level="h4" className="capitalize">
          {organizationName}
        </Heading>
      </div>
      <div className="catalog-items-container flex justify-end gap-2 text-sm italic">
        <p> 1 Dataservice </p>
        <p> {datasetNumber} Datasets </p>
      </div>
    </div>
  );
};

export default CatalogItem;
