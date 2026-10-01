import React, { useState } from "react";
import { formatUrn } from "shared/src/lib/utils";
import { NegotiationMessageDto } from "../data/orval/model";
import { Badge } from "shared/src/components/ui/badge";
import { Button } from "shared/src/components/ui/button";
import dayjs from "dayjs";
import {
  Send,
  Handshake,
  FileText,
  ShieldCheck,
  Play,
  CheckCheck,
  XCircle,
  Copy,
  ChevronDown,
  ChevronUp,
  Clock,
  ArrowRight,
} from "lucide-react";
import { toast } from "sonner";
import { cn } from "shared/src/lib/utils";

export { ProcessMessagesTable } from "./ProcessMessagesTable";

type MessageComponentProps = {
  message: NegotiationMessageDto;
};

const formatSpaces = (text: string) => {
  if (!text) return "";
  return text.replace(/^dspace:/i, "").replace(/(?!^)([A-Z])/g, " $1").trim();
};

/**
 * Component for displaying a single Contract Negotiation process message with chat bubble design.
 */
const MessageComponent: React.FC<MessageComponentProps> = ({ message }) => {
  const [showPayload, setShowPayload] = useState(false);
  const from = (message.stateTransitionFrom || "").toLowerCase();
  const isConsumer = from.includes("consumer") || from.includes("customer");
  const actor = isConsumer ? "Consumer" : "Provider";
  const receiver = isConsumer ? "Provider" : "Consumer";

  return (
    <div
      className={cn(
        "flex flex-col w-full my-2 transition-all",
        isConsumer ? "items-end pl-6 sm:pl-12" : "items-start pr-6 sm:pr-12",
      )}
    >
      {/* Sender Mini Header */}
      <div
        className={cn(
          "flex items-center gap-2 mb-1 px-1 text-xs text-muted-foreground",
          isConsumer ? "flex-row-reverse" : "flex-row",
        )}
      >
        <div
          className={cn(
            "w-5 h-5 rounded-full flex items-center justify-center font-bold text-[10px] border",
            isConsumer
              ? "bg-roles-consumer/15 text-roles-consumer border-roles-consumer/30"
              : "bg-roles-provider/15 text-roles-provider border-roles-provider/30",
          )}
        >
          {isConsumer ? "C" : "P"}
        </div>
        <span className="font-semibold text-foreground">{actor}</span>
        <span className="text-[11px] font-mono opacity-60">
          {dayjs(message.createdAt).format("HH:mm:ss")}
        </span>
      </div>

      {/* Bubble Container */}
      <div
        className={cn(
          "w-full max-w-xl rounded-2xl p-4 border shadow-xs space-y-2.5",
          isConsumer
            ? "bg-gradient-to-br from-card/95 to-roles-consumer/5 border-roles-consumer/25 rounded-tr-xs"
            : "bg-gradient-to-br from-card/95 to-roles-provider/5 border-roles-provider/25 rounded-tl-xs",
        )}
      >
        <div className="flex items-center justify-between gap-2 border-b border-ink/5 pb-2">
          <div>
            <h4 className="text-sm font-semibold text-foreground">
              {formatSpaces(message.messageType)}
            </h4>
            <div className="flex items-center gap-2 text-[11px] text-muted-foreground">
              <span className="font-mono">{message.messageType}</span>
              <span>•</span>
              <span className="font-mono">DSP 2024/1</span>
            </div>
          </div>
          <Badge
            variant="role"
            dsrole={actor}
            className="text-[10px] px-1.5 py-0 uppercase"
          >
            {actor}
          </Badge>
        </div>

        {/* Transition if present */}
        {(message.stateTransitionFrom || message.stateTransitionTo) && (
          <div className="flex items-center gap-2 text-xs font-mono">
            <span className="text-[11px] text-muted-foreground font-sans">Transition:</span>
            {message.stateTransitionFrom && (
              <Badge variant="outline" className="text-[10px]">
                {message.stateTransitionFrom}
              </Badge>
            )}
            <ArrowRight className="h-3 w-3 text-muted-foreground" />
            {message.stateTransitionTo && (
              <Badge variant="status" state={message.stateTransitionTo} className="text-[10px]">
                {message.stateTransitionTo}
              </Badge>
            )}
          </div>
        )}

        {/* Meta IDs */}
        <div className="flex flex-wrap gap-2 text-[11px] text-muted-foreground font-mono">
          <span>ID: {formatUrn(message.id, true)}</span>
          {message.negotiationAgentProcessId && (
            <span>• Process: {formatUrn(message.negotiationAgentProcessId, true)}</span>
          )}
        </div>

        {/* Toggle Payload */}
        <div className="pt-1 border-t border-ink/5 flex items-center justify-between">
          <button
            type="button"
            onClick={() => setShowPayload((v) => !v)}
            className="text-xs text-primary hover:underline inline-flex items-center gap-1 font-medium"
          >
            {showPayload ? (
              <>
                <ChevronUp className="h-3 w-3" />
                Hide Payload
              </>
            ) : (
              <>
                <ChevronDown className="h-3 w-3" />
                View Payload
              </>
            )}
          </button>
          <button
            type="button"
            onClick={() => {
              navigator.clipboard.writeText(JSON.stringify(message.payload, null, 2));
              toast.success("Payload copied");
            }}
            className="text-muted-foreground hover:text-foreground text-xs p-1"
            title="Copy payload"
          >
            <Copy className="h-3 w-3" />
          </button>
        </div>

        {showPayload && (
          <pre className="p-3 rounded-xl bg-sunken/60 text-xs font-mono overflow-x-auto whitespace-pre-wrap break-all max-h-64 overflow-y-auto border border-ink/10">
            {JSON.stringify(message.payload, null, 2)}
          </pre>
        )}
      </div>
    </div>
  );
};

export default MessageComponent;

