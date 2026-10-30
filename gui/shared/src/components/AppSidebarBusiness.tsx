import { Apple, Archive, ChartBarIncreasing } from "lucide-react";
import { eunomiaLogoLight as logoImg } from "@eunomia/ui";
import { eunomiaLogoDark as logoImgDark } from "@eunomia/ui";
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
} from "@eunomia/ui";

import { AuthContext, AuthContextType } from "shared/src/context/AuthContext";
import { useTheme } from "@eunomia/ui";

const businessItems = [
  {
    title: "Datahub Catalogs",
    url: "/datahub-catalog",
    icon: Archive,
  },
  {
    title: "Data Access Requests",
    url: "/business-requests",
    icon: Apple,
  },
  {
    title: "Dashboard",
    url: "/dashboard",
    icon: ChartBarIncreasing,
  },
];
const customerItems = [
  {
    title: "Datahub Catalogs",
    url: "/datahub-catalog",
    icon: Archive,
  },
  {
    title: "Customer Requests",
    url: "/customer-requests",
    icon: Apple,
  },
];

/**
 * Sidebar component for Business users.
 */
export function AppSidebarBusiness() {
  const routerState = useRouterState();
  const { resolvedTheme } = useTheme();
  const { participant } = useContext<AuthContextType | null>(AuthContext)!;

  return (
    <Sidebar className="bg-base-sidebar">
      <SidebarContent>
        <SidebarGroup>
          <img
            src={resolvedTheme === "dark" ? logoImg : logoImgDark}
            className="h-11 mt-2 mb-4 mr-auto ml-1 flex justify-start object-contain"
          ></img>

          <SidebarGroupContent>
            <SidebarMenu>
              {participant?.participant_type == "Provider" &&
                businessItems.map((item) => (
                  <SidebarMenuItem key={item.title}>
                    <SidebarMenuButton asChild>
                      <Link
                        to={item.url}
                        className={
                          routerState.location.pathname === item.url ? "bg-ink/10 text-ink" : ""
                        }
                      >
                        <item.icon />
                        <span>{item.title}</span>
                      </Link>
                    </SidebarMenuButton>
                  </SidebarMenuItem>
                ))}
              {participant?.participant_type == "Consumer" &&
                customerItems.map((item) => (
                  <SidebarMenuItem key={item.title}>
                    <SidebarMenuButton asChild>
                      <Link
                        to={item.url}
                        className={
                          routerState.location.pathname === item.url ? "bg-ink/10 text-ink" : ""
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
