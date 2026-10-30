/**
 * ProcessMessagesTable.tsx
 *
 * The Dataspace Protocol messages of a negotiation or transfer, oldest first, with a dialog per
 * message showing its payload as a table or highlighted JSON.
 */

import React, { useMemo, useState } from "react";
import {
  AlertOctagon,
  ArrowDownLeft,
  ArrowRight,
  ArrowUpRight,
  CheckCheck,
  FileText,
  Handshake,
  MessageSquare,
  Play,
  Send,
  ShieldCheck,
  XCircle,
} from "lucide-react";
import { cn } from "@eunomia/ui";
import { Badge } from "@eunomia/ui";
import { RelativeTime } from "@eunomia/ui";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@eunomia/ui";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@eunomia/ui";
import { FieldGrid } from "@eunomia/ui";
import { JsonTable } from "@eunomia/ui";
import { Identifier } from "@eunomia/ui";
import dayjs from "dayjs";

export interface ProcessMessageItem {
  id?: string;
  occurredAt?: string | Date;
  createdAt?: string | Date;
  direction?: string;
  messageType?: string;
  stateTransitionFrom?: string;
  stateTransitionTo?: string;
  /** Negotiation messages carry the DSP message here. */
  payload?: unknown;
  /** Transfer messages carry it inside their envelope, with its canonical form. */
  envelope?: { payload?: unknown; canonicalForm?: string | null; canonicalHash?: string | null };
  [key: string]: any;
}

export interface ProcessMessagesTableProps {
  messages: ProcessMessageItem[];
  /** Role of this connector in the process, to name who sent each message. */
  role?: "Provider" | "Consumer" | string;
  className?: string;
  emptyMessage?: string;
  /** Kept for callers of the former table; unused. */
  processId?: string;
  title?: string;
}

interface Row {
  key: string;
  step: number;
  id?: string;
  at: string;
  outbound: boolean;
  type: string;
  label: string;
  Icon: React.ComponentType<{ className?: string }>;
  from: string;
  to: string;
  payload: unknown;
  canonicalForm?: string;
  canonicalHash?: string;
}

const ICONS: [string, React.ComponentType<{ className?: string }>][] = [
  ["termination", XCircle],
  ["error", XCircle],
  ["suspension", AlertOctagon],
  ["completion", CheckCheck],
  ["start", Play],
  ["verification", ShieldCheck],
  ["agreement", FileText],
  ["offer", Handshake],
  ["request", Send],
];

export function ProcessMessagesTable({
  messages,
  role,
  className,
  emptyMessage = "No protocol messages yet.",
}: ProcessMessagesTableProps) {
  const [selected, setSelected] = useState<Row | null>(null);
  const rows = useMemo(() => ProcessMessagesTable.rows(messages), [messages]);
  const peer = role === "Provider" ? "Consumer" : role === "Consumer" ? "Provider" : "Peer";

  if (rows.length === 0) {
    return (
      <div
        className={cn(
          "rounded-lg border border-dashed border-ink/15 py-10 text-center text-sm text-ink/50",
          className,
        )}
      >
        {emptyMessage}
      </div>
    );
  }

  return (
    <>
      <div className={cn("overflow-hidden rounded-lg border border-ink/10", className)}>
        <table className="w-full text-sm">
          <thead className="bg-ink/[0.03] text-left text-xs font-medium text-ink/55">
            <tr>
              <th className="w-10 px-3 py-2.5 font-medium">#</th>
              <th className="px-3 py-2.5 font-medium">Message</th>
              <th className="px-3 py-2.5 font-medium">Sent by</th>
              <th className="px-3 py-2.5 font-medium">State change</th>
              <th className="px-3 py-2.5 text-right font-medium">When</th>
            </tr>
          </thead>
          <tbody>
            {rows.map((r) => (
              <tr
                key={r.key}
                onClick={() => setSelected(r)}
                className="cursor-pointer border-t border-ink/[0.07] transition-colors hover:bg-ink/[0.03]"
              >
                <td className="px-3 py-3 tabular-nums text-ink/40">{r.step}</td>
                <td className="px-3 py-3">
                  <span className="flex items-center gap-2.5">
                    <span className="inline-flex h-7 w-7 shrink-0 items-center justify-center rounded-md bg-ink/[0.06] text-ink/70">
                      <r.Icon className="h-3.5 w-3.5" />
                    </span>
                    <span className="font-medium">{r.label}</span>
                  </span>
                </td>
                <td className="px-3 py-3">
                  <ProcessMessagesTable.Sender outbound={r.outbound} role={role} peer={peer} />
                </td>
                <td className="px-3 py-3">
                  <ProcessMessagesTable.Transition from={r.from} to={r.to} />
                </td>
                <td className="px-3 py-3 text-right text-ink/60">
                  <RelativeTime date={r.at} />
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      <Dialog open={!!selected} onOpenChange={(o) => !o && setSelected(null)}>
        <DialogContent className="max-h-[90vh] max-w-3xl overflow-y-auto">
          {selected && (
            <>
              <DialogHeader>
                <DialogTitle className="flex items-center gap-2">
                  <selected.Icon className="h-4 w-4 text-ink/60" />
                  {selected.label}
                </DialogTitle>
                <DialogDescription className="font-mono text-xs">{selected.type}</DialogDescription>
              </DialogHeader>
              <FieldGrid
                columns={2}
                fields={[
                  {
                    label: "Sent by",
                    value: (
                      <ProcessMessagesTable.Sender
                        outbound={selected.outbound}
                        role={role}
                        peer={peer}
                      />
                    ),
                  },
                  {
                    label: "State change",
                    value: (
                      <ProcessMessagesTable.Transition from={selected.from} to={selected.to} />
                    ),
                  },
                  { label: "Occurred", value: dayjs(selected.at).format("YYYY-MM-DD HH:mm:ss") },
                  { label: "Message", value: <Identifier value={selected.id} /> },
                ]}
              />
              <Tabs defaultValue="message" className="mt-2">
                <TabsList>
                  <TabsTrigger value="message">DSP message</TabsTrigger>
                  {selected.canonicalForm && (
                    <TabsTrigger value="canonical">Canonical form</TabsTrigger>
                  )}
                </TabsList>
                <TabsContent value="message" className="pt-3">
                  <JsonTable
                    value={selected.payload}
                    expandDepth={2}
                    emptyText="The message has no body."
                  />
                </TabsContent>
                {selected.canonicalForm && (
                  <TabsContent value="canonical" className="space-y-2 pt-3">
                    <pre className="overflow-x-auto rounded-lg border border-ink/10 bg-sunken p-3 font-mono text-xs leading-relaxed text-ink/80">
                      {selected.canonicalForm}
                    </pre>
                    {selected.canonicalHash && (
                      <p className="text-xs text-ink/50">
                        SHA-256 <span className="font-mono">{selected.canonicalHash}</span>
                      </p>
                    )}
                  </TabsContent>
                )}
              </Tabs>
            </>
          )}
        </DialogContent>
      </Dialog>
    </>
  );
}

/** Messages as rows, oldest first, so the table reads as the conversation happened. */
ProcessMessagesTable.rows = (messages: ProcessMessageItem[]): Row[] =>
  [...messages]
    .map((m, i) => {
      const type = (m.messageType ?? "Message").replace(/^dspace:/i, "");
      const payload = m.payload ?? m.envelope?.payload ?? {};
      return {
        key: m.id ?? String(i),
        step: 0,
        id: m.id,
        at: String(m.occurredAt ?? m.createdAt ?? ""),
        outbound: (m.direction ?? "").toLowerCase() === "outbound",
        type,
        label: ProcessMessagesTable.label(type, payload),
        Icon: ICONS.find(([k]) => type.toLowerCase().includes(k))?.[1] ?? MessageSquare,
        from: m.stateTransitionFrom ?? "",
        to: m.stateTransitionTo ?? "",
        payload,
        canonicalForm: m.envelope?.canonicalForm ?? undefined,
        canonicalHash: m.envelope?.canonicalHash ?? undefined,
      };
    })
    .sort((a, b) => new Date(a.at).getTime() - new Date(b.at).getTime())
    .map((r, i) => ({ ...r, step: i + 1 }));

/** `TransferRequestMessage` → `Transfer request`; negotiation events name their event. */
ProcessMessagesTable.label = (type: string, payload: unknown): string => {
  const words = type
    .replace(/Message$/, "")
    .replace(/(?!^)([A-Z])/g, " $1")
    .toLowerCase();
  const event = (payload as Record<string, unknown>)?.eventType;
  const base = words.charAt(0).toUpperCase() + words.slice(1);
  return typeof event === "string"
    ? `${base}: ${event.replace(/^.*[:/]/, "").toLowerCase()}`
    : base;
};

ProcessMessagesTable.Sender = function Sender({
  outbound,
  role,
  peer,
}: {
  outbound: boolean;
  role?: string;
  peer: string;
}) {
  return (
    <span className="inline-flex items-center gap-1.5">
      {outbound ? (
        <ArrowUpRight className="h-3.5 w-3.5 text-brand-sky" />
      ) : (
        <ArrowDownLeft className="h-3.5 w-3.5 text-ink/50" />
      )}
      {outbound ? (
        <span>You{role && <span className="text-ink/50"> · {role}</span>}</span>
      ) : (
        <span>{peer}</span>
      )}
    </span>
  );
};

ProcessMessagesTable.Transition = function Transition({ from, to }: { from: string; to: string }) {
  if (!from && !to) return <span className="text-ink/35">—</span>;
  const initial = !from || from === "INITIAL";
  return (
    <span className="inline-flex items-center gap-1.5">
      <span className={cn("text-xs", initial ? "italic text-ink/45" : "text-ink/70")}>
        {initial ? "Initial" : from}
      </span>
      <ArrowRight className="h-3 w-3 text-ink/35" />
      <Badge variant="status" state={to} size="sm">
        {to}
      </Badge>
    </span>
  );
};

export default ProcessMessagesTable;
