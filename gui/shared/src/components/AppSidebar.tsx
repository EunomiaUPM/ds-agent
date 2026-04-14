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

import { Archive, ArrowLeftRight, Feather, Handshake, Users, Lock, Search } from "lucide-react";
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
  SidebarGroupLabel
} from "./ui/sidebar";
import logoImg from "./../img/eunomia_logo_lg_light.svg";
import { GlobalInfoContext, GlobalInfoContextType } from "shared/src/context/GlobalInfoContext";

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
 * - Logo display at the top
 * - Navigation menu with icons
 * - Active state highlighting based on current route
 * - Dynamic filtering based on catalog type configuration
 *
 * @returns The sidebar navigation component
 */
export function AppSidebar() {
  const routerState = useRouterState();
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
        title: "Catalog",
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
        title: "SSI Auth",
        url: "/admin/ssi-auth",
        icon: Lock,
      },
    ],
  },
  {
    title: "My area",
    items: [
        {
        title: "My Catalog",
        url: "/admin/my-catalog",
        icon: Archive,
      },
    ]
  }
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
    <Sidebar className="bg-base-sidebar">
      <SidebarContent>
           {/* Logo */}
          <Link to="/admin/">
            <img
              src={logoImg}
              className="h-11 mt-2 mb-4 mr-auto ml-1 flex justify-start object-contain"
              alt="Eunomia Logo"
            />
            {console.log(itemsFiltered, "itemsFiltered")}
          </Link>
         {navGroups.map((group) => (
        <SidebarGroup>
          {/* Navigation Menu */}
          <SidebarGroupContent>
            <SidebarGroupLabel>{group.title}</SidebarGroupLabel>
            <SidebarMenu>
              {(navGroups[0].title !== group.title) ? (group.items.map((item) => (
                <SidebarMenuItem key={item.title}>
                  <SidebarMenuButton asChild>
                    <Link
                      to={item.url}
                      className={
                        routerState.location.pathname === item.url ? "bg-white/10 text-white" : ""
                      }
                    >
                      <item.icon />
                      <span>{item.title}</span>
                    </Link>
                  </SidebarMenuButton>
                </SidebarMenuItem>
              )))
               : 
              (itemsFiltered.map((item) => (
                <SidebarMenuItem key={item.title}>
                  <SidebarMenuButton asChild>
                    <Link
                      to={item.url}
                      className={
                        routerState.location.pathname === item.url ? "bg-white/10 text-white" : ""
                      }
                    >
                      <item.icon />
                      <span>{item.title}</span>
                    </Link>
                  </SidebarMenuButton>
                </SidebarMenuItem>
              )
            ))}
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>
         )
        )}
      </SidebarContent>
    </Sidebar>
  );
}
