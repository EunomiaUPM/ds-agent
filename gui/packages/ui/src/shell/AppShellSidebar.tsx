import React from "react";
import { useActivePath, useLink } from "../link";
import { useTheme } from "../hooks/useTheme";
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarGroup,
  SidebarGroupContent,
  SidebarGroupLabel,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
} from "../ui/sidebar";
import defaultLogoOnDark from "../assets/eunomia_logo_lg_light.svg";
import defaultLogoOnLight from "../assets/eunomia_logo_lg_dark.svg";

export interface NavItem {
  /** Display title for the menu item */
  title: string;
  /** Route URL to navigate to */
  url: string;
  /** Lucide icon component to display */
  icon: React.ComponentType<{ className?: string }>;
  /** Path prefix that marks the item active; defaults to `url` */
  section?: string;
}

export interface NavGroup {
  title: string;
  items: NavItem[];
}

export interface AppShellSidebarProps {
  /** Product tagline shown under the Eunomia logo (e.g., "DS-Agent", "Heimdall") */
  productName: string;
  /** Target route when clicking the sidebar brand header */
  homeTo?: string;
  /** Navigation groups and items */
  navGroups: NavGroup[];
  /** Optional explicit pathname override */
  activePath?: string;
  /** Logo rendered on dark backgrounds */
  logoOnDark?: string;
  /** Logo rendered on light backgrounds */
  logoOnLight?: string;
  /** Optional footer content (e.g., DevMode widget or version info) */
  footerSlot?: React.ReactNode;
  footerClassName?: string;
}

export function AppShellSidebar({
  productName,
  homeTo = "/",
  navGroups,
  activePath,
  logoOnDark = defaultLogoOnDark,
  logoOnLight = defaultLogoOnLight,
  footerSlot,
  footerClassName = "p-3 border-t border-sidebar-border/40",
}: AppShellSidebarProps) {
  const Link = useLink();
  const ctxPath = useActivePath();
  const { resolvedTheme } = useTheme();
  const isDark = resolvedTheme === "dark";

  const pathname = (activePath ?? ctxPath).replace(/\/$/, "");
  const activeItem = navGroups
    .flatMap((group) => group.items)
    .filter((item) => {
      const section = (item.section ?? item.url).replace(/\/$/, "");
      return pathname === section || pathname.startsWith(`${section}/`);
    })
    .sort(
      (a, b) =>
        (b.section ?? b.url).replace(/\/$/, "").length -
        (a.section ?? a.url).replace(/\/$/, "").length,
    )[0];

  return (
    <Sidebar className="bg-base-sidebar z-40">
      <SidebarHeader className="px-3 pt-3 pb-2.5 border-b border-sidebar-border/40">
        <Link
          to={homeTo}
          className="block group p-1 rounded-xl hover:bg-ink/[0.03] transition-colors"
        >
          <img
            src={isDark ? logoOnDark : logoOnLight}
            className="w-full h-auto object-contain pl-1 pr-3 pt-1 pb-1 transition-opacity group-hover:opacity-90"
            alt="Eunomia Logo"
          />
          <div className="flex items-center gap-2 mt-2 px-1 group-data-[collapsible=icon]:hidden">
            <span className="text-xs font-black tracking-widest uppercase font-mono text-slate-900 dark:text-white">
              {productName}
            </span>
            <span className="h-1.5 w-1.5 rounded-full bg-emerald-500 animate-pulse" />
          </div>
        </Link>
      </SidebarHeader>

      <SidebarContent className="overflow-y-auto">
        {navGroups.map((group) => (
          <SidebarGroup key={group.title}>
            <SidebarGroupContent>
              <SidebarGroupLabel>{group.title}</SidebarGroupLabel>
              <SidebarMenu>
                {group.items.map((item) => (
                  <SidebarMenuItem key={item.title}>
                    <SidebarMenuButton asChild>
                      <Link
                        to={item.url}
                        className={item === activeItem ? "bg-ink/10 text-ink" : ""}
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

      {footerSlot && <SidebarFooter className={footerClassName}>{footerSlot}</SidebarFooter>}
    </Sidebar>
  );
}
