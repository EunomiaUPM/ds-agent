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

import { Archive, ArrowLeftRight, Feather, Handshake, Users, Lock } from "lucide-react";
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
} from "./ui/sidebar";
import logoImg from "./../img/eunomia_logo_lg_light.svg";
import { GlobalInfoContext, GlobalInfoContextType } from "shared/src/context/GlobalInfoContext";

// =============================================================================
// TYPES
// =============================================================================

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
  const items: NavItem[] = [
    {
      title: "Catalogs",
      url: "/admin/catalog",
      icon: Archive,
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
  ];

  /**
   * Filter items based on catalog type.
   * - Datahub: Hide "Catalogs" (show Datahub Catalogs)
   * - Eunomia DS-Agent: Hide "Datahub Catalogs" (show Catalogs)
   */
  const itemsFiltered = items.filter((item) => {
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
        <SidebarGroup>
          {/* Logo */}
          <Link to="/admin/">
            <img
              src={logoImg}
              className="h-11 mt-2 mb-4 mr-auto ml-1 flex justify-start object-contain"
              alt="Eunomia Logo"
            />
          </Link>

          {/* Navigation Menu */}
          <SidebarGroupContent>
            <SidebarMenu>
              {itemsFiltered.map((item) => (
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
              ))}
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>
      </SidebarContent>
    </Sidebar>
  );
}
