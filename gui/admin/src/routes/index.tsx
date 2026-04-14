import React from "react";
import { createFileRoute } from "@tanstack/react-router";
import {
  ArrowLeftRight,
  BookOpen,
  FileText,
  HandshakeIcon,
  ShieldCheck,
  Users,
} from "lucide-react";

const features = [
  {
    icon: BookOpen,
    title: "Catalog",
    description:
      "Browse and manage data offerings published by participants. Explore datasets, their policies, and available distributions.",
  },
  {
    icon: HandshakeIcon,
    title: "Contract Negotiation",
    description:
      "Manage the full lifecycle of contract negotiations between providers and consumers, following the Dataspace Protocol.",
  },
  {
    icon: FileText,
    title: "Agreements",
    description:
      "Inspect finalized contracts. Each agreement records the agreed policy and the identities of both parties.",
  },
  {
    icon: ArrowLeftRight,
    title: "Transfer Processes",
    description:
      "Monitor and control active data transfers end-to-end, including control-plane state and dataplane activity.",
  },
  {
    icon: Users,
    title: "Participants",
    description:
      "Manage dataspace participants — organizations that are registered and authorized to interact within the ecosystem.",
  },
  {
    icon: ShieldCheck,
    title: "SSI Auth",
    description:
      "Self-Sovereign Identity layer. Issue credentials, manage wallets, and control access through verifiable identity.",
  },

];

const Index = () => {
  return (
    <div className="flex flex-1 flex-col gap-4 p-8 pt-6 overflow-hidden items-start justify-start w-full h-full">
      <div className="p-6 max-w-3xl">
        <div className="mb-8">
          <h1 className="text-3xl font-bold tracking-tight mb-2">
            Welcome to Eunomia DS-Agent
          </h1>
          <p className="text-muted-foreground text-base leading-relaxed">
            Eunomia DS-Agent is a <strong>Dataspace Protocol</strong> administration
            console. It gives operators full visibility and control over the{" "}
            <strong>catalog</strong>, <strong>contract negotiations</strong>,{" "}
            <strong>agreements</strong>, and <strong>data transfers</strong>{" "}
            that flow through the dataspace — alongside identity management
            powered by <strong>Self-Sovereign Identity (SSI)</strong>.
          </p>
        </div>

        <h2 className="text-lg font-semibold mb-4 text-foreground/80">
          What you can do
        </h2>

        <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
          {features.map(({ icon: Icon, title, description }) => (
            <div
              key={title}
              className="rounded-xl border border-white/10 bg-white/5 p-4 flex gap-3 items-start hover:bg-white/10 transition-colors"
            >
              <div className="mt-0.5 shrink-0 text-primary">
                <Icon className="h-5 w-5" />
              </div>
              <div>
                <p className="font-semibold text-sm mb-1">{title}</p>
                <p className="text-xs text-muted-foreground leading-relaxed">
                  {description}
                </p>
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};

export const Route = createFileRoute("/")({
  component: Index,
});

export default Index;
