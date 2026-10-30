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

import { createFileRoute } from "@tanstack/react-router";
import {
  ArrowLeftRight,
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
import { FeatureCard, FeatureCardProps } from "@eunomia/ui";
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
        <div className="grid grid-cols-1 gap-3 md:grid-cols-2 xl:grid-cols-3">
          {FEATURES.map((f) => (
            <FeatureCard key={f.title} {...f} />
          ))}
        </div>
      </PageSection>
    </PageLayout>
  );
}
