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
import { LogOut, Bell, User, Check, Copy, ExternalLink, Key, Radio, Shield } from "lucide-react";
import { Badge } from "shared/src/components/ui/badge";
import { getAdminInfo, logoutOAuth } from "shared/src/lib/session";
import { SidebarTrigger } from "shared/src/components/ui/sidebar";
import { Separator } from "shared/src/components/ui/separator";
import { Tooltip, TooltipContent, TooltipTrigger } from "shared/src/components/ui/tooltip";
import { Popover, PopoverContent, PopoverTrigger } from "shared/src/components/ui/popover";
import { ThemeToggle } from "shared/src/components/ui/theme-toggle";
import { TenantSelector } from "shared/src/components/TenantSelector";

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
  oauth: "OAuth 2.0",
  clients: "Clients",
  pats: "Personal Access Tokens",
  events: "Events & Streaming",
  feed: "Live Feed",
  dlq: "Dead Letter Queue",
  keystore: "Keystore",
  parameters: "Parameters",
  secrets: "Secrets",
  config: "Configuration",
  wallet: "Wallet",
  connections: "Connections",
  authority: "Authority",
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
export const Header = ({ onSignOut }: { onSignOut?: () => void } = {}) => {
  const routerState = useRouterState();
  const authCtx = useContext<AuthContextType | null>(AuthContext);
  const adminInfo = getAdminInfo();
  const [copied, setCopied] = React.useState(false);

  const handleCopySub = () => {
    if (adminInfo?.sub) {
      navigator.clipboard.writeText(adminInfo.sub);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  };

  const handleSignOut = async () => {
    await logoutOAuth();
    authCtx?.unsetAuthentication?.();
    onSignOut?.();
  };

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
    <div className="bg-background/80 backdrop-blur-md w-full border-b border-ink/10 z-40 h-12 px-4 flex justify-between items-center gap-4 sticky top-0 transition-colors">
      {/* Left section: Sidebar trigger and breadcrumbs */}
      <div className="flex items-center gap-2.5 overflow-hidden min-w-0">
        <SidebarTrigger className="h-7 w-7 shrink-0 text-muted-foreground hover:text-foreground" />
        <Separator orientation="vertical" className="h-4 shrink-0 bg-ink/10" />
        <Breadcrumb className="overflow-hidden">
          <BreadcrumbList className="flex-nowrap text-xs">{renderBreadcrumbs()}</BreadcrumbList>
        </Breadcrumb>
      </div>

      {/* Right section: Quick actions and user profile */}
      <div className="flex flex-row gap-3 shrink-0 items-center">
        <Link
          to="/events/feed"
          className="hidden sm:flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-mono border border-emerald-500/30 bg-emerald-500/10 text-emerald-700 dark:text-emerald-400 hover:bg-emerald-500/20 transition-colors"
          title="Event Bus Live Stream"
        >
          <span className="h-1.5 w-1.5 rounded-full bg-emerald-400 animate-pulse" />
          <span>Bus Live</span>
        </Link>

        <Link
          to="/events/feed"
          className="relative p-1 text-muted-foreground hover:text-foreground transition-colors"
        >
          <Bell className="h-4 w-4" />
        </Link>

        <TenantSelector />

        <ThemeToggle />

        <Popover>
          <PopoverTrigger asChild>
            <button className="flex items-center gap-2 pl-2 pr-2.5 py-1 rounded-full bg-ink/5 border border-ink/10 hover:border-ink/20 hover:bg-ink/10 text-foreground transition-all group">
              <div className="relative flex items-center justify-center h-6 w-6 rounded-full bg-primary-700 text-white font-bold text-xs shadow-sm">
                {(adminInfo?.name || adminInfo?.preferred_username || "A")
                  .slice(0, 2)
                  .toUpperCase()}
                <span className="absolute bottom-0 right-0 h-2 w-2 rounded-full bg-emerald-400 border border-background" />
              </div>
              <div className="hidden sm:flex flex-col text-left">
                <span className="text-xs font-medium text-foreground leading-none">
                  {adminInfo?.name || adminInfo?.preferred_username || "Operator"}
                </span>
                <span className="text-xs text-muted-foreground leading-tight">
                  {adminInfo?.role || "Admin"}
                </span>
              </div>
            </button>
          </PopoverTrigger>

          <PopoverContent
            align="end"
            sideOffset={8}
            className="w-80 p-0 z-[100] bg-background-800/95 backdrop-blur-xl border border-ink/15 shadow-2xl rounded-xl overflow-hidden"
          >
            {/* Header / Identity Banner */}
            <div className="p-3.5 bg-gradient-to-r from-primary-950/60 to-background-800 border-b border-ink/10 flex items-start gap-3">
              <div className="relative flex items-center justify-center h-10 w-10 rounded-xl bg-primary-700 text-white font-bold text-sm shadow-md shrink-0">
                {(adminInfo?.name || adminInfo?.preferred_username || "AD")
                  .slice(0, 2)
                  .toUpperCase()}
                <span className="absolute -bottom-0.5 -right-0.5 h-3 w-3 rounded-full bg-emerald-400 border-2 border-background-800" />
              </div>
              <div className="flex flex-col min-w-0 flex-1">
                <div className="flex items-center justify-between gap-1">
                  <span className="font-semibold text-sm text-foreground truncate">
                    {adminInfo?.name || adminInfo?.preferred_username || "Operator Admin"}
                  </span>
                  <Badge variant="role" dsrole="Business" size="xs">
                    {adminInfo?.role || "Admin"}
                  </Badge>
                </div>
                <span className="text-xs text-muted-foreground truncate">
                  {adminInfo?.email || "admin@eunomia.local"}
                </span>
                <div className="mt-1 flex items-center gap-1.5">
                  <Badge variant="success" size="xs" dot={true} className="text-xs">
                    OAuth2 / OIDC Active
                  </Badge>
                </div>
              </div>
            </div>

            {/* Admin Technical Information */}
            <div className="p-3.5 flex flex-col gap-2.5 text-xs border-b border-ink/10">
              <div className="flex flex-col gap-1">
                <span className="text-xs font-semibold text-muted-foreground uppercase tracking-wider">
                  Subject Identifier (sub)
                </span>
                <div className="flex items-center justify-between gap-2 p-1.5 rounded bg-sunken/40 border border-ink/5 font-mono text-xs text-brand-sky">
                  <span className="truncate">{adminInfo?.sub || "urn:eunomia:admin:local"}</span>
                  <button
                    onClick={handleCopySub}
                    className="text-muted-foreground hover:text-ink transition-colors shrink-0 p-0.5"
                    title="Copy subject identifier"
                  >
                    {copied ? (
                      <Check className="h-3.5 w-3.5 text-emerald-700 dark:text-emerald-400" />
                    ) : (
                      <Copy className="h-3.5 w-3.5" />
                    )}
                  </button>
                </div>
              </div>

              <div className="grid grid-cols-2 gap-2 pt-1 text-xs">
                <div>
                  <span className="text-muted-foreground text-xs block">Auth Protocol</span>
                  <span className="font-medium text-foreground">OAuth 2.0 / OIDC</span>
                </div>
                <div>
                  <span className="text-muted-foreground text-xs block">Token Type</span>
                  <span className="font-medium text-foreground">
                    {adminInfo?.token_type || "Bearer JWT"}
                  </span>
                </div>
              </div>

              {/* Scopes & Roles */}
              <div className="flex flex-col gap-1 pt-1">
                <span className="text-xs font-semibold text-muted-foreground uppercase tracking-wider">
                  Granted Scopes & Roles
                </span>
                <div className="flex flex-wrap gap-1">
                  {(adminInfo?.scope
                    ? adminInfo.scope.split(" ")
                    : ["openid", "profile", "email", "admin"]
                  ).map((s) => (
                    <span
                      key={s}
                      className="px-1.5 py-0.5 rounded bg-ink/5 border border-ink/10 text-xs font-mono text-muted-foreground"
                    >
                      {s}
                    </span>
                  ))}
                </div>
              </div>

              {adminInfo?.login_at && (
                <div className="text-xs text-muted-foreground/80 pt-1">
                  Session started: {new Date(adminInfo.login_at).toLocaleTimeString()}
                </div>
              )}
            </div>

            {/* Quick Links */}
            <div className="p-2 bg-ink/[0.02] border-b border-ink/10 flex flex-col gap-0.5">
              <Link
                to="/oauth/clients"
                className="flex items-center justify-between px-2.5 py-1.5 rounded-md text-xs text-foreground/80 hover:text-ink hover:bg-ink/5 transition-colors"
              >
                <span className="flex items-center gap-2">
                  <Shield className="h-3.5 w-3.5 text-brand-sky" />
                  OAuth Clients & Keys
                </span>
                <ExternalLink className="h-3 w-3 opacity-40" />
              </Link>
              <Link
                to="/oauth/pats"
                className="flex items-center justify-between px-2.5 py-1.5 rounded-md text-xs text-foreground/80 hover:text-ink hover:bg-ink/5 transition-colors"
              >
                <span className="flex items-center gap-2">
                  <Key className="h-3.5 w-3.5 text-secondary-700 dark:text-secondary-400" />
                  Personal Access Tokens
                </span>
                <ExternalLink className="h-3 w-3 opacity-40" />
              </Link>
              <Link
                to="/events/feed"
                className="flex items-center justify-between px-2.5 py-1.5 rounded-md text-xs text-foreground/80 hover:text-ink hover:bg-ink/5 transition-colors"
              >
                <span className="flex items-center gap-2">
                  <Radio className="h-3.5 w-3.5 text-emerald-700 dark:text-emerald-400" />
                  Live Event Bus
                </span>
                <ExternalLink className="h-3 w-3 opacity-40" />
              </Link>
            </div>

            {/* Sign out footer */}
            <div className="p-2 bg-sunken/20">
              <button
                onClick={handleSignOut}
                className="flex w-full items-center justify-center gap-2 rounded-lg px-3 py-2 text-xs font-semibold text-danger-700 dark:text-danger-300 hover:text-danger-800 dark:hover:text-danger-200 bg-danger-50 dark:bg-danger-950/30 hover:bg-danger-100 dark:hover:bg-danger-950/60 border border-danger-700/40 transition-all active:scale-[0.98]"
              >
                <LogOut className="h-3.5 w-3.5" />
                Sign out Operator
              </button>
            </div>
          </PopoverContent>
        </Popover>
      </div>
    </div>
  );
};
