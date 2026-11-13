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
      { title: "Catalog Browser", url: "/admin/catalog", icon: Search },
      { title: "Participants", url: "/admin/participants", icon: Users },
      { title: "Connections", url: "/admin/connections", icon: Link2 },
      { title: "VC Requests", url: "/admin/authority", icon: ShieldCheck },
    ],
  },
  {
    title: "Contracts",
    items: [
      { title: "Negotiations", url: "/admin/contract-negotiation", icon: Feather },
      { title: "Agreements", url: "/admin/agreements", icon: Handshake },
      { title: "Transfers", url: "/admin/transfer-process", icon: ArrowLeftRight },
    ],
  },
  {
    title: "Publishing",
    items: [
      { title: "My Catalog", url: "/admin/my-catalog", icon: Archive },
      { title: "Data Schemas", url: "/admin/my-catalog/schemas", icon: FileJson },
      { title: "Policy Library", url: "/admin/my-catalog/policies", icon: Library },
      {
        title: "Connector Templates",
        url: "/admin/my-catalog/connectors",
        section: "/admin/my-catalog/connectors",
        icon: Plug,
      },
    ],
  },
  {
    title: "Agent",
    items: [
      { title: "Wallet", url: "/admin/wallet/info", section: "/admin/wallet", icon: Wallet },
      {
        title: "Keystore",
        url: "/admin/keystore/parameters",
        section: "/admin/keystore",
        icon: KeyRound,
      },
      { title: "Events", url: "/admin/events/feed", section: "/admin/events", icon: Radio },
    ],
  },
];

export function AppSidebar() {
  return (
    <AppShellSidebar
      productName="DS-Agent"
      homeTo="/admin/"
      navGroups={NAV_GROUPS}
      footerSlot={<DevMode />}
      footerClassName="p-2 pb-12 border-t border-sidebar-border/40"
    />
  );
}
