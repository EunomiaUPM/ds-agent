/**
 * AppSidebar.tsx
 *
 * Main navigation sidebar for the Admin/Provider application.
 * Displays navigation links with icons, filtered based on catalog type.
 *
 * The sidebar adapts its menu items based on the `catalog_type` from
 * GlobalInfoContext, showing either Datahub or Eunomia DS-Agent catalog options.
 *
 * @example
 * // Used in the root layout
 * <SidebarProvider>
 *   <AppSidebar />
 *   <SidebarInset>
 *     <Outlet />
 *   </SidebarInset>
 * </SidebarProvider>
 */

import {
  Archive,
  ArrowLeftRight,
  Feather,
  Handshake,
  Users,
  Search,
  Wallet,
  ShieldCheck,
  KeyRound,
  Lock,
  Radio,
} from "lucide-react";
import React, { useContext } from "react";
import { Link, useRouterState } from "@tanstack/react-router";
import {
  Sidebar,
  SidebarContent,
  SidebarGroup,
  SidebarGroupContent,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarGroupLabel,
  SidebarHeader,
  SidebarFooter,
} from "./ui/sidebar";
import logoImg from "./../img/eunomia_logo_lg_light.svg";
import logoImgDark from "./../img/eunomia_logo_lg_dark.svg";
import { useTheme } from "shared/src/hooks/useTheme";
import { GlobalInfoContext, GlobalInfoContextType } from "shared/src/context/GlobalInfoContext";
import { DevMode } from "./DevMode";

// =============================================================================
// TYPES
// =============================================================================

/**
 * Navigation group configuration
 */
interface NavGroup {
  title: string; // título del grupo (ej: "General", "Admin", etc.)
  items: NavItem[];
}

/**
 * Navigation item configuration.
 */
interface NavItem {
  /** Display title for the menu item */
  title: string;
  /** Route URL to navigate to */
  url: string;
  /** Lucide icon component to display */
  icon: React.ComponentType<{ className?: string }>;
}

// =============================================================================
// COMPONENT
// =============================================================================

/**
 * Provider/Admin sidebar navigation component.
 *
 * Features:
 * - Logo display at the top with DS-Agent tagline
 * - Navigation menu with icons
 * - Active state highlighting based on current route
 * - Dynamic filtering based on catalog type configuration
 * - DevMode widget in the sidebar footer above TanStackRouterDevtools
 *
 * @returns The sidebar navigation component
 */
export function AppSidebar() {
  const routerState = useRouterState();
  const { resolvedTheme } = useTheme();
  const isDark = resolvedTheme === "dark";
  const { catalog_type } = useContext<GlobalInfoContextType | null>(GlobalInfoContext)!;

  // ---------------------------------------------------------------------------
  // Navigation Items Configuration
  // ---------------------------------------------------------------------------

  /**
   * Full list of navigation items.
   * Items are filtered based on catalog_type before rendering.
   */
  const navGroups: NavGroup[] = [
    {
      title: "General",
      items: [
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
          title: "Contract Negotiation",
          url: "/admin/contract-negotiation",
          icon: Feather,
        },
        {
          title: "Agreements",
          url: "/admin/agreements",
          icon: Handshake,
        },
        {
          title: "Transferences",
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
      ],
    },
    {
      title: "My area",
      items: [
        {
          title: "My Wallet",
          url: "/admin/wallet/info",
          icon: Wallet,
        },
        {
          title: "Keystore",
          url: "/admin/keystore/parameters",
          icon: KeyRound,
        },
        {
          title: "My Catalog",
          url: "/admin/my-catalog",
          icon: Archive,
        },
        {
          title: "OAuth & Security",
          url: "/admin/oauth/clients",
          icon: Lock,
        },
        {
          title: "Events & Bus",
          url: "/admin/events/feed",
          icon: Radio,
        },
      ],
    },
  ];

  /**
   * Filter items based on catalog type.
   * - Datahub: Hide "Catalogs" (show Datahub Catalogs)
   * - Eunomia DS-Agent: Hide "Datahub Catalogs" (show Catalogs)
   */
  const itemsFiltered = navGroups[0].items.filter((item) => {
    if (catalog_type === "datahub") {
      if (item.title === "Catalogs") return false;
    }
    if (catalog_type === "agent") {
      if (item.title === "Datahub Catalogs") return false;
    }
    return true;
  });

  // ---------------------------------------------------------------------------
  // Render
  // ---------------------------------------------------------------------------

  return (
    <Sidebar className="bg-base-sidebar z-40">
      {/* Sidebar Header with Eunomia Logo & DS-Agent Tagline */}
      <SidebarHeader className="px-3 pt-3 pb-2.5 border-b border-sidebar-border/40">
        <Link
          to="/admin/"
          className="block group p-1 rounded-xl hover:bg-ink/[0.03] transition-colors"
        >
          <img
            src={isDark ? logoImg : logoImgDark}
            className="w-full h-auto object-contain pl-1 pr-3 pt-1 pb-1 transition-opacity group-hover:opacity-90"
            alt="Eunomia Logo"
          />
          <div className="flex items-center gap-2 mt-2 px-1 group-data-[collapsible=icon]:hidden">
            <span className="text-xs font-black tracking-widest uppercase font-mono text-slate-900 dark:text-white">
              DS-Agent
            </span>
            <span className="h-1.5 w-1.5 rounded-full bg-emerald-500 animate-pulse" />
          </div>
        </Link>
      </SidebarHeader>

      {/* Main Navigation Menu */}
      <SidebarContent className="overflow-y-auto">
        {navGroups.map((group) => (
          <SidebarGroup key={group.title}>
            <SidebarGroupContent>
              <SidebarGroupLabel>{group.title}</SidebarGroupLabel>
              <SidebarMenu>
                {navGroups[0].title !== group.title
                  ? group.items.map((item) => (
                      <SidebarMenuItem key={item.title}>
                        <SidebarMenuButton asChild>
                          <Link
                            to={item.url}
                            className={
                              routerState.location.pathname === item.url
                                ? "bg-ink/10 text-ink"
                                : ""
                            }
                          >
                            <item.icon />
                            <span>{item.title}</span>
                          </Link>
                        </SidebarMenuButton>
                      </SidebarMenuItem>
                    ))
                  : itemsFiltered.map((item) => (
                      <SidebarMenuItem key={item.title}>
                        <SidebarMenuButton asChild>
                          <Link
                            to={item.url}
                            className={
                              routerState.location.pathname === item.url
                                ? "bg-ink/10 text-ink"
                                : ""
                            }
                          >
                            <item.icon />
                            <span>{item.title}</span>
                          </Link>
                        </SidebarMenuButton>
                      </SidebarMenuItem>
                    ))}
              </SidebarMenu>
            </SidebarGroupContent>
          </SidebarGroup>
        ))}
      </SidebarContent>

      {/* Sidebar Footer with DevMode widget positioned above TanStackRouterDevtools */}
      <SidebarFooter className="p-3 pb-14 border-t border-sidebar-border/40">
        <DevMode />
      </SidebarFooter>
    </Sidebar>
  );
}
