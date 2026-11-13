/**
 * AppSidebar.tsx
 *
 * Main navigation sidebar of the admin console, grouped by what the operator is doing.
 */

import {
  Archive,
  FileJson,
  Library,
  Link2,
  Plug,
  ArrowLeftRight,
  Feather,
  Handshake,
  Users,
  Search,
  Wallet,
  ShieldCheck,
  KeyRound,
  Radio,
} from "lucide-react";
import React from "react";
import { AppShellSidebar, type NavGroup } from "@eunomia/ui";
import { DevMode } from "./DevMode";

const NAV_GROUPS: NavGroup[] = [
  {
    title: "Dataspace",
    items: [
      { title: "Catalog Browser", url: "/catalog", icon: Search },
      { title: "Participants", url: "/participants", icon: Users },
      { title: "Connections", url: "/connections", icon: Link2 },
      { title: "VC Requests", url: "/authority", icon: ShieldCheck },
    ],
  },
  {
    title: "Contracts",
    items: [
      { title: "Negotiations", url: "/contract-negotiation", icon: Feather },
      { title: "Agreements", url: "/agreements", icon: Handshake },
      { title: "Transfers", url: "/transfer-process", icon: ArrowLeftRight },
    ],
  },
  {
    title: "Publishing",
    items: [
      { title: "My Catalog", url: "/my-catalog", icon: Archive },
      { title: "Data Schemas", url: "/my-catalog/schemas", icon: FileJson },
      { title: "Policy Library", url: "/my-catalog/policies", icon: Library },
      {
        title: "Connector Templates",
        url: "/my-catalog/connectors",
        section: "/my-catalog/connectors",
        icon: Plug,
      },
    ],
  },
  {
    title: "Agent",
    items: [
      { title: "Wallet", url: "/wallet/info", section: "/wallet", icon: Wallet },
      {
        title: "Keystore",
        url: "/keystore/parameters",
        section: "/keystore",
        icon: KeyRound,
      },
      { title: "Events", url: "/events/feed", section: "/events", icon: Radio },
    ],
  },
];

export function AppSidebar() {
  return (
    <AppShellSidebar
      productName="DS-Agent"
      homeTo="/"
      navGroups={NAV_GROUPS}
      footerSlot={<DevMode />}
      footerClassName="p-2 pb-12 border-t border-sidebar-border/40"
    />
  );
}
