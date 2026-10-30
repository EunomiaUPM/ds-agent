/**
 * AppSidebar.tsx
 *
 * Main navigation sidebar for the Admin/Provider application.
 * Displays navigation links with icons, filtered based on catalog type.
 *
 * The sidebar adapts its menu items based on the `catalog_type` from
 * GlobalInfoContext, showing either Datahub or Eunomia DS-Agent catalog options.
 */

import {
  Archive,
  FileJson,
  Library,
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
import React, { useContext } from "react";
import { AppShellSidebar, type NavGroup } from "@eunomia/ui";
import { GlobalInfoContext, GlobalInfoContextType } from "shared/src/context/GlobalInfoContext";
import { DevMode } from "./DevMode";

export function AppSidebar() {
  const { catalog_type } = useContext<GlobalInfoContextType | null>(GlobalInfoContext)!;

  const generalItems = [
    {
      title: "Catalog Browser",
      url: "/admin/catalog",
      icon: Search,
    },
    {
      title: "Datahub Catalogs",
      url: "/admin/datahub-catalog",
      icon: Archive,
    },
    {
      title: "Negotiations",
      url: "/admin/contract-negotiation",
      icon: Feather,
    },
    {
      title: "Agreements",
      url: "/admin/agreements",
      icon: Handshake,
    },
    {
      title: "Transfers",
      url: "/admin/transfer-process",
      icon: ArrowLeftRight,
    },
    {
      title: "Participants",
      url: "/admin/participants",
      icon: Users,
    },
    {
      title: "VC Requests",
      url: "/admin/authority",
      icon: ShieldCheck,
    },
    {
      title: "My Connections",
      url: "/admin/connections",
      icon: Users,
    },
  ].filter((item) => {
    if (catalog_type === "datahub" && item.title === "Catalogs") return false;
    if (catalog_type === "agent" && item.title === "Datahub Catalogs") return false;
    return true;
  });

  const navGroups: NavGroup[] = [
    {
      title: "General",
      items: generalItems,
    },
    {
      title: "My area",
      items: [
        {
          title: "My Wallet",
          url: "/admin/wallet/info",
          section: "/admin/wallet",
          icon: Wallet,
        },
        {
          title: "Keystore",
          url: "/admin/keystore/parameters",
          section: "/admin/keystore",
          icon: KeyRound,
        },
        {
          title: "My Catalog",
          url: "/admin/my-catalog",
          icon: Archive,
        },
        {
          title: "Data Schemas",
          url: "/admin/my-catalog/schemas",
          icon: FileJson,
        },
        {
          title: "Policy Library",
          url: "/admin/my-catalog/policies",
          icon: Library,
        },
        {
          title: "Events",
          url: "/admin/events/feed",
          section: "/admin/events",
          icon: Radio,
        },
      ],
    },
  ];

  return (
    <AppShellSidebar
      productName="DS-Agent"
      homeTo="/admin/"
      navGroups={navGroups}
      footerSlot={<DevMode />}
      footerClassName="p-3 pb-14 border-t border-sidebar-border/40"
    />
  );
}
