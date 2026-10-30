import React, { useMemo, useState } from "react";
import { Check, Copy, LogOut } from "lucide-react";
import { useActivePath, useLink } from "../link";
import { decodePathId } from "../lib/utils";
import { type SessionUser, sessionDisplayName } from "../lib/session";
import { Identifier } from "../identity/Identifier";
import {
  Breadcrumb,
  BreadcrumbEllipsis,
  BreadcrumbItem,
  BreadcrumbLink,
  BreadcrumbList,
  BreadcrumbPage,
  BreadcrumbSeparator,
} from "../ui/breadcrumb";
import { Badge } from "../ui/badge";
import { SidebarTrigger } from "../ui/sidebar";
import { Separator } from "../ui/separator";
import { Tooltip, TooltipContent, TooltipTrigger } from "../ui/tooltip";
import { Popover, PopoverContent, PopoverTrigger } from "../ui/popover";
import { ThemeToggle } from "../ui/theme-toggle";

export interface BreadcrumbItemData {
  key: string;
  href: string;
  label: string;
  isDynamic: boolean;
}

export const DEFAULT_ROUTE_LABELS: Record<string, string> = {
  "datahub-catalog": "DataHub Catalog",
  "provider-catalog": "Provider Catalog",
  "contract-negotiation": "Negotiations",
  "transfer-process": "Transfers",
  "business-requests": "Business Requests",
  "customer-requests": "Customer Requests",
  subscriptions: "Subscriptions",
  participants: "Participants",
  agreements: "Agreements",
  catalog: "Catalog",
  dataset: "Dataset",
  dashboard: "Dashboard",
  home: "Home",
  about: "About",
  requests: "VC Requests",
  login: "Login",
  oauth: "OAuth 2.0",
  clients: "Clients",
  pats: "Personal Access Tokens",
  events: "Events",
  feed: "Feed",
  dlq: "Dead letters",
  keystore: "Keystore",
  parameters: "Parameters",
  secrets: "Secrets",
  config: "Configuration",
  wallet: "Wallet",
  connections: "Connections",
  authority: "VC Requests",
  did: "DID",
  keys: "Keys",
  credentials: "Credentials",
  info: "Info",
  oidc4vp: "OID4VP",
  oidc4vci: "OID4VCI",
  "request-details": "Request",
};

const DEFAULT_ENCODED_ID_PARENTS = new Set(["participants", "participant"]);
const DEFAULT_HIDDEN_SEGMENTS = new Set(["data-service"]);
const DEFAULT_MERGE_WITH_NEXT = new Set(["dataset", "transfer-message", "cn-message"]);

function formatSegmentLabel(segment: string, labels: Record<string, string>): string {
  if (labels[segment]) {
    return labels[segment];
  }
  if (segment.startsWith("urn:") || segment.startsWith("did:")) {
    return Identifier.short(segment);
  }
  return segment
    .split("-")
    .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
    .join(" ");
}

function isDynamicSegment(segment: string): boolean {
  return (
    segment.includes("urn:") ||
    segment.includes("did:") ||
    /^[0-9a-f]{8}-[0-9a-f]{4}-/i.test(segment) ||
    segment.startsWith("$")
  );
}

function generateBreadcrumbs(
  pathname: string,
  labels: Record<string, string>,
  rootBreadcrumb?: { label: string; href: string },
): BreadcrumbItemData[] {
  const segments = pathname.split("/").filter((s) => s !== "");
  const items: BreadcrumbItemData[] = [];
  if (rootBreadcrumb) {
    items.push({
      key: "__root__",
      href: rootBreadcrumb.href,
      label: rootBreadcrumb.label,
      isDynamic: false,
    });
  }

  let currentPath = "";
  for (let i = 0; i < segments.length; i++) {
    const segment = decodeURIComponent(segments[i]);
    currentPath += `/${segments[i]}`;

    if (DEFAULT_HIDDEN_SEGMENTS.has(segment)) {
      continue;
    }

    const nextSegment = segments[i + 1] ? decodeURIComponent(segments[i + 1]) : null;
    if (DEFAULT_MERGE_WITH_NEXT.has(segment) && nextSegment) {
      continue;
    }

    const prevSegment = segments[i - 1] ? decodeURIComponent(segments[i - 1]) : null;
    let label = formatSegmentLabel(segment, labels);
    if (prevSegment && DEFAULT_ENCODED_ID_PARENTS.has(prevSegment)) {
      try {
        label = Identifier.short(decodePathId(segment));
      } catch {
        // An id not base64url-encoded: format normally
        if (segment.startsWith("did:") || segment.startsWith("urn:")) {
          label = Identifier.short(segment);
        }
      }
    }
    if (prevSegment && DEFAULT_MERGE_WITH_NEXT.has(prevSegment)) {
      const prevLabel = formatSegmentLabel(prevSegment, labels);
      label = `${prevLabel}: ${formatSegmentLabel(segment, labels)}`;
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

export interface AppShellHeaderProps {
  user?: SessionUser | null;
  onSignOut?: () => void;
  activePath?: string;
  routeLabels?: Record<string, string>;
  rootBreadcrumb?: { label: string; href: string };
  /** Extra action widgets rendered left of ThemeToggle (e.g., NotificationBell) */
  rightActionsSlot?: React.ReactNode;
  /** Optional quick links rendered inside the user popover */
  quickLinksSlot?: React.ReactNode;
}

export function AppShellHeader({
  user,
  onSignOut,
  activePath,
  routeLabels,
  rootBreadcrumb,
  rightActionsSlot,
  quickLinksSlot,
}: AppShellHeaderProps) {
  const Link = useLink();
  const ctxPath = useActivePath();
  const pathname = activePath ?? ctxPath;
  const [copied, setCopied] = useState(false);

  const labels = useMemo(
    () => (routeLabels ? { ...DEFAULT_ROUTE_LABELS, ...routeLabels } : DEFAULT_ROUTE_LABELS),
    [routeLabels],
  );

  const displayName = sessionDisplayName(user ?? null) || "Operator";
  const roleLabel = user ? (user.root ? "Root" : user.role) : "Console";

  const handleCopySub = () => {
    if (user?.userId) {
      navigator.clipboard.writeText(user.userId);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  };

  const breadcrumbs = useMemo(
    () => generateBreadcrumbs(pathname, labels, rootBreadcrumb),
    [pathname, labels, rootBreadcrumb],
  );

  const renderBreadcrumbs = () => {
    if (breadcrumbs.length === 0) return null;

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

    const firstItem = breadcrumbs[0];
    const middleItems = breadcrumbs.slice(1, -1);
    const lastItem = breadcrumbs[breadcrumbs.length - 1];

    return (
      <>
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

        <BreadcrumbItem className="truncate max-w-[200px]">
          <BreadcrumbPage className="truncate font-medium">{lastItem.label}</BreadcrumbPage>
        </BreadcrumbItem>
      </>
    );
  };

  return (
    <div className="bg-background/80 backdrop-blur-md w-full border-b border-ink/10 z-40 h-12 px-4 flex justify-between items-center gap-4 sticky top-0 transition-colors">
      <div className="flex items-center gap-2.5 overflow-hidden min-w-0">
        <SidebarTrigger className="h-7 w-7 shrink-0 text-muted-foreground hover:text-foreground" />
        <Separator orientation="vertical" className="h-4 shrink-0 bg-ink/10" />
        <Breadcrumb className="overflow-hidden">
          <BreadcrumbList className="flex-nowrap text-xs">{renderBreadcrumbs()}</BreadcrumbList>
        </Breadcrumb>
      </div>

      <div className="flex flex-row gap-3 shrink-0 items-center">
        {rightActionsSlot}

        <ThemeToggle />

        <Popover>
          <PopoverTrigger asChild>
            <button
              type="button"
              className="flex items-center gap-2 pl-2 pr-2.5 py-1 rounded-full bg-ink/5 border border-ink/10 hover:border-ink/20 hover:bg-ink/10 text-foreground transition-all group"
            >
              <div className="relative flex items-center justify-center h-6 w-6 rounded-full bg-primary-700 text-white font-bold text-xs shadow-sm">
                {displayName.slice(0, 2).toUpperCase()}
                <span className="absolute bottom-0 right-0 h-2 w-2 rounded-full bg-emerald-400 border border-background" />
              </div>
              <div className="hidden sm:flex flex-col text-left">
                <span className="text-xs font-medium text-foreground leading-none">
                  {displayName}
                </span>
                <span className="text-xs text-muted-foreground leading-tight">{roleLabel}</span>
              </div>
            </button>
          </PopoverTrigger>

          <PopoverContent
            align="end"
            sideOffset={8}
            className="w-80 p-0 z-[100] bg-background-800/95 backdrop-blur-xl border border-ink/15 shadow-2xl rounded-xl overflow-hidden"
          >
            <div className="p-3.5 bg-gradient-to-r from-primary-950/60 to-background-800 border-b border-ink/10 flex items-start gap-3">
              <div className="relative flex items-center justify-center h-10 w-10 rounded-xl bg-primary-700 text-white font-bold text-sm shadow-md shrink-0">
                {displayName.slice(0, 2).toUpperCase()}
                <span className="absolute -bottom-0.5 -right-0.5 h-3 w-3 rounded-full bg-emerald-400 border-2 border-background-800" />
              </div>
              <div className="flex flex-col min-w-0 flex-1">
                <div className="flex items-center justify-between gap-1">
                  <span className="font-semibold text-sm text-foreground truncate">
                    {displayName}
                  </span>
                  <Badge variant="role" dsrole="Business" size="xs">
                    {roleLabel}
                  </Badge>
                </div>
                {user?.email && (
                  <span className="text-xs text-muted-foreground truncate">{user.email}</span>
                )}
                <div className="mt-1 flex items-center gap-1.5">
                  <Badge variant="success" size="xs" dot={true} className="text-xs">
                    Session active
                  </Badge>
                </div>
              </div>
            </div>

            {user && (
              <div className="p-3.5 flex flex-col gap-2.5 text-xs border-b border-ink/10">
                <div className="flex flex-col gap-1">
                  <span className="text-xs font-semibold text-muted-foreground uppercase tracking-wider">
                    User id
                  </span>
                  <div className="flex items-center justify-between gap-2 p-1.5 rounded bg-sunken/40 border border-ink/5 font-mono text-xs text-brand-sky">
                    <span className="truncate">{user.userId}</span>
                    <button
                      type="button"
                      onClick={handleCopySub}
                      className="text-muted-foreground hover:text-ink transition-colors shrink-0 p-0.5"
                      title="Copy user id"
                    >
                      {copied ? (
                        <Check className="h-3.5 w-3.5 text-emerald-700 dark:text-emerald-400" />
                      ) : (
                        <Copy className="h-3.5 w-3.5" />
                      )}
                    </button>
                  </div>
                </div>

                <div className="flex flex-col gap-1 pt-1">
                  <span className="text-xs font-semibold text-muted-foreground uppercase tracking-wider">
                    Role
                  </span>
                  <span className="px-1.5 py-0.5 rounded bg-ink/5 border border-ink/10 text-xs font-mono text-muted-foreground w-fit">
                    {user.role}
                  </span>
                </div>
              </div>
            )}

            {quickLinksSlot && (
              <div className="p-2 bg-ink/[0.02] border-b border-ink/10 flex flex-col gap-0.5">
                {quickLinksSlot}
              </div>
            )}

            {onSignOut && (
              <div className="p-2 bg-sunken/20">
                <button
                  type="button"
                  onClick={() => onSignOut()}
                  className="flex w-full items-center justify-center gap-2 rounded-lg px-3 py-2 text-xs font-semibold text-danger-700 dark:text-danger-300 hover:text-danger-800 dark:hover:text-danger-200 bg-danger-50 dark:bg-danger-950/30 hover:bg-danger-100 dark:hover:bg-danger-950/60 border border-danger-700/40 transition-all active:scale-[0.98]"
                >
                  <LogOut className="h-3.5 w-3.5" />
                  Sign out
                </button>
              </div>
            )}
          </PopoverContent>
        </Popover>
      </div>
    </div>
  );
}
