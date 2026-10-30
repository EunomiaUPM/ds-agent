import React, { createContext, useContext } from "react";
import { cn } from "../lib/utils";
import { Identifier } from "./Identifier";

export interface ParticipantDirectoryInfo {
  nick?: string | null;
  isSelf?: boolean;
}

export type ParticipantResolverFn = (id: string) => ParticipantDirectoryInfo | undefined;

const ParticipantDirectoryContext = createContext<ParticipantResolverFn>(() => undefined);

export const ParticipantDirectoryProvider = ParticipantDirectoryContext.Provider;

export const useParticipantResolver = () => useContext(ParticipantDirectoryContext);

export interface ParticipantLabelProps {
  id?: string | null;
  nick?: string | null;
  isSelf?: boolean;
  className?: string;
  /** Hides the identifier line under the name. */
  compact?: boolean;
}

/** A participant by its nickname, marked when it is this agent, with its DID underneath. */
export function ParticipantLabel({
  id,
  nick: explicitNick,
  isSelf: explicitIsSelf,
  className,
  compact = false,
}: ParticipantLabelProps) {
  const resolveParticipant = useParticipantResolver();
  if (!id) return <span className="text-ink/35">—</span>;

  const resolved = resolveParticipant(id);
  const nick = explicitNick ?? resolved?.nick;
  const self = explicitIsSelf ?? resolved?.isSelf ?? false;

  return (
    <span className={cn("inline-flex min-w-0 flex-col", className)}>
      <span className="inline-flex items-center gap-1.5">
        <span className={cn("truncate font-medium", !nick && "text-ink/60")}>
          {nick ?? "Unknown participant"}
        </span>
        {self && (
          <span className="rounded bg-brand-sky/15 px-1.5 py-px text-[0.6875rem] font-semibold text-brand-sky">
            You
          </span>
        )}
      </span>
      {!compact && <Identifier value={id} />}
    </span>
  );
}
