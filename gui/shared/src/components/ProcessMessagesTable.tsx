import React, { useState, useMemo } from "react";
import { DataTable, Column } from "shared/src/components/DataTable";
import { Badge } from "shared/src/components/ui/badge";
import { Button } from "shared/src/components/ui/button";
import { Input } from "shared/src/components/ui/input";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from "shared/src/components/ui/dialog";
import { FormatDate } from "shared/src/components/ui/format-date";
import { cn, formatUrn, formatIdentifier } from "shared/src/lib/utils";
import { toast } from "sonner";
import dayjs from "dayjs";
import {
  MessageSquare,
  MessageSquareText,
  Send,
  Inbox,
  ArrowRight,
  Copy,
  Check,
  Search,
  Eye,
  FileJson,
  Table2,
  MessagesSquare,
  Clock,
  ArrowUpRight,
  ArrowDownLeft,
  ShieldCheck,
  FileText,
  Handshake,
  Play,
  CheckCheck,
  XCircle,
  AlertOctagon,
  ChevronDown,
  ChevronUp,
  X,
  ExternalLink,
} from "lucide-react";

// =============================================================================
// TYPES
// =============================================================================

export interface ProcessMessageItem {
  id?: string;
  negotiationAgentProcessId?: string;
  transferAgentProcessId?: string;
  processId?: string;
  createdAt?: string | Date;
  direction?: string;
  protocol?: string;
  messageType?: string;
  stateTransitionFrom?: string;
  stateTransitionTo?: string;
  payload?: any;
  offer?: any;
  agreement?: any;
  [key: string]: any;
}

export interface ProcessMessagesTableProps {
  messages: ProcessMessageItem[];
  processId?: string;
  className?: string;
  title?: string;
}

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

function normalizeMessage(msg: ProcessMessageItem) {
  return {
    id: msg.id || "",
    processId: msg.negotiationAgentProcessId || msg.transferAgentProcessId || msg.processId || "",
    createdAt: msg.createdAt || new Date().toISOString(),
    direction: msg.direction || "",
    protocol: msg.protocol || "DSP 2024/1",
    messageType: msg.messageType || "UnknownMessage",
    stateTransitionFrom: msg.stateTransitionFrom || "",
    stateTransitionTo: msg.stateTransitionTo || "",
    payload: msg.payload || {},
    offer: msg.offer,
    agreement: msg.agreement,
  };
}

type NormalizedMessage = ReturnType<typeof normalizeMessage>;

function formatMessageType(type: string): { title: string; cleanType: string; icon: React.ComponentType<{ className?: string }> } {
  const clean = type.replace(/^dspace:/i, "").replace(/^dss:/i, "");
  const spaced = clean.replace(/(?!^)([A-Z])/g, " $1").trim();

  const lower = clean.toLowerCase();
  let icon = MessageSquare;

  if (lower.includes("request")) {
    icon = Send;
  } else if (lower.includes("offer")) {
    icon = Handshake;
  } else if (lower.includes("agreement")) {
    icon = FileText;
  } else if (lower.includes("verification")) {
    icon = ShieldCheck;
  } else if (lower.includes("start")) {
    icon = Play;
  } else if (lower.includes("complete") || lower.includes("completion")) {
    icon = CheckCheck;
  } else if (lower.includes("terminate") || lower.includes("termination") || lower.includes("error")) {
    icon = XCircle;
  } else if (lower.includes("suspend") || lower.includes("suspension")) {
    icon = AlertOctagon;
  }

  return { title: spaced, cleanType: clean, icon };
}

function getMessageActor(msg: NormalizedMessage): {
  sender: "Consumer" | "Provider";
  receiver: "Consumer" | "Provider";
  isOutgoing: boolean;
} {
  const from = (msg.stateTransitionFrom || "").toLowerCase();
  const type = (msg.messageType || "").toLowerCase();
  const dir = (msg.direction || "").toLowerCase();

  if (from.includes("consumer") || from.includes("customer")) {
    return { sender: "Consumer", receiver: "Provider", isOutgoing: dir.includes("out") || dir.includes("sent") };
  }
  if (from.includes("provider") || from.includes("business")) {
    return { sender: "Provider", receiver: "Consumer", isOutgoing: dir.includes("out") || dir.includes("sent") };
  }

  if (type.includes("request") || type.includes("verification")) {
    return { sender: "Consumer", receiver: "Provider", isOutgoing: dir.includes("out") || dir.includes("sent") };
  }
  if (type.includes("offer") || type.includes("agreement") || type.includes("start") || type.includes("completion")) {
    return { sender: "Provider", receiver: "Consumer", isOutgoing: dir.includes("out") || dir.includes("sent") };
  }

  return { sender: "Consumer", receiver: "Provider", isOutgoing: true };
}

function formatRelativeTime(dateStr?: string | Date): string {
  if (!dateStr) return "";
  const diffMs = Date.now() - new Date(dateStr).getTime();
  const diffSec = Math.floor(diffMs / 1000);
  if (diffSec < 60) return "Just now";
  const diffMin = Math.floor(diffSec / 60);
  if (diffMin < 60) return `${diffMin}m ago`;
  const diffHours = Math.floor(diffMin / 60);
  if (diffHours < 24) return `${diffHours}h ago`;
  const diffDays = Math.floor(diffHours / 24);
  return `${diffDays}d ago`;
}

function getPayloadSnippet(payload: unknown): string {
  if (!payload || typeof payload !== "object") return "";
  try {
    const keys = Object.keys(payload);
    if (keys.length === 0) return "Empty payload";
    const entries = Object.entries(payload as Record<string, unknown>)
      .filter(([k]) => !k.startsWith("@"))
      .slice(0, 3)
      .map(([k, v]) => `${k}: ${typeof v === "object" ? "{...}" : JSON.stringify(v)}`)
      .join(" • ");
    return entries || JSON.stringify(payload).slice(0, 60);
  } catch {
    return "";
  }
}

// =============================================================================
// SYNTAX HIGHLIGHTED JSON VIEWER
// =============================================================================

function PayloadJsonViewer({ data }: { data: unknown }) {
  const [copied, setCopied] = useState(false);
  const jsonStr = useMemo(() => JSON.stringify(data, null, 2), [data]);
  const lines = useMemo(() => jsonStr.split("\n"), [jsonStr]);

  const handleCopy = () => {
    navigator.clipboard.writeText(jsonStr);
    setCopied(true);
    toast.success("Payload copied to clipboard");
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className="rounded-xl border border-ink/10 bg-sunken/40 overflow-hidden text-xs font-mono shadow-xs">
      <div className="flex items-center justify-between px-3.5 py-2 bg-ink/5 border-b border-ink/10">
        <div className="flex items-center gap-2">
          <FileJson className="h-3.5 w-3.5 text-muted-foreground" />
          <span className="text-muted-foreground/80 font-medium text-[11px]">
            Payload Content ({lines.length} lines)
          </span>
        </div>
        <Button
          variant="ghost"
          size="sm"
          onClick={handleCopy}
          className="h-6 px-2 text-[11px] font-sans gap-1 text-muted-foreground hover:text-foreground"
        >
          {copied ? <Check className="h-3 w-3 text-success-500" /> : <Copy className="h-3 w-3" />}
          {copied ? "Copied" : "Copy"}
        </Button>
      </div>

      <div className="p-3 overflow-x-auto max-h-[400px] overflow-y-auto leading-relaxed">
        <table className="w-full border-collapse">
          <tbody>
            {lines.map((line, idx) => {
              const lineNum = idx + 1;
              const keyValMatch = line.match(/^(\s*)(".*?")(\s*:\s*)(.*)$/);

              return (
                <tr key={idx} className="hover:bg-ink/[0.03]">
                  <td className="select-none text-right pr-3 text-muted-foreground/40 w-8 text-[11px] align-top">
                    {lineNum}
                  </td>
                  <td className="whitespace-pre break-all pl-2 font-mono">
                    {keyValMatch ? (
                      <>
                        <span>{keyValMatch[1]}</span>
                        <span className="text-sky-600 dark:text-sky-300 font-semibold">
                          {keyValMatch[2]}
                        </span>
                        <span className="text-muted-foreground/60">{keyValMatch[3]}</span>
                        <span className="text-foreground/90">{keyValMatch[4]}</span>
                      </>
                    ) : (
                      <span className="text-foreground/80">{line}</span>
                    )}
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>
    </div>
  );
}

// =============================================================================
// MESSAGE INSPECT MODAL (CHAT BUBBLE DETAIL)
// =============================================================================

function MessageInspectModal({
  message,
  index,
  total,
  open,
  onClose,
}: {
  message: NormalizedMessage | null;
  index: number;
  total: number;
  open: boolean;
  onClose: () => void;
}) {
  if (!message) return null;

  const actor = getMessageActor(message);
  const typeMeta = formatMessageType(message.messageType);
  const TypeIcon = typeMeta.icon;

  const isConsumer = actor.sender === "Consumer";

  return (
    <Dialog open={open} onOpenChange={(o) => !o && onClose()}>
      <DialogContent className="sm:max-w-2xl max-h-[85vh] flex flex-col p-6 overflow-hidden">
        <DialogHeader className="space-y-3 pb-3 border-b border-ink/10">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <Badge variant="outline" className="font-mono text-xs">
                #{index + 1} of {total}
              </Badge>
              <Badge variant="secondary" className="text-xs font-mono">
                {message.protocol}
              </Badge>
            </div>
            <span className="text-xs text-muted-foreground flex items-center gap-1.5 font-mono">
              <Clock className="h-3 w-3" />
              {dayjs(message.createdAt).format("DD/MM/YYYY HH:mm:ss")}
            </span>
          </div>

          <div className="flex items-center gap-3">
            <div
              className={cn(
                "w-10 h-10 rounded-full flex items-center justify-center font-bold text-sm border shadow-xs shrink-0",
                isConsumer
                  ? "bg-roles-consumer/15 text-roles-consumer border-roles-consumer/30"
                  : "bg-roles-provider/15 text-roles-provider border-roles-provider/30",
              )}
            >
              {isConsumer ? "C" : "P"}
            </div>
            <div>
              <DialogTitle className="text-base font-semibold text-foreground flex items-center gap-2">
                <TypeIcon className="h-4 w-4 text-primary" />
                {typeMeta.title}
              </DialogTitle>
              <div className="flex items-center gap-2 text-xs text-muted-foreground mt-0.5">
                <span className="font-medium text-foreground">{actor.sender}</span>
                <ArrowRight className="h-3 w-3" />
                <span>{actor.receiver}</span>
              </div>
            </div>
          </div>
        </DialogHeader>

        {/* Modal Body */}
        <div className="flex-1 overflow-y-auto space-y-4 py-3 pr-1">
          {/* Transition Banner if available */}
          {(message.stateTransitionFrom || message.stateTransitionTo) && (
            <div className="p-3 rounded-xl bg-ink/[0.02] border border-ink/10 flex items-center justify-between text-xs">
              <span className="text-muted-foreground font-medium">State Transition:</span>
              <div className="flex items-center gap-2 font-mono">
                {message.stateTransitionFrom ? (
                  <Badge variant="outline">{message.stateTransitionFrom}</Badge>
                ) : (
                  <span className="text-muted-foreground/60">—</span>
                )}
                <ArrowRight className="h-3 w-3 text-muted-foreground" />
                {message.stateTransitionTo ? (
                  <Badge variant="status" state={message.stateTransitionTo}>
                    {message.stateTransitionTo}
                  </Badge>
                ) : (
                  <span className="text-muted-foreground/60">—</span>
                )}
              </div>
            </div>
          )}

          {/* Identifier Metadata */}
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-2 text-xs">
            <div className="p-2.5 rounded-lg bg-ink/[0.02] border border-ink/5">
              <div className="text-[10px] uppercase font-bold text-muted-foreground tracking-wider mb-1">
                Message ID
              </div>
              <div className="flex items-center justify-between gap-1 font-mono text-xs break-all">
                <span className="truncate">{message.id}</span>
                <button
                  type="button"
                  onClick={() => {
                    navigator.clipboard.writeText(message.id);
                    toast.success("Message ID copied");
                  }}
                  className="text-muted-foreground hover:text-foreground p-1"
                  title="Copy ID"
                >
                  <Copy className="h-3 w-3" />
                </button>
              </div>
            </div>

            <div className="p-2.5 rounded-lg bg-ink/[0.02] border border-ink/5">
              <div className="text-[10px] uppercase font-bold text-muted-foreground tracking-wider mb-1">
                Process ID
              </div>
              <div className="flex items-center justify-between gap-1 font-mono text-xs break-all">
                <span className="truncate">{message.processId || "—"}</span>
                {message.processId && (
                  <button
                    type="button"
                    onClick={() => {
                      navigator.clipboard.writeText(message.processId);
                      toast.success("Process ID copied");
                    }}
                    className="text-muted-foreground hover:text-foreground p-1"
                    title="Copy Process ID"
                  >
                    <Copy className="h-3 w-3" />
                  </button>
                )}
              </div>
            </div>
          </div>

          {/* Formatted JSON Payload */}
          <div>
            <PayloadJsonViewer data={message.payload} />
          </div>
        </div>

        <DialogFooter className="pt-2 border-t border-ink/10">
          <Button variant="outline" size="sm" onClick={onClose}>
            Close
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

// =============================================================================
// CHAT STREAM VIEW (CONVERSATION BUBBLES FEED)
// =============================================================================

function ChatStreamView({
  messages,
  onInspect,
}: {
  messages: NormalizedMessage[];
  onInspect: (msg: NormalizedMessage, index: number) => void;
}) {
  const [expandedPayloads, setExpandedPayloads] = useState<Record<string, boolean>>({});

  const togglePayload = (id: string) => {
    setExpandedPayloads((prev) => ({ ...prev, [id]: !prev[id] }));
  };

  return (
    <div className="space-y-4 py-2 px-1 max-h-[620px] overflow-y-auto">
      {messages.map((msg, idx) => {
        const actor = getMessageActor(msg);
        const isConsumer = actor.sender === "Consumer";
        const typeMeta = formatMessageType(msg.messageType);
        const TypeIcon = typeMeta.icon;
        const payloadExpanded = !!expandedPayloads[msg.id];
        const snippet = getPayloadSnippet(msg.payload);

        return (
          <div
            key={msg.id || idx}
            className={cn(
              "flex flex-col w-full transition-all duration-200",
              isConsumer ? "items-end pl-8 sm:pl-16" : "items-start pr-8 sm:pr-16",
            )}
          >
            {/* Sender and Time Mini Header */}
            <div
              className={cn(
                "flex items-center gap-2 mb-1.5 px-1 text-xs text-muted-foreground",
                isConsumer ? "flex-row-reverse" : "flex-row",
              )}
            >
              <div
                className={cn(
                  "w-5 h-5 rounded-full flex items-center justify-center font-bold text-[10px] border shadow-2xs",
                  isConsumer
                    ? "bg-roles-consumer/15 text-roles-consumer border-roles-consumer/30"
                    : "bg-roles-provider/15 text-roles-provider border-roles-provider/30",
                )}
              >
                {isConsumer ? "C" : "P"}
              </div>
              <span className="font-semibold text-foreground text-xs">{actor.sender}</span>
              <span className="text-[11px] font-mono opacity-60">
                {dayjs(msg.createdAt).format("HH:mm:ss")}
              </span>
              <Badge variant="outline" className="text-[9px] px-1 py-0 h-4 font-mono">
                #{idx + 1}
              </Badge>
            </div>

            {/* Chat Bubble Container */}
            <div
              className={cn(
                "w-full max-w-xl rounded-2xl p-4 border shadow-sm transition-all duration-200 space-y-3",
                isConsumer
                  ? "bg-gradient-to-br from-card/90 to-roles-consumer/5 border-roles-consumer/25 rounded-tr-xs"
                  : "bg-gradient-to-br from-card/90 to-roles-provider/5 border-roles-provider/25 rounded-tl-xs",
              )}
            >
              {/* Bubble Header */}
              <div className="flex items-center justify-between gap-2 border-b border-ink/5 pb-2">
                <div className="flex items-center gap-2">
                  <div
                    className={cn(
                      "p-1.5 rounded-lg border",
                      isConsumer
                        ? "bg-roles-consumer/10 text-roles-consumer border-roles-consumer/20"
                        : "bg-roles-provider/10 text-roles-provider border-roles-provider/20",
                    )}
                  >
                    <TypeIcon className="h-4 w-4" />
                  </div>
                  <div>
                    <h4 className="text-sm font-semibold text-foreground leading-tight">
                      {typeMeta.title}
                    </h4>
                    <span className="font-mono text-[10px] text-muted-foreground">
                      {typeMeta.cleanType}
                    </span>
                  </div>
                </div>

                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => onInspect(msg, idx)}
                  className="h-7 text-xs px-2 gap-1 text-muted-foreground hover:text-foreground"
                >
                  <Eye className="h-3 w-3" />
                  Inspect
                </Button>
              </div>

              {/* State Transition if present */}
              {(msg.stateTransitionFrom || msg.stateTransitionTo) && (
                <div className="flex items-center gap-2 text-xs font-mono pt-0.5">
                  <span className="text-[11px] text-muted-foreground font-sans">Transition:</span>
                  {msg.stateTransitionFrom && (
                    <Badge variant="outline" className="text-[10px]">
                      {msg.stateTransitionFrom}
                    </Badge>
                  )}
                  <ArrowRight className="h-3 w-3 text-muted-foreground" />
                  {msg.stateTransitionTo && (
                    <Badge variant="status" state={msg.stateTransitionTo} className="text-[10px]">
                      {msg.stateTransitionTo}
                    </Badge>
                  )}
                </div>
              )}

              {/* Payload Preview or Collapsible Content */}
              {snippet && (
                <div className="text-xs text-muted-foreground font-mono bg-ink/[0.03] p-2 rounded-lg border border-ink/5 truncate">
                  {snippet}
                </div>
              )}

              {/* Toggle Full Payload Accordion inside Bubble */}
              <div className="pt-1 border-t border-ink/5 flex items-center justify-between">
                <button
                  type="button"
                  onClick={() => togglePayload(msg.id)}
                  className="text-xs text-primary hover:underline inline-flex items-center gap-1 font-medium"
                >
                  {payloadExpanded ? (
                    <>
                      <ChevronUp className="h-3 w-3" />
                      Hide JSON payload
                    </>
                  ) : (
                    <>
                      <ChevronDown className="h-3 w-3" />
                      View JSON payload
                    </>
                  )}
                </button>

                <button
                  type="button"
                  onClick={() => {
                    navigator.clipboard.writeText(JSON.stringify(msg.payload, null, 2));
                    toast.success("Payload copied");
                  }}
                  className="text-muted-foreground hover:text-foreground text-xs p-1"
                  title="Copy payload"
                >
                  <Copy className="h-3 w-3" />
                </button>
              </div>

              {payloadExpanded && (
                <div className="pt-2">
                  <PayloadJsonViewer data={msg.payload} />
                </div>
              )}
            </div>
          </div>
        );
      })}
    </div>
  );
}

// =============================================================================
// MAIN COMPONENT: ProcessMessagesTable
// =============================================================================

export function ProcessMessagesTable({
  messages,
  processId,
  className,
  title = "Exchange Messages",
}: ProcessMessagesTableProps) {
  const [searchQuery, setSearchQuery] = useState("");
  const [selectedActor, setSelectedActor] = useState<"all" | "Consumer" | "Provider">("all");
  const [viewMode, setViewMode] = useState<"table" | "chat">("table");
  const [sortOrder, setSortOrder] = useState<"asc" | "desc">("asc");
  const [inspectingMessage, setInspectingMessage] = useState<{
    message: NormalizedMessage;
    index: number;
  } | null>(null);

  // Normalize all messages
  const normalizedList = useMemo(() => {
    return (messages || []).map(normalizeMessage);
  }, [messages]);

  // Filter and sort messages
  const filteredMessages = useMemo(() => {
    let list = [...normalizedList];

    // Filter by Actor
    if (selectedActor !== "all") {
      list = list.filter((m) => getMessageActor(m).sender === selectedActor);
    }

    // Filter by Search Query
    if (searchQuery.trim()) {
      const q = searchQuery.toLowerCase();
      list = list.filter((m) => {
        const matchType = m.messageType.toLowerCase().includes(q);
        const matchFrom = m.stateTransitionFrom.toLowerCase().includes(q);
        const matchTo = m.stateTransitionTo.toLowerCase().includes(q);
        const matchId = m.id.toLowerCase().includes(q);
        const matchPayload = JSON.stringify(m.payload).toLowerCase().includes(q);
        return matchType || matchFrom || matchTo || matchId || matchPayload;
      });
    }

    // Sort by Chronological order
    list.sort((a, b) => {
      const timeA = new Date(a.createdAt).getTime();
      const timeB = new Date(b.createdAt).getTime();
      return sortOrder === "asc" ? timeA - timeB : timeB - timeA;
    });

    return list;
  }, [normalizedList, selectedActor, searchQuery, sortOrder]);

  // Table Columns Definition
  const columns: Column<NormalizedMessage>[] = useMemo(
    () => [
      {
        header: "#",
        className: "w-12 text-center",
        cell: (item) => {
          const origIdx = normalizedList.findIndex((m) => m.id === item.id);
          return (
            <Badge variant="outline" className="font-mono text-[10px] px-1.5 py-0 h-5">
              #{origIdx !== -1 ? origIdx + 1 : "•"}
            </Badge>
          );
        },
      },
      {
        header: "Direction / Actor",
        className: "min-w-[170px]",
        cell: (item) => {
          const actor = getMessageActor(item);
          const isConsumer = actor.sender === "Consumer";
          return (
            <div className="flex items-center gap-2">
              <div
                className={cn(
                  "w-7 h-7 rounded-full flex items-center justify-center font-bold text-xs border shrink-0 shadow-2xs",
                  isConsumer
                    ? "bg-roles-consumer/15 text-roles-consumer border-roles-consumer/30"
                    : "bg-roles-provider/15 text-roles-provider border-roles-provider/30",
                )}
              >
                {isConsumer ? "C" : "P"}
              </div>
              <div className="flex flex-col">
                <div className="flex items-center gap-1">
                  <Badge
                    variant="role"
                    dsrole={actor.sender}
                    className="text-[10px] px-1.5 py-0 leading-tight"
                  >
                    {actor.sender}
                  </Badge>
                  <ArrowRight className="h-3 w-3 text-muted-foreground/60" />
                  <span className="text-[11px] text-muted-foreground font-medium">
                    {actor.receiver}
                  </span>
                </div>
                <span className="text-[10px] text-muted-foreground/70 font-mono mt-0.5">
                  {actor.isOutgoing ? "Outgoing ↗" : "Incoming ↙"}
                </span>
              </div>
            </div>
          );
        },
      },
      {
        header: "Message",
        className: "min-w-[280px]",
        cell: (item) => {
          const typeMeta = formatMessageType(item.messageType);
          const TypeIcon = typeMeta.icon;
          const snippet = getPayloadSnippet(item.payload);

          return (
            <div className="flex flex-col gap-1 py-1">
              <div className="flex items-center gap-1.5 flex-wrap">
                <TypeIcon className="h-3.5 w-3.5 text-primary shrink-0" />
                <span className="font-semibold text-foreground text-xs">{typeMeta.title}</span>
                <Badge variant="code" className="text-[9px] px-1 py-0 border-ink/10">
                  {typeMeta.cleanType}
                </Badge>
              </div>
              {snippet && (
                <span className="text-xs font-mono text-muted-foreground/80 truncate max-w-[340px]">
                  {snippet}
                </span>
              )}
            </div>
          );
        },
      },
      {
        header: "State Transition",
        className: "min-w-[180px]",
        cell: (item) => {
          if (!item.stateTransitionFrom && !item.stateTransitionTo) {
            return <span className="text-muted-foreground/50 text-xs font-mono">—</span>;
          }

          return (
            <div className="flex items-center gap-1.5 font-mono text-xs">
              {item.stateTransitionFrom ? (
                <Badge variant="outline" className="text-[10px]">
                  {item.stateTransitionFrom}
                </Badge>
              ) : (
                <span className="text-muted-foreground/50">—</span>
              )}
              <ArrowRight className="h-3 w-3 text-muted-foreground/60" />
              {item.stateTransitionTo ? (
                <Badge variant="status" state={item.stateTransitionTo} className="text-[10px]">
                  {item.stateTransitionTo}
                </Badge>
              ) : (
                <span className="text-muted-foreground/50">—</span>
              )}
            </div>
          );
        },
      },
      {
        header: "Time",
        className: "min-w-[140px]",
        sortable: true,
        sortValue: (item) => new Date(item.createdAt).getTime(),
        cell: (item) => (
          <div className="flex flex-col text-xs font-mono">
            <span className="text-foreground font-medium">
              {dayjs(item.createdAt).format("HH:mm:ss")}
            </span>
            <span className="text-[10px] text-muted-foreground/70">
              {formatRelativeTime(item.createdAt)}
            </span>
          </div>
        ),
      },
      {
        header: "Actions",
        className: "w-24 text-right",
        sortable: false,
        searchable: false,
        cell: (item) => {
          const origIdx = normalizedList.findIndex((m) => m.id === item.id);
          return (
            <div className="flex items-center justify-end gap-1">
              <Button
                variant="ghost"
                size="icon_sm"
                onClick={(e) => {
                  e.stopPropagation();
                  setInspectingMessage({ message: item, index: origIdx !== -1 ? origIdx : 0 });
                }}
                className="h-7 w-7 text-muted-foreground hover:text-foreground"
                title="Inspect message payload"
              >
                <Eye className="h-3.5 w-3.5" />
              </Button>
              <Button
                variant="ghost"
                size="icon_sm"
                onClick={(e) => {
                  e.stopPropagation();
                  navigator.clipboard.writeText(JSON.stringify(item.payload, null, 2));
                  toast.success("Payload JSON copied");
                }}
                className="h-7 w-7 text-muted-foreground hover:text-foreground"
                title="Copy payload"
              >
                <Copy className="h-3.5 w-3.5" />
              </Button>
            </div>
          );
        },
      },
    ],
    [normalizedList],
  );

  return (
    <div className={cn("w-full space-y-4", className)}>
      {/* Table Toolbar / Controls */}
      <div className="flex flex-col sm:flex-row items-stretch sm:items-center justify-between gap-3 p-3.5 rounded-2xl border border-ink/10 bg-card/60 backdrop-blur-sm shadow-xs">
        <div className="flex items-center gap-2.5 flex-wrap">
          <div className="flex items-center gap-2">
            <MessageSquareText className="h-4 w-4 text-primary" />
            <h4 className="text-sm font-semibold text-foreground tracking-tight">{title}</h4>
            <Badge variant="outline" className="text-xs font-mono">
              {normalizedList.length} {normalizedList.length === 1 ? "message" : "messages"}
            </Badge>
          </div>

          {/* Actor filter buttons */}
          <div className="inline-flex rounded-lg border border-ink/10 p-0.5 bg-sunken/40">
            <button
              type="button"
              onClick={() => setSelectedActor("all")}
              className={cn(
                "px-2.5 py-1 rounded-md text-xs font-medium transition-all",
                selectedActor === "all"
                  ? "bg-card text-foreground shadow-2xs font-semibold"
                  : "text-muted-foreground hover:text-foreground",
              )}
            >
              All
            </button>
            <button
              type="button"
              onClick={() => setSelectedActor("Consumer")}
              className={cn(
                "px-2.5 py-1 rounded-md text-xs font-medium transition-all",
                selectedActor === "Consumer"
                  ? "bg-card text-roles-consumer shadow-2xs font-semibold"
                  : "text-muted-foreground hover:text-foreground",
              )}
            >
              Consumer
            </button>
            <button
              type="button"
              onClick={() => setSelectedActor("Provider")}
              className={cn(
                "px-2.5 py-1 rounded-md text-xs font-medium transition-all",
                selectedActor === "Provider"
                  ? "bg-card text-roles-provider shadow-2xs font-semibold"
                  : "text-muted-foreground hover:text-foreground",
              )}
            >
              Provider
            </button>
          </div>
        </div>

        {/* Right side controls: Search, Order, View Toggle */}
        <div className="flex items-center gap-2 flex-wrap self-end sm:self-auto">
          {/* View mode toggle */}
          <div className="inline-flex rounded-xl border border-ink/10 p-0.5 bg-sunken/40">
            <button
              type="button"
              onClick={() => setViewMode("table")}
              className={cn(
                "px-2.5 py-1 rounded-lg text-xs font-medium inline-flex items-center gap-1.5 transition-all",
                viewMode === "table"
                  ? "bg-card text-foreground shadow-2xs font-semibold"
                  : "text-muted-foreground hover:text-foreground",
              )}
              title="Table view"
            >
              <Table2 className="h-3.5 w-3.5" />
              Table
            </button>
            <button
              type="button"
              onClick={() => setViewMode("chat")}
              className={cn(
                "px-2.5 py-1 rounded-lg text-xs font-medium inline-flex items-center gap-1.5 transition-all",
                viewMode === "chat"
                  ? "bg-card text-foreground shadow-2xs font-semibold"
                  : "text-muted-foreground hover:text-foreground",
              )}
              title="Chat stream view"
            >
              <MessagesSquare className="h-3.5 w-3.5" />
              Chat Feed
            </button>
          </div>

          {/* Chronological order toggle */}
          <Button
            variant="outline"
            size="sm"
            onClick={() => setSortOrder((v) => (v === "asc" ? "desc" : "asc"))}
            className="text-xs h-7 px-2 font-mono gap-1 rounded-lg border-ink/10 text-muted-foreground"
            title={sortOrder === "asc" ? "Showing oldest first" : "Showing newest first"}
          >
            <Clock className="h-3 w-3" />
            {sortOrder === "asc" ? "Oldest ➔ Newest" : "Newest ➔ Oldest"}
          </Button>
        </div>
      </div>

      {/* Main Content: Table View vs Chat Feed View */}
      {normalizedList.length === 0 ? (
        <div className="p-12 text-center rounded-2xl border border-dashed border-ink/15 bg-card/40 space-y-3">
          <div className="w-10 h-10 rounded-full bg-ink/5 flex items-center justify-center mx-auto text-muted-foreground">
            <MessageSquare className="h-5 w-5" />
          </div>
          <div className="space-y-1">
            <h4 className="text-sm font-semibold text-foreground">No exchange messages recorded</h4>
            <p className="text-xs text-muted-foreground max-w-sm mx-auto">
              Protocol messages exchanged between Consumer and Provider will be logged here in chronological order.
            </p>
          </div>
        </div>
      ) : viewMode === "chat" ? (
        <ChatStreamView
          messages={filteredMessages}
          onInspect={(msg, idx) => setInspectingMessage({ message: msg, index: idx })}
        />
      ) : (
        <DataTable
          data={filteredMessages}
          columns={columns}
          keyExtractor={(item) => item.id}
          searchPlaceholder="Search messages by type, sender, transition, or payload..."
          emptyMessage="No messages matching the filter criteria"
          onRowClick={(item) => {
            const idx = normalizedList.findIndex((m) => m.id === item.id);
            setInspectingMessage({ message: item, index: idx !== -1 ? idx : 0 });
          }}
          className="text-sm rounded-2xl overflow-hidden border border-ink/10 bg-card/60 backdrop-blur-sm"
        />
      )}

      {/* Message Inspection Dialog */}
      <MessageInspectModal
        message={inspectingMessage?.message || null}
        index={inspectingMessage?.index || 0}
        total={normalizedList.length}
        open={!!inspectingMessage}
        onClose={() => setInspectingMessage(null)}
      />
    </div>
  );
}

export default ProcessMessagesTable;
