import React from "react";
import { cn } from "shared/src/lib/utils";
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "./tooltip";

/** Longest DID shown whole: every did:web of the mini, standalone and production fits. */
const MAX_DID_LENGTH = 36;
/** Characters kept from the end of a cut DID: did:jwk ones share their start. */
const DID_TAIL = 8;

/** `did` cut in the middle past {@link MAX_DID_LENGTH} characters. */
export const shortenDid = (did: string) =>
  did.length <= MAX_DID_LENGTH
    ? did
    : `${did.slice(0, MAX_DID_LENGTH - DID_TAIL - 1)}…${did.slice(-DID_TAIL)}`;

/** A DID on one line, cut when too long, with the whole DID on hover. */
export const FormatDid = ({ did, className }: { did: string; className?: string }) => {
  const short = shortenDid(did);
  const text = (
    <span className={cn("inline-block max-w-full truncate align-bottom font-mono", className)}>
      {short}
    </span>
  );
  if (short === did) return text;
  return (
    <TooltipProvider delayDuration={150}>
      <Tooltip>
        <TooltipTrigger asChild>{text}</TooltipTrigger>
        {/* Above the popovers (z-[100]) the DID may be shown in. */}
        <TooltipContent className="z-[110] max-w-md font-mono break-all">{did}</TooltipContent>
      </Tooltip>
    </TooltipProvider>
  );
};
