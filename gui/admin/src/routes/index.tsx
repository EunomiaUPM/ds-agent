/*
 * Copyright (C) 2026 - Universidad Politécnica de Madrid - UPM
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program. If not, see <https://www.gnu.org/licenses/>.
 */

import { createFileRoute, Link } from "@tanstack/react-router";
import {
  ArrowLeftRight,
  ArrowRight,
  BellDot,
  BookOpen,
  FileSignature,
  Fingerprint,
  Handshake,
  Radio,
  ShieldCheck,
} from "lucide-react";
import { PageLayout } from "@eunomia/ui";
import { PageHeader } from "@eunomia/ui";
import { PageSection } from "@eunomia/ui";
import { StatTile } from "@eunomia/ui";
import type { FeatureCardProps } from "@eunomia/ui";
import {
  useGetAgreements,
  useGetNegotiationProcesses,
} from "shared/src/data/orval/negotiations/negotiations";
import { useGetTransferProcesses } from "shared/src/data/orval/transfers/transfers";
import { useGetUnreadEventCount } from "shared/src/data/orval/events/events";

export const Route = createFileRoute("/")({
  component: Overview,
});

const FEATURES: FeatureCardProps[] = [
  {
    icon: ShieldCheck,
    title: "Dataspace Protocol compliant",
    description:
      "Catalog, contract negotiation and transfer process exactly as Dataspace Protocol 2025-1 defines them, over JSON-LD.",
    tags: ["DSP 2025-1", "JSON-LD", "ODRL"],
    to: "/contract-negotiation",
    action: "Review negotiations",
  },
  {
    icon: ArrowLeftRight,
    title: "Embedded transfer dataplane",
    description:
      "Push and pull transfers served by the built-in dataplane, with a proxy per transfer and a log of every state it goes through.",
    tags: ["Push", "Pull", "HTTP proxy"],
    to: "/transfer-process",
    action: "Monitor transfers",
  },
  {
    icon: BookOpen,
    title: "Catalog and connector configuration",
    description:
      "Publish datasets, distributions and ODRL offers, and bind each distribution to a connector built from a template.",
    tags: ["DCAT-AP", "Connector templates"],
    to: "/my-catalog",
    action: "Open my catalog",
  },
  {
    icon: Radio,
    title: "Pub/Sub events",
    description:
      "Every change is published on the event bus, streamed live to this console and delivered to signed webhooks.",
    tags: ["Live stream", "Webhooks", "Dead letters"],
    to: "/events/feed",
    action: "Open the event feed",
  },
  {
    icon: Fingerprint,
    title: "GNAP4DS, Decentralized Claims, OAuth",
    description:
      "Peers authenticate with GNAP for dataspaces and verifiable credentials from the wallet; people sign in through OAuth 2.0.",
    tags: ["GNAP4DS", "DCP", "OID4VP", "OAuth 2.0"],
    to: "/wallet/info",
    action: "Open the wallet",
  },
];

/** One capability per line: what it is, its standards and where to go. */
function CapabilityRow({ icon: Icon, title, description, tags, to, action }: FeatureCardProps) {
  return (
    <Link
      to={to}
      title={action}
      className="group flex items-center gap-3 px-3 py-2 transition-colors hover:bg-ink/[0.04]"
    >
      <Icon className="h-4 w-4 shrink-0 text-brand-sky" />
      <div className="min-w-0 flex-1">
        <div className="truncate text-sm font-medium text-ink/90">{title}</div>
        <div className="truncate text-xs text-ink/55">{description}</div>
      </div>
      <div className="hidden shrink-0 gap-1 lg:flex">
        {tags?.map((t) => (
          <span key={t} className="rounded bg-ink/[0.06] px-1.5 py-0.5 text-xs text-ink/60">
            {t}
          </span>
        ))}
      </div>
      <ArrowRight className="h-3.5 w-3.5 shrink-0 text-ink/30 transition-transform group-hover:translate-x-0.5 group-hover:text-brand-sky" />
    </Link>
  );
}

function Overview() {
  const total = (r: { status: number; data: unknown } | undefined) =>
    r?.status === 200 ? (r.data as { total?: number }).total : undefined;

  const { data: negotiations } = useGetNegotiationProcesses({ limit: 1 });
  const { data: agreements } = useGetAgreements({ limit: 1 });
  const { data: transfers } = useGetTransferProcesses({ limit: 1 });
  const { data: started } = useGetTransferProcesses({ limit: 1, state: "STARTED" });
  const { data: unread } = useGetUnreadEventCount();

  return (
    <PageLayout>
      <PageHeader
        title="Overview"
        description="Eunomia DS-Agent, a Dataspace Protocol connector for sovereign data sharing."
      />

      <PageSection title="Activity">
        <div className="grid grid-cols-1 gap-3 sm:grid-cols-2 xl:grid-cols-4">
          <StatTile
            label="Contract negotiations"
            value={total(negotiations)}
            icon={Handshake}
            to="/contract-negotiation"
          />
          <StatTile
            label="Agreements"
            value={total(agreements)}
            icon={FileSignature}
            to="/agreements"
          />
          <StatTile
            label="Transfers"
            value={total(transfers)}
            hint={total(started) !== undefined ? `${total(started)} in progress` : undefined}
            icon={ArrowLeftRight}
            to="/transfer-process"
          />
          <StatTile
            label="Unread events"
            value={unread?.status === 200 ? unread.data.count : undefined}
            icon={BellDot}
            to="/events/feed"
          />
        </div>
      </PageSection>

      <PageSection title="Capabilities">
        <div className="divide-y divide-ink/[0.06] overflow-hidden rounded-lg border border-ink/10">
          {FEATURES.map((f) => (
            <CapabilityRow key={f.title} {...f} />
          ))}
        </div>
      </PageSection>
    </PageLayout>
  );
}
