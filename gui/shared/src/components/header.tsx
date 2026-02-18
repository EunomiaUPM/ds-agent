/**
 * header.tsx
 *
 * Main application header component with dynamic breadcrumb navigation
 * and user action controls.
 *
 * Features:
 * - Clean breadcrumb generation from current route
 * - Proper formatting for URNs and path segments
 * - Responsive design with dropdown for intermediate items on mobile
 * - User authentication controls
 *
 * @example
 * <Header />
 */

import React, { useContext, useMemo } from "react";
import { Link, useRouterState } from "@tanstack/react-router";
import { cn, formatUrn } from "shared/src/lib/utils";
import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbLink,
  BreadcrumbList,
  BreadcrumbPage,
  BreadcrumbSeparator,
  BreadcrumbEllipsis,
} from "shared/src/components/ui/breadcrumb";
import { AuthContext, AuthContextType } from "shared/src/context/AuthContext";
import { Button } from "shared/src/components/ui/button";
import { LogOut, Bell, User } from "lucide-react";
import { SidebarTrigger } from "shared/src/components/ui/sidebar";
import { Separator } from "shared/src/components/ui/separator";
import { Tooltip, TooltipContent, TooltipTrigger } from "shared/src/components/ui/tooltip";

// =============================================================================
// TYPES
// =============================================================================

/**
 * Represents a single breadcrumb item.
 */
interface BreadcrumbItemData {
  /** Unique key for React rendering */
  key: string;
  /** Navigation path */
  href: string;
  /** Display label */
  label: string;
  /** Whether this is a dynamic segment (e.g., urn:xxx) */
  isDynamic: boolean;
}

// =============================================================================
// CONSTANTS
// =============================================================================

/**
 * Mapping of route segments to human-readable labels.
 * Add new routes here for proper display.
 */
const ROUTE_LABELS: Record<string, string> = {
  "datahub-catalog": "DataHub Catalog",
  "provider-catalog": "Provider Catalog",
  "contract-negotiation": "Contract Negotiation",
  "transfer-process": "Transfer Process",
  "business-requests": "Business Requests",
  "customer-requests": "Customer Requests",
  subscriptions: "Subscriptions",
  participants: "Participants",
  agreements: "Agreements",
  catalog: "Catalog",
  dataset: "Dataset",
  dashboard: "Dashboard",
  login: "Login",
};

/**
 * Segments that should be hidden from breadcrumbs.
 * These are typically route structure artifacts.
 */
const HIDDEN_SEGMENTS = new Set(["data-service"]);

/**
 * Segments that should be merged with the following segment.
 * For example: /catalog/$catalogId becomes "Catalog: catalogId"
 */
const MERGE_WITH_NEXT = new Set(["dataset", "transfer-message", "cn-message"]);

// =============================================================================
// HELPERS
// =============================================================================

/**
 * Formats a path segment for display.
 *
 * @param segment - Raw URL segment
 * @returns Formatted display label
 */
function formatSegmentLabel(segment: string): string {
  // Check for configured label
  if (ROUTE_LABELS[segment]) {
    return ROUTE_LABELS[segment];
  }

  // Format URNs
  if (segment.includes("urn:")) {
    return formatUrn(segment);
  }

  // Convert kebab-case to Title Case
  return segment
    .split("-")
    .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
    .join(" ");
}

/**
 * Checks if a segment is a dynamic route parameter (URN or UUID-like).
 */
function isDynamicSegment(segment: string): boolean {
  return (
    segment.includes("urn:") ||
    /^[0-9a-f]{8}-[0-9a-f]{4}-/i.test(segment) ||
    segment.startsWith("$")
  );
}

/**
 * Generates breadcrumb items from a pathname.
 *
 * @param pathname - Current URL pathname
 * @returns Array of breadcrumb items
 */
function generateBreadcrumbs(pathname: string): BreadcrumbItemData[] {
  const segments = pathname.split("/").filter((s) => s !== "");
  const items: BreadcrumbItemData[] = [];
  let currentPath = "";

  for (let i = 0; i < segments.length; i++) {
    const segment = decodeURIComponent(segments[i]);
    currentPath += `/${segments[i]}`;

    // Skip hidden segments
    if (HIDDEN_SEGMENTS.has(segment)) {
      continue;
    }

    // Check if this segment should be merged with next
    const nextSegment = segments[i + 1] ? decodeURIComponent(segments[i + 1]) : null;
    if (MERGE_WITH_NEXT.has(segment) && nextSegment) {
      // Skip this segment, it will be included in the next item's label
      continue;
    }

    // Check if previous segment wanted to merge
    const prevSegment = segments[i - 1] ? decodeURIComponent(segments[i - 1]) : null;
    let label = formatSegmentLabel(segment);
    if (prevSegment && MERGE_WITH_NEXT.has(prevSegment)) {
      const prevLabel = formatSegmentLabel(prevSegment);
      label = `${prevLabel}: ${formatSegmentLabel(segment)}`;
    }

    items.push({
      key: currentPath,
      href: currentPath,
      label,
      isDynamic: isDynamicSegment(segment),
    });
  }

  return items;
}

// =============================================================================
// COMPONENT
// =============================================================================

/**
 * Application header with breadcrumb navigation and user controls.
 *
 * Breadcrumbs are automatically generated from the current URL path
 * with smart formatting and responsive behavior.
 */
export const Header = () => {
  const routerState = useRouterState();
  const { isAuthenticated, unsetAuthentication } = useContext<AuthContextType | null>(AuthContext)!;

  // ---------------------------------------------------------------------------
  // Breadcrumb Generation
  // ---------------------------------------------------------------------------

  const breadcrumbs = useMemo(() => {
    return generateBreadcrumbs(routerState.location.pathname);
  }, [routerState.location.pathname]);

  // ---------------------------------------------------------------------------
  // Render Helpers
  // ---------------------------------------------------------------------------

  /**
   * Renders the breadcrumb items with responsive behavior.
   * - On mobile: Shows first, dropdown with middle items, and last
   * - On desktop: Shows all items
   */
  const renderBreadcrumbs = () => {
    if (breadcrumbs.length === 0) {
      return null;
    }

    // If only 1-3 items, show all
    if (breadcrumbs.length <= 3) {
      return breadcrumbs.map((item, index) => {
        const isLast = index === breadcrumbs.length - 1;
        return (
          <React.Fragment key={item.key}>
            <BreadcrumbItem className="truncate max-w-[200px]">
              {isLast ? (
                <BreadcrumbPage className="truncate font-medium">{item.label}</BreadcrumbPage>
              ) : (
                <BreadcrumbLink asChild>
                  <Link
                    to={item.href}
                    className="text-muted-foreground hover:text-foreground transition-colors truncate"
                  >
                    {item.label}
                  </Link>
                </BreadcrumbLink>
              )}
            </BreadcrumbItem>
            {!isLast && <BreadcrumbSeparator />}
          </React.Fragment>
        );
      });
    }

    // For 4+ items: show first, dropdown, last
    const firstItem = breadcrumbs[0];
    const middleItems = breadcrumbs.slice(1, -1);
    const lastItem = breadcrumbs[breadcrumbs.length - 1];

    return (
      <>
        {/* First Item */}
        <BreadcrumbItem className="truncate max-w-[150px]">
          <BreadcrumbLink asChild>
            <Link
              to={firstItem.href}
              className="text-muted-foreground hover:text-foreground transition-colors truncate"
            >
              {firstItem.label}
            </Link>
          </BreadcrumbLink>
        </BreadcrumbItem>
        <BreadcrumbSeparator />

        {/* Middle Items Ellipsis (shown on mobile, hidden on desktop) */}
        <BreadcrumbItem className="md:hidden">
          <Tooltip>
            <TooltipTrigger className="flex items-center text-muted-foreground hover:text-foreground transition-colors">
              <BreadcrumbEllipsis className="h-4 w-4" />
            </TooltipTrigger>
            <TooltipContent side="bottom" className="flex flex-col gap-1">
              {middleItems.map((item) => (
                <Link key={item.key} to={item.href} className="text-sm hover:underline">
                  {item.label}
                </Link>
              ))}
            </TooltipContent>
          </Tooltip>
        </BreadcrumbItem>
        <BreadcrumbSeparator className="md:hidden" />

        {/* Middle Items (visible on desktop) */}
        {middleItems.map((item) => (
          <React.Fragment key={item.key}>
            <BreadcrumbItem className="hidden md:inline-flex truncate max-w-[150px]">
              <BreadcrumbLink asChild>
                <Link
                  to={item.href}
                  className="text-muted-foreground hover:text-foreground transition-colors truncate"
                >
                  {item.label}
                </Link>
              </BreadcrumbLink>
            </BreadcrumbItem>
            <BreadcrumbSeparator className="hidden md:flex" />
          </React.Fragment>
        ))}

        {/* Last Item */}
        <BreadcrumbItem className="truncate max-w-[200px]">
          <BreadcrumbPage className="truncate font-medium">{lastItem.label}</BreadcrumbPage>
        </BreadcrumbItem>
      </>
    );
  };

  // ---------------------------------------------------------------------------
  // Render
  // ---------------------------------------------------------------------------

  return (
    <div className="bg-background w-full border-b border-white/5 py-1 z-50 h-9 px-3 flex justify-between items-center gap-4 sticky top-0">
      {/* Left section: Sidebar trigger and breadcrumbs */}
      <div className="flex items-center gap-2 overflow-hidden min-w-0">
        <SidebarTrigger className="h-6 w-6 shrink-0" />
        <Separator orientation="vertical" className="h-4 shrink-0" />
        <Breadcrumb className="overflow-hidden">
          <BreadcrumbList className="flex-nowrap">{renderBreadcrumbs()}</BreadcrumbList>
        </Breadcrumb>
      </div>

      {/* Right section: User actions */}
      <div className="flex flex-row gap-4 shrink-0">
        <Link to="/subscriptions">
          <Bell className="cursor-pointer text-muted-foreground hover:text-foreground transition-colors h-5 w-5" />
        </Link>

        <Link to="">
          <User className="cursor-pointer text-muted-foreground hover:text-foreground transition-colors h-5 w-5" />
        </Link>

        {isAuthenticated && (
          <Button variant="ghost" size="xs" onClick={() => unsetAuthentication()}>
            Logout
            <LogOut className="ml-2 h-4 w-4" />
          </Button>
        )}
      </div>
    </div>
  );
};
