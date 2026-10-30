import {
  Identifier,
  cn
} from "./chunk-4GNLXNMN.js";

// src/wallet/WalletLayout.tsx
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Loader2 as Loader22, RefreshCw, Wallet } from "lucide-react";
import { toast } from "sonner";

// src/lib/http.ts
var UNAUTHORIZED_EVENT = "eunomia:unauthorized";
var API_GATEWAY_BASE = "";
var setApiGatewayBase = (url) => {
  API_GATEWAY_BASE = url;
};
var getApiGatewayBase = () => API_GATEWAY_BASE;
var customInstance = async (url, options) => {
  const { method, headers, params, data, _noSessionRedirect, ...rest } = options || {};
  const normalizedHeaders = {
    "Content-Type": "application/json"
  };
  if (headers) {
    if (headers instanceof Headers) {
      headers.forEach((v, k) => {
        normalizedHeaders[k] = v;
      });
    } else if (Array.isArray(headers)) {
      headers.forEach(([k, v]) => {
        normalizedHeaders[k] = v;
      });
    } else {
      Object.entries(headers).forEach(([k, v]) => {
        if (v !== void 0) {
          normalizedHeaders[k] = Array.isArray(v) ? v.join(", ") : String(v);
        }
      });
    }
  }
  const config = {
    ...rest,
    headers: normalizedHeaders
  };
  let targetUrl = url;
  if (params) {
    const searchParams = new URLSearchParams();
    Object.entries(params).forEach(([key, value]) => {
      if (value !== void 0 && value !== null) {
        if (Array.isArray(value)) {
          value.forEach((v) => searchParams.append(key, String(v)));
        } else {
          searchParams.append(key, String(value));
        }
      }
    });
    const queryString = searchParams.toString();
    if (queryString) {
      targetUrl += (targetUrl.includes("?") ? "&" : "?") + queryString;
    }
  }
  const fullUrl = API_GATEWAY_BASE ? `${API_GATEWAY_BASE}${targetUrl}` : targetUrl;
  const requestMethod = method || "GET";
  const fetchOptions = {
    ...config,
    method: requestMethod
  };
  if (data !== void 0) {
    fetchOptions.body = JSON.stringify(data);
  }
  const response = await fetch(fullUrl, fetchOptions);
  let parsedData;
  if (response.status === 204) {
    parsedData = {};
  } else {
    try {
      parsedData = await response.clone().json();
    } catch {
      parsedData = await response.text();
    }
  }
  if (response.status === 401 && !_noSessionRedirect && typeof window !== "undefined") {
    window.dispatchEvent(new Event(UNAUTHORIZED_EVENT));
  }
  if (parsedData && typeof parsedData === "object" && !Array.isArray(parsedData) && Array.isArray(parsedData.items)) {
    const envelope = parsedData;
    const arr = [...envelope.items];
    arr.items = envelope.items;
    arr.total = envelope.total;
    arr.nextCursor = envelope.nextCursor;
    parsedData = arr;
  }
  return {
    status: response.status,
    data: parsedData,
    headers: response.headers
  };
};

// src/layout/PageLayout.tsx
import { jsx } from "react/jsx-runtime";
function PageLayout({ className, children, ...props }) {
  return /* @__PURE__ */ jsx("div", { className: cn("space-y-4 pb-8 px-3 w-full", className), ...props, children });
}

// src/ui/heading.tsx
import { jsx as jsx2 } from "react/jsx-runtime";
var levelToElement = {
  h1: "h1",
  h2: "h2",
  h3: "h3",
  h4: "h4",
  h5: "h5",
  h6: "h6",
  table: "h6",
  "title-sm": "h6",
  subtitle: "h5"
};
var sizeClasses = {
  h1: "text-2xl mb-4 font-semibold font-title tracking-tight",
  h2: "text-xl mb-3 font-semibold tracking-tight",
  h3: "text-lg mb-3 font-medium tracking-tight",
  h4: "text-base mb-2 font-medium font-display tracking-normal text-ink/90",
  h5: "text-sm text-ink/80 mb-2 font-medium tracking-wide",
  h6: "text-xs font-semibold mb-1 uppercase tracking-wider text-muted-foreground",
  table: "text-xs font-semibold uppercase tracking-wider",
  "title-sm": "text-sm font-medium mb-1 leading-snug",
  subtitle: "text-base mb-2 font-normal text-muted-foreground max-w-[65ch]"
};
var Heading = ({ level = "h1", children, ref, className = "" }) => {
  const Component = levelToElement[level] || "h1";
  const baseClasses = "text-foreground-100 text-balance";
  return /* @__PURE__ */ jsx2(Component, { className: cn(baseClasses, sizeClasses[level], className), children });
};
var heading_default = Heading;

// src/layout/PageHeader.tsx
import { jsx as jsx3, jsxs } from "react/jsx-runtime";
function PageHeader({
  title,
  badge,
  eyebrow,
  description,
  actions,
  className,
  children,
  ...props
}) {
  return /* @__PURE__ */ jsxs(
    "header",
    {
      className: cn(
        "pt-4 mb-5 pb-4 border-b border-ink/10 sticky top-12 z-30 bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/80 transition-colors",
        className
      ),
      ...props,
      children: [
        /* @__PURE__ */ jsxs("div", { className: "flex flex-wrap items-start justify-between gap-x-6 gap-y-3", children: [
          /* @__PURE__ */ jsxs("div", { className: "min-w-0 flex-1", children: [
            eyebrow && /* @__PURE__ */ jsx3("div", { className: "mb-1 text-xs font-medium uppercase tracking-wide text-ink/50", children: eyebrow }),
            /* @__PURE__ */ jsxs(
              heading_default,
              {
                level: "h1",
                className: "flex flex-wrap gap-2 items-center font-display text-2xl mb-0 text-ink/90",
                children: [
                  /* @__PURE__ */ jsx3("span", { className: "min-w-0 break-words", children: title }),
                  badge
                ]
              }
            ),
            description && /* @__PURE__ */ jsx3("p", { className: "mt-1 text-sm text-ink/60", children: description })
          ] }),
          actions && /* @__PURE__ */ jsx3("div", { className: "flex shrink-0 flex-wrap items-center gap-2", children: actions })
        ] }),
        children
      ]
    }
  );
}

// src/link.tsx
import React, { createContext, useContext } from "react";
import { jsx as jsx4 } from "react/jsx-runtime";
var DefaultLink = React.forwardRef(({ to, ...rest }, ref) => /* @__PURE__ */ jsx4("a", { ref, href: to, ...rest }));
DefaultLink.displayName = "DefaultLink";
var LinkContext = createContext(DefaultLink);
var ActivePathContext = createContext("");
var LinkProvider = LinkContext.Provider;
var ActivePathProvider = ActivePathContext.Provider;
var useLink = () => useContext(LinkContext);
var useActivePath = () => {
  const ctxPath = useContext(ActivePathContext);
  if (ctxPath) return ctxPath;
  if (typeof window !== "undefined") return window.location.pathname;
  return "";
};
function NavigationProvider({ link, activePath = "", children }) {
  return /* @__PURE__ */ jsx4(LinkContext.Provider, { value: link, children: /* @__PURE__ */ jsx4(ActivePathContext.Provider, { value: activePath, children }) });
}

// src/ui/count-badge.tsx
import { jsx as jsx5 } from "react/jsx-runtime";
function CountBadge({
  n,
  showZero = false,
  className
}) {
  if (n === void 0 || n === 0 && !showZero) return null;
  return /* @__PURE__ */ jsx5(
    "span",
    {
      className: cn("rounded-full bg-ink/10 px-1.5 text-xs font-medium tabular-nums", className),
      children: n
    }
  );
}

// src/layout/RouteTabs.tsx
import { jsx as jsx6, jsxs as jsxs2 } from "react/jsx-runtime";
var tabListClass = "flex w-full items-end gap-1 border-b border-ink/10 overflow-x-auto";
var tabTriggerClass = cn(
  "relative -mb-px inline-flex items-center gap-2 whitespace-nowrap rounded-t-md px-4 py-2.5",
  "text-sm font-medium text-ink/55 border-b-2 border-transparent transition-colors",
  "hover:text-ink/90 hover:bg-ink/[0.03]",
  "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-brand-sky/50"
);
var tabActiveClass = "text-ink font-semibold border-brand-sky bg-brand-sky/[0.08]";
function RouteTabs({
  tabs,
  activePath,
  className
}) {
  const Link = useLink();
  const ctxPath = useActivePath();
  const pathname = (activePath ?? ctxPath).replace(/\/$/, "");
  return /* @__PURE__ */ jsx6("nav", { className: cn(tabListClass, "mb-6", className), role: "tablist", children: tabs.map(({ label, to, icon: Icon2, count }) => {
    const cleanTo = to.replace(/\/$/, "");
    const active = pathname.endsWith(cleanTo) || pathname.includes(`${cleanTo}/`);
    return /* @__PURE__ */ jsxs2(
      Link,
      {
        to,
        role: "tab",
        "aria-selected": active,
        className: cn(tabTriggerClass, active && tabActiveClass),
        children: [
          Icon2 && /* @__PURE__ */ jsx6(Icon2, { className: "h-4 w-4" }),
          label,
          /* @__PURE__ */ jsx6(CountBadge, { n: count, showZero: true })
        ]
      },
      to
    );
  }) });
}

// src/ui/button.tsx
import * as React2 from "react";
import { Loader2 } from "lucide-react";
import { Slot } from "@radix-ui/react-slot";
import { cva } from "class-variance-authority";
import { Fragment, jsx as jsx7, jsxs as jsxs3 } from "react/jsx-runtime";
var buttonVariants = cva(
  "inline-flex items-center flex-nowrap min-w-fit !leading-none justify-center gap-2 whitespace-nowrap rounded-md text-sm font-medium transition-all focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-brand-sky/40 focus-visible:ring-offset-1 focus-visible:ring-offset-background disabled:pointer-events-none disabled:opacity-40 disabled:cursor-not-allowed [&_svg]:pointer-events-none [&_svg]:size-4 [&_svg]:shrink-0",
  {
    variants: {
      variant: {
        default: "bg-primary-700 text-white hover:bg-primary-600 border border-primary-500/30 hover:border-primary-400/50 shadow-sm active:scale-[0.98]",
        destructive: "border border-danger-700/50 bg-danger-50 dark:bg-danger-950/40 text-danger-700 dark:text-danger-300 hover:bg-danger-100 dark:hover:bg-danger-900/60 hover:text-danger-800 dark:hover:text-danger-100 hover:border-danger-600/70 shadow-sm active:scale-[0.98]",
        outline: "border border-ink/20 text-ink/90 bg-transparent hover:bg-ink/10 hover:text-ink hover:border-ink/30 shadow-sm active:scale-[0.98]",
        outline_destructive: "border border-danger-700/60 text-danger-700 dark:text-danger-300 bg-transparent hover:bg-danger-50 dark:bg-danger-950/40 hover:text-danger-800 dark:hover:text-danger-200 hover:border-danger-600 active:scale-[0.98]",
        secondary: "bg-ink/10 text-ink hover:bg-ink/15 border border-ink/15 hover:border-ink/25 shadow-sm active:scale-[0.98]",
        ghost: "text-ink/80 hover:text-ink hover:bg-ink/10 active:scale-[0.98]",
        success: "border border-success-700/50 bg-success-50 dark:bg-success-950/40 text-success-700 dark:text-success-300 hover:bg-success-100 dark:hover:bg-success-900/60 hover:text-success-800 dark:hover:text-success-100 hover:border-success-600/70 shadow-sm active:scale-[0.98]",
        icon_destructive: "flex text-danger-700 dark:text-danger-400 border border-danger-700/40 bg-danger-50 dark:bg-danger-950/20 hover:bg-danger-100 dark:hover:bg-danger-900/50 hover:text-danger-800 dark:hover:text-danger-200 active:scale-[0.98] p-1.5",
        link: "!px-0 flex-nowrap normal-case text-brand-sky underline-offset-4 hover:underline hover:text-ink active:scale-100 transition-colors"
      },
      size: {
        default: "h-9 px-4",
        sm: "h-8 px-3 text-[0.8125rem]",
        xs: "h-7 px-2.5 text-xs [&_svg]:size-3.5",
        lg: "h-10 px-6",
        icon: "h-9 w-9 p-0",
        icon_sm: "h-8 w-8 p-0 [&_svg]:size-3.5"
      },
      policy: {
        default: "",
        permission: "bg-success-600/20 border-success-700 text-success-800 dark:text-success-100 hover:bg-success-600/30",
        obligation: "bg-warn-700/20 border-warn-700 text-warn-800 dark:text-warn-100 hover:bg-warn-700/30",
        prohibition: "bg-danger-600/20 border-danger-700 text-danger-800 dark:text-danger-100 hover:bg-danger-600/30"
      }
    },
    defaultVariants: {
      variant: "default",
      size: "default",
      policy: "default"
    }
  }
);
var Button = React2.forwardRef(
  ({ className, variant, size, policy, asChild = false, isLoading, ...props }, ref) => {
    const Comp = asChild ? Slot : "button";
    return /* @__PURE__ */ jsx7(
      Comp,
      {
        className: cn(buttonVariants({ variant, size, policy, className })),
        ref,
        disabled: isLoading || props.disabled,
        ...props,
        children: asChild ? props.children : /* @__PURE__ */ jsxs3(Fragment, { children: [
          isLoading && /* @__PURE__ */ jsx7(Loader2, { className: "animate-spin" }),
          props.children
        ] })
      }
    );
  }
);
Button.displayName = "Button";

// src/wallet/WalletLayout.tsx
import { Fragment as Fragment2, jsx as jsx8, jsxs as jsxs4 } from "react/jsx-runtime";
function WalletLayout({
  basePath = "/wallet",
  onboarded: controlledOnboarded,
  onLink: controlledOnLink,
  isLinking: controlledIsLinking,
  children
}) {
  const queryClient = useQueryClient();
  const isControlled = controlledOnboarded !== void 0;
  const { data: linkedRes, isLoading: checkingLink } = useQuery({
    queryKey: ["wallet-is-linked"],
    queryFn: () => customInstance(
      "/wallet/is-linked",
      { method: "GET" }
    ),
    enabled: !isControlled
  });
  const linkMutation = useMutation({
    mutationFn: () => customInstance("/wallet/link", { method: "POST" }),
    onSuccess: async () => {
      toast.success("Wallet linked");
      await queryClient.invalidateQueries({ queryKey: ["wallet-is-linked"] });
      await queryClient.invalidateQueries({ queryKey: ["wallet-info-custom"] });
    },
    onError: (err) => {
      toast.error(err instanceof Error ? err.message : "Failed to link wallet");
    }
  });
  const rawData = linkedRes?.data;
  const autoLinked = linkedRes?.status === 200 && (typeof rawData === "boolean" ? rawData : typeof rawData === "object" && rawData !== null && "linked" in rawData ? Boolean(rawData.linked) : true);
  const ownWalletOnboarded = isControlled ? controlledOnboarded : autoLinked;
  const isLinking = isControlled ? Boolean(controlledIsLinking) : linkMutation.isPending;
  const handleLink = () => {
    if (controlledOnLink) {
      void controlledOnLink();
    } else {
      linkMutation.mutate();
    }
  };
  const cleanBase = basePath.replace(/\/$/, "");
  const tabs = [
    { label: "Info", to: `${cleanBase}/info` },
    { label: "DID", to: `${cleanBase}/did` },
    { label: "Keys", to: `${cleanBase}/keys` },
    { label: "Credentials", to: `${cleanBase}/credentials` },
    { label: "OID4VP", to: `${cleanBase}/oidc4vp` },
    { label: "OID4VCI", to: `${cleanBase}/oidc4vci` }
  ];
  return /* @__PURE__ */ jsxs4(PageLayout, { children: [
    /* @__PURE__ */ jsx8(
      PageHeader,
      {
        title: "Wallet",
        description: "Decentralized identity: DIDs, keys and verifiable credentials.",
        actions: ownWalletOnboarded && /* @__PURE__ */ jsxs4(Button, { onClick: handleLink, isLoading: isLinking, variant: "outline", size: "sm", children: [
          !isLinking && /* @__PURE__ */ jsx8(RefreshCw, {}),
          isLinking ? "Relinking\u2026" : "Relink wallet"
        ] })
      }
    ),
    !isControlled && checkingLink ? /* @__PURE__ */ jsx8("div", { className: "flex items-center justify-center min-h-[300px]", children: /* @__PURE__ */ jsx8(Loader22, { className: "h-6 w-6 animate-spin text-brand-sky" }) }) : !ownWalletOnboarded ? /* @__PURE__ */ jsxs4("div", { className: "flex flex-col items-center justify-center min-h-[400px] border border-dashed border-ink/10 rounded-xl bg-ink/5 p-12", children: [
      /* @__PURE__ */ jsx8(Wallet, { className: "h-12 w-12 text-muted-foreground/40 mb-4" }),
      /* @__PURE__ */ jsx8("p", { className: "text-muted-foreground font-medium text-lg", children: "Wallet not linked" }),
      /* @__PURE__ */ jsx8("p", { className: "text-muted-foreground/60 text-sm mt-1 mb-6", children: "Link your wallet to start managing your identity." }),
      /* @__PURE__ */ jsx8(Button, { onClick: handleLink, isLoading: isLinking, children: "Link wallet" })
    ] }) : /* @__PURE__ */ jsxs4(Fragment2, { children: [
      /* @__PURE__ */ jsx8(RouteTabs, { tabs }),
      children
    ] })
  ] });
}

// src/wallet/WalletInfoView.tsx
import { useQuery as useQuery2 } from "@tanstack/react-query";

// src/layout/PageSection.tsx
import { jsx as jsx9, jsxs as jsxs5 } from "react/jsx-runtime";
function PageSection({ title, action, className, children, ...props }) {
  return /* @__PURE__ */ jsxs5("div", { className: cn("mb-6", className), ...props, children: [
    /* @__PURE__ */ jsxs5("div", { className: "flex items-center justify-between gap-4 mb-2", children: [
      title && /* @__PURE__ */ jsx9(
        heading_default,
        {
          level: "h6",
          className: "mt-2 text-ink/80 uppercase tracking-wide text-xs font-semibold",
          children: title
        }
      ),
      action && /* @__PURE__ */ jsx9("div", { className: "mt-1 ml-auto", children: action })
    ] }),
    children
  ] });
}

// src/layout/FieldGrid.tsx
import { jsx as jsx10, jsxs as jsxs6 } from "react/jsx-runtime";
var COLS = {
  1: "",
  2: "md:grid-cols-2",
  3: "md:grid-cols-3",
  4: "md:grid-cols-2 xl:grid-cols-4"
};
var SPAN = { 1: "", 2: "md:col-span-2", 3: "md:col-span-3", 4: "md:col-span-full" };
function FieldGrid({
  fields,
  columns = 3,
  className
}) {
  const visible = fields.filter((f) => !f.hidden);
  return (
    // Each cell draws its right and bottom border; the frame clips the outer ones.
    /* @__PURE__ */ jsx10("div", { className: cn("overflow-hidden rounded-lg border border-ink/10", className), children: /* @__PURE__ */ jsx10("dl", { className: cn("-mb-px -mr-px grid grid-cols-1", COLS[Math.min(columns, visible.length)]), children: visible.map((f) => /* @__PURE__ */ jsxs6(
      "div",
      {
        className: cn("min-w-0 border-b border-r border-ink/10 px-4 py-3", SPAN[f.span ?? 1]),
        children: [
          /* @__PURE__ */ jsx10("dt", { className: "mb-1 text-xs font-medium text-ink/50", children: f.label }),
          /* @__PURE__ */ jsx10("dd", { className: "min-w-0 text-sm text-ink/90", children: f.value ?? /* @__PURE__ */ jsx10("span", { className: "text-ink/35", children: "\u2014" }) })
        ]
      },
      f.label
    )) }) })
  );
}

// src/table/DataTable.tsx
import { useState as useState2, useMemo as useMemo2, useEffect as useEffect2, useCallback as useCallback2 } from "react";

// src/ui/table.tsx
import * as React3 from "react";
import { jsx as jsx11 } from "react/jsx-runtime";
var Table = React3.forwardRef(({ className, containerClassName, ...props }, ref) => /* @__PURE__ */ jsx11(
  "div",
  {
    className: cn(
      "relative w-full overflow-auto rounded-xl border border-ink/10 bg-background-800/50 backdrop-blur-sm shadow-sm",
      containerClassName
    ),
    children: /* @__PURE__ */ jsx11(
      "table",
      {
        ref,
        className: cn("w-full caption-bottom text-xs text-foreground-200 border-collapse", className),
        ...props
      }
    )
  }
));
Table.displayName = "Table";
var TableHeader = React3.forwardRef(({ className, ...props }, ref) => /* @__PURE__ */ jsx11(
  "thead",
  {
    ref,
    className: cn("bg-ink/[0.03] border-b border-ink/10", className),
    ...props
  }
));
TableHeader.displayName = "TableHeader";
var TableBody = React3.forwardRef(({ className, ...props }, ref) => /* @__PURE__ */ jsx11(
  "tbody",
  {
    ref,
    className: cn("[&_tr:last-child]:border-0 divide-y divide-ink/[0.05]", className),
    ...props
  }
));
TableBody.displayName = "TableBody";
var TableFooter = React3.forwardRef(({ className, ...props }, ref) => /* @__PURE__ */ jsx11(
  "tfoot",
  {
    ref,
    className: cn("border-t border-ink/10 bg-ink/[0.02] font-medium text-xs", className),
    ...props
  }
));
TableFooter.displayName = "TableFooter";
var TableRow = React3.forwardRef(
  ({ className, ...props }, ref) => /* @__PURE__ */ jsx11(
    "tr",
    {
      ref,
      className: cn(
        "border-b border-ink/[0.06] transition-colors hover:bg-ink/[0.03] data-[state=selected]:bg-ink/[0.06]",
        className
      ),
      ...props
    }
  )
);
TableRow.displayName = "TableRow";
var TableHead = React3.forwardRef(({ className, ...props }, ref) => /* @__PURE__ */ jsx11(
  "th",
  {
    ref,
    className: cn(
      "h-9 px-3.5 text-left align-middle font-semibold text-muted-foreground uppercase tracking-wider text-xs select-none",
      className
    ),
    ...props
  }
));
TableHead.displayName = "TableHead";
var TableCell = React3.forwardRef(({ className, ...props }, ref) => /* @__PURE__ */ jsx11(
  "td",
  {
    ref,
    className: cn("px-3.5 py-3 align-middle text-xs text-foreground-200", className),
    ...props
  }
));
TableCell.displayName = "TableCell";
var TableCaption = React3.forwardRef(({ className, ...props }, ref) => /* @__PURE__ */ jsx11("caption", { ref, className: cn("mt-4 text-xs text-muted-foreground", className), ...props }));
TableCaption.displayName = "TableCaption";

// src/ui/input.tsx
import * as React4 from "react";
import { Search, X } from "lucide-react";
import { jsx as jsx12, jsxs as jsxs7 } from "react/jsx-runtime";
var Input = React4.forwardRef(
  ({
    className,
    containerClassName,
    type = "text",
    placeholder,
    startIcon,
    endIcon,
    onClear,
    value,
    ...props
  }, ref) => {
    const isSearch = type === "search";
    const hasDecorator = Boolean(startIcon || endIcon || onClear || isSearch);
    const baseInputStyles = "w-full rounded-md border border-ink/15 bg-background-800/90 px-3 py-1.5 text-xs text-brand-snow placeholder:text-muted-foreground/60 transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-brand-sky/60 focus-visible:border-brand-sky/60 disabled:cursor-not-allowed disabled:opacity-50 [color-scheme:dark]";
    if (!hasDecorator) {
      return /* @__PURE__ */ jsx12(
        "input",
        {
          ...props,
          ref,
          type,
          value,
          placeholder,
          className: cn("flex h-9", baseInputStyles, className)
        }
      );
    }
    const defaultPlaceholder = isSearch ? "Search..." : void 0;
    const resolvedPlaceholder = placeholder !== void 0 ? placeholder : defaultPlaceholder;
    return /* @__PURE__ */ jsxs7(
      "div",
      {
        className: cn(
          "flex h-9 w-full items-center rounded-md border border-ink/15 bg-background-800/90 px-2.5 text-xs text-brand-snow transition-all focus-within:border-brand-sky/60 focus-within:ring-1 focus-within:ring-brand-sky/60 hover:border-ink/25",
          containerClassName
        ),
        children: [
          startIcon ? /* @__PURE__ */ jsx12("span", { className: "mr-2 flex items-center text-muted-foreground/70 shrink-0", children: startIcon }) : isSearch ? /* @__PURE__ */ jsx12(Search, { className: "mr-2 h-3.5 w-3.5 text-muted-foreground/70 shrink-0" }) : null,
          /* @__PURE__ */ jsx12(
            "input",
            {
              ...props,
              ref,
              type,
              value,
              placeholder: resolvedPlaceholder,
              className: cn(
                "w-full bg-transparent text-xs text-brand-snow placeholder:text-muted-foreground/60 focus:outline-none disabled:cursor-not-allowed disabled:opacity-50 [color-scheme:dark]",
                className
              )
            }
          ),
          onClear && value && String(value).length > 0 ? /* @__PURE__ */ jsx12(
            "button",
            {
              type: "button",
              onClick: onClear,
              className: "ml-1 text-muted-foreground/60 hover:text-ink transition-colors shrink-0",
              children: /* @__PURE__ */ jsx12(X, { className: "h-3.5 w-3.5" })
            }
          ) : endIcon ? /* @__PURE__ */ jsx12("span", { className: "ml-2 flex items-center text-muted-foreground/70 shrink-0", children: endIcon }) : null
        ]
      }
    );
  }
);
Input.displayName = "Input";

// src/ui/badge.tsx
import { Slot as Slot2 } from "@radix-ui/react-slot";
import { cva as cva2 } from "class-variance-authority";
import { jsx as jsx13, jsxs as jsxs8 } from "react/jsx-runtime";
var normalizeStatus = (status) => {
  switch ((status || "").toLowerCase()) {
    case "active":
    case "accepted":
    case "verified":
    case "started":
    case "approved":
    case "agreed":
    case "in_progress":
    case "running":
    case "valid":
      return "process";
    case "offered":
    case "requested":
    case "pending":
    case "processing":
    case "waiting":
    case "expiring":
    case "warning":
      return "warn";
    case "finalized":
    case "completed":
    case "success":
    case "confirmed":
    case "delivered":
    case "delivered_confirmed":
    case "ok":
      return "success";
    case "inactive":
    case "suspended":
    case "pause":
    case "paused":
    case "by_provider":
    case "by_consumer":
    case "on_request":
    case "stop":
    case "stopped":
    case "idle":
      return "pause";
    case "terminated":
    case "rejected":
    case "failed":
    case "error":
    case "revoked":
    case "dead_letter":
    case "dlq":
    case "expired":
      return "danger";
    default:
      return "default";
  }
};
var badgeVariants = cva2(
  "px-2 py-0.5 w-fit inline-flex justify-start items-center font-medium rounded-md border whitespace-nowrap shrink-0 gap-1.5 transition-all text-xs tracking-tight",
  {
    variants: {
      variant: {
        default: "bg-ink/10 text-brand-snow border-ink/15 hover:bg-ink/15",
        secondary: "bg-secondary-100 dark:bg-secondary-800/40 text-secondary-700 dark:text-secondary-200 border-secondary-600/40",
        outline: "bg-transparent text-foreground-200 border-ink/20 hover:border-ink/30",
        destructive: "bg-danger-500/15 text-danger-700 dark:text-danger-300 border-danger-500/30",
        danger: "bg-danger-500/15 text-danger-700 dark:text-danger-300 border-danger-500/30",
        success: "bg-success-500/15 text-success-700 dark:text-success-300 border-success-500/30",
        warning: "bg-warn-500/15 text-warn-700 dark:text-warn-300 border-warn-500/30",
        warn: "bg-warn-500/15 text-warn-700 dark:text-warn-300 border-warn-500/30",
        info: "font-mono bg-primary-100 dark:bg-primary-900/40 text-brand-sky border-primary-600/30",
        infoLighter: "font-mono bg-ink/10 text-secondary-700 dark:text-secondary-300 border-ink/15",
        role: "text-ink uppercase border-ink/15",
        status: "border-ink/15 text-foreground-300 uppercase tracking-wide",
        detail: "text-xs bg-brand-sky/15 text-brand-sky border-brand-sky/30 !px-1.5 !py-0.5 max-w-[140px] !whitespace-normal",
        code: "bg-sunken/60 border border-danger-200 dark:border-danger-900/40 rounded font-mono text-danger-700 dark:text-danger-400 !py-0 px-1.5",
        wizard: "bg-violet-100 dark:bg-violet-900/50 border border-violet-600/60 text-violet-700 dark:text-violet-200 uppercase tracking-wide px-3 py-0.5",
        wizardSuccess: "bg-success-100 dark:bg-success-900/50 border border-success-600/60 text-success-800 dark:text-success-200 uppercase tracking-wide px-3 py-0.5"
      },
      state: {
        default: "",
        process: "bg-process-400/15 text-process-800 dark:text-process-300 border-process-400/30 [&>span]:bg-process-400",
        warn: "bg-warn-500/15 text-warn-700 dark:text-warn-300 border-warn-500/30 [&>span]:bg-warn-400",
        success: "bg-success-500/15 text-success-700 dark:text-success-300 border-success-500/30 [&>span]:bg-success-400",
        pause: "bg-pause-500/15 text-pause-700 dark:text-pause-300 border-pause-500/30 [&>span]:bg-pause-400",
        danger: "bg-danger-500/15 text-danger-700 dark:text-danger-300 border-danger-500/30 [&>span]:bg-danger-400"
      },
      dsrole: {
        Provider: "bg-roles-provider/15 text-roles-provider border-roles-provider/30",
        Consumer: "bg-roles-consumer/15 text-roles-consumer border-roles-consumer/30",
        Business: "bg-roles-bussiness/15 text-roles-bussiness border-roles-bussiness/30",
        Customer: "bg-roles-customer/15 text-roles-customer border-roles-customer/30"
      },
      size: {
        xs: "text-xs px-1.5 py-0 leading-tight font-medium rounded",
        sm: "text-xs px-2 py-0.5 leading-tight font-medium rounded",
        default: "text-xs px-2.5 py-0.5 font-medium rounded-md",
        lg: "text-sm px-3 py-1 font-semibold rounded-md"
      }
    },
    defaultVariants: {
      variant: "default",
      state: "default",
      size: "default"
    }
  }
);
function Badge({
  className,
  variant,
  state,
  size,
  dsrole,
  dot,
  asChild = false,
  children,
  ...props
}) {
  const Comp = asChild ? Slot2 : "span";
  let resolvedRole = dsrole;
  if (!resolvedRole && variant === "role" && typeof children === "string") {
    const roleStr = children.trim().toLowerCase();
    if (roleStr === "provider") resolvedRole = "Provider";
    else if (roleStr === "consumer") resolvedRole = "Consumer";
    else if (roleStr === "business") resolvedRole = "Business";
    else if (roleStr === "customer") resolvedRole = "Customer";
  }
  const showDot = variant === "status" || dot === true;
  const stateStyle = normalizeStatus(state);
  return /* @__PURE__ */ jsxs8(
    Comp,
    {
      "data-slot": "badge",
      className: cn(
        badgeVariants({ variant, size, state: stateStyle, dsrole: resolvedRole }),
        className
      ),
      ...props,
      children: [
        showDot && /* @__PURE__ */ jsx13("span", { className: "w-1.5 h-1.5 rounded-full mr-1 shrink-0 inline-block bg-current shadow-[0_0_6px_currentColor]" }),
        children
      ]
    }
  );
}

// src/ui/select.tsx
import * as React5 from "react";
import * as SelectPrimitive from "@radix-ui/react-select";
import { CheckIcon, ChevronDownIcon, ChevronUpIcon } from "@radix-ui/react-icons";
import { jsx as jsx14, jsxs as jsxs9 } from "react/jsx-runtime";
var Select = SelectPrimitive.Root;
var SelectGroup = SelectPrimitive.Group;
var SelectValue = SelectPrimitive.Value;
var SelectTrigger = React5.forwardRef(({ className, children, ...props }, ref) => /* @__PURE__ */ jsxs9(
  SelectPrimitive.Trigger,
  {
    ref,
    className: cn(
      "flex h-9 w-full items-center justify-between whitespace-nowrap rounded-md border border-ink/15 bg-background-800/90 px-3 py-1.5 text-xs text-brand-snow shadow-sm ring-offset-background data-[placeholder]:text-muted-foreground/60 transition-colors focus:outline-none focus:ring-1 focus:ring-brand-sky/60 focus:border-brand-sky/60 disabled:cursor-not-allowed disabled:opacity-50 [&>span]:line-clamp-1",
      className
    ),
    ...props,
    children: [
      children,
      /* @__PURE__ */ jsx14(SelectPrimitive.Icon, { asChild: true, children: /* @__PURE__ */ jsx14(ChevronDownIcon, { className: "h-4 w-4 opacity-50" }) })
    ]
  }
));
SelectTrigger.displayName = SelectPrimitive.Trigger.displayName;
var SelectScrollUpButton = React5.forwardRef(({ className, ...props }, ref) => /* @__PURE__ */ jsx14(
  SelectPrimitive.ScrollUpButton,
  {
    ref,
    className: cn("flex cursor-default items-center justify-center py-1", className),
    ...props,
    children: /* @__PURE__ */ jsx14(ChevronUpIcon, { className: "h-4 w-4" })
  }
));
SelectScrollUpButton.displayName = SelectPrimitive.ScrollUpButton.displayName;
var SelectScrollDownButton = React5.forwardRef(({ className, ...props }, ref) => /* @__PURE__ */ jsx14(
  SelectPrimitive.ScrollDownButton,
  {
    ref,
    className: cn("flex cursor-default items-center justify-center py-1", className),
    ...props,
    children: /* @__PURE__ */ jsx14(ChevronDownIcon, { className: "h-4 w-4" })
  }
));
SelectScrollDownButton.displayName = SelectPrimitive.ScrollDownButton.displayName;
var SelectContent = React5.forwardRef(({ className, children, position = "popper", ...props }, ref) => /* @__PURE__ */ jsx14(SelectPrimitive.Portal, { children: /* @__PURE__ */ jsxs9(
  SelectPrimitive.Content,
  {
    ref,
    className: cn(
      "relative z-50 max-h-[--radix-select-content-available-height] min-w-[8rem] overflow-y-auto overflow-x-hidden rounded-md border border-ink/15 bg-background-800 text-brand-snow shadow-xl backdrop-blur-md data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95 data-[side=bottom]:slide-in-from-top-2 data-[side=left]:slide-in-from-right-2 data-[side=right]:slide-in-from-left-2 data-[side=top]:slide-in-from-bottom-2 origin-[--radix-select-content-transform-origin]",
      position === "popper" && "data-[side=bottom]:translate-y-1 data-[side=left]:-translate-x-1 data-[side=right]:translate-x-1 data-[side=top]:-translate-y-1",
      className
    ),
    position,
    ...props,
    children: [
      /* @__PURE__ */ jsx14(SelectScrollUpButton, {}),
      /* @__PURE__ */ jsx14(
        SelectPrimitive.Viewport,
        {
          className: cn(
            "p-1",
            position === "popper" && "h-[var(--radix-select-trigger-height)] w-full min-w-[var(--radix-select-trigger-width)]"
          ),
          children
        }
      ),
      /* @__PURE__ */ jsx14(SelectScrollDownButton, {})
    ]
  }
) }));
SelectContent.displayName = SelectPrimitive.Content.displayName;
var SelectLabel = React5.forwardRef(({ className, ...props }, ref) => /* @__PURE__ */ jsx14(
  SelectPrimitive.Label,
  {
    ref,
    className: cn("px-2 py-1.5 text-sm font-semibold", className),
    ...props
  }
));
SelectLabel.displayName = SelectPrimitive.Label.displayName;
var SelectItem = React5.forwardRef(({ className, children, ...props }, ref) => /* @__PURE__ */ jsxs9(
  SelectPrimitive.Item,
  {
    ref,
    className: cn(
      "relative flex w-full cursor-default select-none items-center rounded-sm py-1.5 pl-2 pr-8 text-sm outline-none focus:bg-background-400 focus:text-accent-foreground data-[disabled]:pointer-events-none data-[disabled]:opacity-50",
      className
    ),
    ...props,
    children: [
      /* @__PURE__ */ jsx14("span", { className: "absolute right-2 flex h-3.5 w-3.5 items-center justify-center", children: /* @__PURE__ */ jsx14(SelectPrimitive.ItemIndicator, { children: /* @__PURE__ */ jsx14(CheckIcon, { className: "h-4 w-4" }) }) }),
      /* @__PURE__ */ jsx14(SelectPrimitive.ItemText, { children })
    ]
  }
));
SelectItem.displayName = SelectPrimitive.Item.displayName;
var SelectSeparator = React5.forwardRef(({ className, ...props }, ref) => /* @__PURE__ */ jsx14(
  SelectPrimitive.Separator,
  {
    ref,
    className: cn("-mx-1 my-1 h-px bg-muted", className),
    ...props
  }
));
SelectSeparator.displayName = SelectPrimitive.Separator.displayName;

// src/table/DataTable.tsx
import {
  ArrowDown,
  ArrowUp,
  ArrowUpDown,
  ChevronLeft,
  ChevronRight,
  ChevronsLeft,
  ChevronsRight,
  Inbox,
  SearchX,
  X as X2
} from "lucide-react";

// src/hooks/useTableQueryParams.ts
import { useState, useMemo, useCallback, useEffect } from "react";
function mapToSortParam(key, direction) {
  if (key === "createdAt" || key === "created_at") {
    return direction === "asc" ? "created_at_asc" : "created_at_desc";
  }
  if (key === "updatedAt" || key === "updated_at" || key === "last_interaction") {
    return direction === "asc" ? "updated_at_asc" : "updated_at_desc";
  }
  const snakeKey = key.replace(/[A-Z]/g, (letter) => `_${letter.toLowerCase()}`);
  return `${snakeKey}_${direction}`;
}
function useTableQueryParams(options = {}) {
  const {
    defaultLimit = 10,
    defaultSort = "created_at_desc",
    defaultFilters = {},
    syncUrl = true
  } = options;
  const [params, setParams] = useState(() => {
    if (typeof window === "undefined") {
      return {
        limit: defaultLimit,
        page: 1,
        cursor: void 0,
        sort: defaultSort,
        search: "",
        filters: { ...defaultFilters }
      };
    }
    const sp = new URLSearchParams(window.location.search);
    const limit = Number(sp.get("limit")) || defaultLimit;
    const page = Number(sp.get("page")) || 1;
    const cursor = sp.get("cursor") || void 0;
    const sort = sp.get("sort") || defaultSort;
    const search = sp.get("search") || "";
    const filters = { ...defaultFilters };
    sp.forEach((value, key) => {
      if (!["limit", "page", "cursor", "sort", "search"].includes(key)) {
        filters[key] = value;
      }
    });
    return { limit, page, cursor, sort, search, filters };
  });
  useEffect(() => {
    if (!syncUrl || typeof window === "undefined") return;
    const url = new URL(window.location.href);
    const sp = url.searchParams;
    if (params.limit !== defaultLimit) {
      sp.set("limit", String(params.limit));
    } else {
      sp.delete("limit");
    }
    if (params.page > 1) {
      sp.set("page", String(params.page));
    } else {
      sp.delete("page");
    }
    if (params.cursor) {
      sp.set("cursor", params.cursor);
    } else {
      sp.delete("cursor");
    }
    if (params.sort && params.sort !== defaultSort) {
      sp.set("sort", params.sort);
    } else {
      sp.delete("sort");
    }
    if (params.search && params.search.trim()) {
      sp.set("search", params.search.trim());
    } else {
      sp.delete("search");
    }
    Object.entries(params.filters).forEach(([key, value]) => {
      if (value && value !== "all") {
        sp.set(key, value);
      } else {
        sp.delete(key);
      }
    });
    const newSearch = sp.toString();
    const newRelativePath = url.pathname + (newSearch ? `?${newSearch}` : "");
    window.history.replaceState(null, "", newRelativePath);
  }, [params, defaultLimit, defaultSort, syncUrl]);
  const apiParams = useMemo(() => {
    const p = {
      limit: params.limit,
      sort: params.sort
    };
    if (params.cursor) {
      p.cursor = params.cursor;
    } else if (params.page > 1) {
      p.page = params.page;
    }
    if (params.search && params.search.trim()) {
      p.search = params.search.trim();
    }
    Object.entries(params.filters).forEach(([k, v]) => {
      if (v && v !== "all") {
        p[k] = v;
      }
    });
    return p;
  }, [params]);
  const onQueryChange = useCallback((updated) => {
    setParams((prev) => {
      const nextFilters = updated.filters ? { ...prev.filters, ...updated.filters } : prev.filters;
      return {
        ...prev,
        ...updated,
        filters: nextFilters
      };
    });
  }, []);
  const setPage = useCallback((page) => {
    setParams((p) => ({ ...p, page, cursor: void 0 }));
  }, []);
  const setLimit = useCallback((limit) => {
    setParams((p) => ({ ...p, limit, page: 1, cursor: void 0 }));
  }, []);
  const setSort = useCallback((sort) => {
    setParams((p) => ({ ...p, sort, page: 1, cursor: void 0 }));
  }, []);
  const setSearch = useCallback((search) => {
    setParams((p) => ({ ...p, search, page: 1, cursor: void 0 }));
  }, []);
  const setFilter = useCallback((id, value) => {
    setParams((p) => ({
      ...p,
      page: 1,
      cursor: void 0,
      filters: { ...p.filters, [id]: value }
    }));
  }, []);
  const resetAll = useCallback(() => {
    setParams({
      limit: defaultLimit,
      page: 1,
      cursor: void 0,
      sort: defaultSort,
      search: "",
      filters: { ...defaultFilters }
    });
  }, [defaultLimit, defaultSort, defaultFilters]);
  return {
    params,
    apiParams,
    onQueryChange,
    setPage,
    setLimit,
    setSort,
    setSearch,
    setFilter,
    resetAll
  };
}

// src/table/DataTable.tsx
import { Fragment as Fragment3, jsx as jsx15, jsxs as jsxs10 } from "react/jsx-runtime";
var toSnakeCase = (str) => str.replace(/[A-Z]/g, (l) => `_${l.toLowerCase()}`);
var isSortConfigMatch = (configKey, colKey) => {
  if (!configKey) return false;
  return configKey === colKey || configKey === toSnakeCase(colKey);
};
function DataTable({
  columns,
  data,
  keyExtractor,
  onRowClick,
  className,
  containerClassName,
  emptyMessage,
  emptyDescription,
  title,
  subtitle,
  toolbarActions,
  searchable = true,
  searchPlaceholder = "Filter records...",
  filterFn,
  filters,
  defaultSortKey,
  defaultSortDirection,
  paginated = true,
  pageSize = 10,
  pageSizeOptions = [10, 20, 50, 100],
  hideToolbar = false,
  hideCount = false,
  serverSide = false,
  totalCount,
  nextCursor,
  queryParams,
  onQueryChange,
  syncUrlParams = true,
  loading = false
}) {
  const rawItems = useMemo2(() => {
    if (!data) return [];
    if (Array.isArray(data)) return data;
    if (typeof data === "object" && Array.isArray(data.items)) {
      return data.items;
    }
    return [];
  }, [data]);
  const totalCountValue = useMemo2(() => {
    if (totalCount !== void 0) return totalCount;
    if (data && typeof data === "object" && typeof data.total === "number") {
      return data.total;
    }
    return void 0;
  }, [totalCount, data]);
  const isServerDriven = serverSide || Boolean(onQueryChange);
  const [searchQuery, setSearchQuery] = useState2(queryParams?.search ?? "");
  useEffect2(() => {
    if (queryParams?.search !== void 0 && queryParams.search !== searchQuery) {
      setSearchQuery(queryParams.search);
    }
  }, [queryParams?.search]);
  const [sortConfig, setSortConfig] = useState2(() => {
    if (queryParams?.sort) {
      const s = queryParams.sort;
      if (s.endsWith("_asc")) return { key: s.slice(0, -4), direction: "asc" };
      if (s.endsWith("_desc")) return { key: s.slice(0, -5), direction: "desc" };
    }
    return defaultSortKey ? { key: defaultSortKey, direction: defaultSortDirection ?? "asc" } : null;
  });
  useEffect2(() => {
    if (queryParams?.sort) {
      const s = queryParams.sort;
      if (s.endsWith("_asc")) {
        const key = s.slice(0, -4);
        setSortConfig({ key, direction: "asc" });
      } else if (s.endsWith("_desc")) {
        const key = s.slice(0, -5);
        setSortConfig({ key, direction: "desc" });
      }
    }
  }, [queryParams?.sort]);
  const [currentPage, setCurrentPage] = useState2(queryParams?.page ?? 1);
  useEffect2(() => {
    if (queryParams?.page !== void 0) {
      setCurrentPage(queryParams.page);
    }
  }, [queryParams?.page]);
  const [itemsPerPage, setItemsPerPage] = useState2(() => {
    if (!paginated) return void 0;
    const initialLimit = queryParams?.limit ?? pageSize;
    if (initialLimit <= 0 || initialLimit === Infinity) return void 0;
    return initialLimit;
  });
  useEffect2(() => {
    if (!paginated) {
      setItemsPerPage(void 0);
    } else if (queryParams?.limit !== void 0) {
      setItemsPerPage(queryParams.limit <= 0 ? void 0 : queryParams.limit);
    } else if (pageSize <= 0 || pageSize === Infinity) {
      setItemsPerPage(void 0);
    } else {
      setItemsPerPage(pageSize);
    }
  }, [paginated, pageSize, queryParams?.limit]);
  const [filterValues, setFilterValues] = useState2(() => {
    const initial = {};
    filters?.forEach((f) => {
      initial[f.id] = queryParams?.filters?.[f.id] ?? f.value ?? f.defaultValue ?? "all";
    });
    return initial;
  });
  useEffect2(() => {
    if (filters || queryParams?.filters) {
      setFilterValues((prev) => {
        let changed = false;
        const next = { ...prev };
        filters?.forEach((f) => {
          const targetVal = queryParams?.filters?.[f.id] ?? f.value ?? f.defaultValue ?? "all";
          if (prev[f.id] !== targetVal) {
            next[f.id] = targetVal;
            changed = true;
          }
        });
        return changed ? next : prev;
      });
    }
  }, [filters, queryParams?.filters]);
  const handleFilterChange = useCallback2(
    (filter, val) => {
      setFilterValues((prev) => {
        const next = { ...prev, [filter.id]: val };
        if (onQueryChange) {
          onQueryChange({ filters: next, page: 1 });
        }
        return next;
      });
      setCurrentPage(1);
      if (filter.onChange) {
        filter.onChange(val);
      }
    },
    [onQueryChange]
  );
  const handleSearchChange = useCallback2(
    (val) => {
      setSearchQuery(val);
      setCurrentPage(1);
      if (onQueryChange) {
        onQueryChange({ search: val, page: 1 });
      }
    },
    [onQueryChange]
  );
  const handlePageSizeChange = useCallback2(
    (newLimit) => {
      setItemsPerPage(newLimit === -1 ? void 0 : newLimit);
      setCurrentPage(1);
      if (onQueryChange) {
        onQueryChange({ limit: newLimit === -1 ? 100 : newLimit, page: 1 });
      }
    },
    [onQueryChange]
  );
  const handlePageChange = useCallback2(
    (newPage) => {
      setCurrentPage(newPage);
      if (onQueryChange) {
        onQueryChange({ page: newPage });
      }
    },
    [onQueryChange]
  );
  const filteredData = useMemo2(() => {
    if (isServerDriven) return rawItems;
    let result = rawItems;
    if (filters && filters.length > 0) {
      result = result.filter((item) => {
        for (const f of filters) {
          const selectedVal = f.value !== void 0 ? f.value : filterValues[f.id] ?? "all";
          if (!selectedVal || selectedVal === "all") continue;
          if (f.filterFn) {
            if (!f.filterFn(item, selectedVal)) return false;
          } else if (f.accessorKey) {
            const raw = item[f.accessorKey];
            if (raw === null || raw === void 0) return false;
            if (String(raw).toLowerCase() !== selectedVal.toLowerCase()) {
              return false;
            }
          }
        }
        return true;
      });
    }
    const trimmed = searchQuery.trim().toLowerCase();
    if (!trimmed) return result;
    const terms = trimmed.split(/\s+/).filter(Boolean);
    return result.filter((item) => {
      if (filterFn) return filterFn(item, trimmed);
      for (const col of columns) {
        if (col.searchable === false) continue;
        let valStr = "";
        if (col.searchValue) {
          valStr = col.searchValue(item).toLowerCase();
        } else if (col.accessorKey) {
          const raw = item[col.accessorKey];
          if (raw !== null && raw !== void 0) {
            valStr = String(raw).toLowerCase();
          }
        }
        if (valStr && terms.every((t) => valStr.includes(t))) {
          return true;
        }
      }
      const itemValues = Object.values(item).filter((v) => typeof v === "string" || typeof v === "number").map((v) => String(v).toLowerCase()).join(" ");
      return terms.every((t) => itemValues.includes(t));
    });
  }, [rawItems, isServerDriven, filters, filterValues, searchQuery, filterFn, columns]);
  const sortedData = useMemo2(() => {
    if (isServerDriven) return filteredData;
    if (!sortConfig) return filteredData;
    const col = columns.find((c, index) => {
      const k = c.sortKey ?? c.accessorKey ?? (typeof c.header === "string" ? c.header : `col_${index}`);
      return isSortConfigMatch(sortConfig.key, String(k));
    });
    return [...filteredData].sort((a, b) => {
      let aVal;
      let bVal;
      if (col?.sortValue) {
        aVal = col.sortValue(a);
        bVal = col.sortValue(b);
      } else if (col?.accessorKey) {
        aVal = a[col.accessorKey];
        bVal = b[col.accessorKey];
      } else {
        aVal = a[sortConfig.key];
        bVal = b[sortConfig.key];
      }
      if (aVal === bVal) return 0;
      if (aVal === null || aVal === void 0) return sortConfig.direction === "asc" ? 1 : -1;
      if (bVal === null || bVal === void 0) return sortConfig.direction === "asc" ? -1 : 1;
      if (aVal instanceof Date && bVal instanceof Date) {
        return sortConfig.direction === "asc" ? aVal.getTime() - bVal.getTime() : bVal.getTime() - aVal.getTime();
      }
      if (typeof aVal === "number" && typeof bVal === "number") {
        return sortConfig.direction === "asc" ? aVal - bVal : bVal - aVal;
      }
      if (typeof aVal === "boolean" && typeof bVal === "boolean") {
        return sortConfig.direction === "asc" ? aVal === bVal ? 0 : aVal ? 1 : -1 : aVal === bVal ? 0 : aVal ? -1 : 1;
      }
      if (typeof aVal === "string" && typeof bVal === "string") {
        const isDateLike = /^\d{4}-\d{2}-\d{2}/.test(aVal) && /^\d{4}-\d{2}-\d{2}/.test(bVal);
        if (isDateLike) {
          const aTime = Date.parse(aVal);
          const bTime = Date.parse(bVal);
          if (!isNaN(aTime) && !isNaN(bTime)) {
            return sortConfig.direction === "asc" ? aTime - bTime : bTime - aTime;
          }
        }
      }
      const aStr = String(aVal).toLowerCase();
      const bStr = String(bVal).toLowerCase();
      return sortConfig.direction === "asc" ? aStr.localeCompare(bStr) : bStr.localeCompare(aStr);
    });
  }, [filteredData, isServerDriven, sortConfig, columns]);
  const activeItemsPerPage = itemsPerPage ?? (isServerDriven ? rawItems.length || 10 : void 0);
  const hasMore = isServerDriven ? totalCountValue !== void 0 ? currentPage < Math.ceil(totalCountValue / (activeItemsPerPage || 10)) : activeItemsPerPage ? rawItems.length === activeItemsPerPage : false : safeCurrentPageCheck();
  function safeCurrentPageCheck() {
    const calcPages = activeItemsPerPage ? Math.ceil(sortedData.length / activeItemsPerPage) || 1 : 1;
    return currentPage < calcPages;
  }
  const totalPages = totalCountValue !== void 0 ? activeItemsPerPage ? Math.ceil(totalCountValue / activeItemsPerPage) || 1 : 1 : isServerDriven ? hasMore ? currentPage + 1 : Math.max(1, currentPage) : activeItemsPerPage ? Math.ceil(sortedData.length / activeItemsPerPage) || 1 : 1;
  const safeCurrentPage = totalCountValue !== void 0 || !isServerDriven ? Math.min(Math.max(1, currentPage), totalPages) : Math.max(1, currentPage);
  const totalItems = totalCountValue !== void 0 ? totalCountValue : isServerDriven ? (safeCurrentPage - 1) * (activeItemsPerPage || 10) + rawItems.length : sortedData.length;
  useEffect2(() => {
    if ((totalCountValue !== void 0 || !isServerDriven) && currentPage > totalPages) {
      setCurrentPage(totalPages);
    }
  }, [currentPage, totalPages, totalCountValue, isServerDriven]);
  const paginatedData = useMemo2(() => {
    if (isServerDriven) return rawItems;
    if (!itemsPerPage) return sortedData;
    const start = (safeCurrentPage - 1) * itemsPerPage;
    return sortedData.slice(start, start + itemsPerPage);
  }, [isServerDriven, rawItems, sortedData, safeCurrentPage, itemsPerPage]);
  const startItem = rawItems.length === 0 ? 0 : (safeCurrentPage - 1) * (activeItemsPerPage ?? (isServerDriven ? rawItems.length : totalItems)) + 1;
  const endItem = rawItems.length === 0 ? 0 : isServerDriven && totalCountValue === void 0 ? startItem + rawItems.length - 1 : activeItemsPerPage ? Math.min(safeCurrentPage * activeItemsPerPage, totalItems) : totalItems;
  const isColumnSortable = (col) => {
    if (col.sortable === false) return false;
    const key = col.sortKey ?? col.accessorKey;
    return key !== void 0 && toSnakeCase(String(key)) === "created_at";
  };
  const handleHeaderClick = (col, index) => {
    if (!isColumnSortable(col)) return;
    const colKey = col.sortKey ?? col.accessorKey ?? (typeof col.header === "string" ? col.header : `col_${index}`);
    const colKeyStr = String(colKey);
    const colSnakeKey = toSnakeCase(colKeyStr);
    const isCurrentKey = isSortConfigMatch(sortConfig?.key, colKeyStr);
    const nextDirection = isCurrentKey && sortConfig?.direction === "asc" ? "desc" : "asc";
    setSortConfig({ key: colSnakeKey, direction: nextDirection });
    if (onQueryChange) {
      const backendSort = mapToSortParam(colKeyStr, nextDirection);
      onQueryChange({ sort: backendSort, page: 1 });
    }
  };
  const hasActiveFilter = Boolean(
    filters?.some((f) => {
      const val = f.value !== void 0 ? f.value : filterValues[f.id];
      return val && val !== "all";
    })
  );
  const isFiltered = searchQuery.trim().length > 0 || hasActiveFilter;
  const showToolbar = !hideToolbar && (searchable || title || toolbarActions || !hideCount || Boolean(filters && filters.length > 0));
  const handleResetAll = useCallback2(() => {
    setSearchQuery("");
    const resetVals = {};
    if (filters) {
      filters.forEach((f) => {
        resetVals[f.id] = "all";
        if (f.onChange) f.onChange("all");
      });
      setFilterValues(resetVals);
    }
    setCurrentPage(1);
    if (onQueryChange) {
      onQueryChange({ search: "", filters: resetVals, page: 1 });
    }
  }, [filters, onQueryChange]);
  return /* @__PURE__ */ jsxs10("div", { className: "flex flex-col gap-2.5 w-full", children: [
    showToolbar && /* @__PURE__ */ jsxs10("div", { className: "flex flex-col sm:flex-row sm:items-center justify-between gap-3 px-1 py-1", children: [
      /* @__PURE__ */ jsxs10("div", { className: "flex flex-wrap items-center gap-3", children: [
        title && /* @__PURE__ */ jsxs10("div", { className: "flex flex-col", children: [
          /* @__PURE__ */ jsx15("h3", { className: "text-sm font-semibold text-brand-snow", children: title }),
          subtitle && /* @__PURE__ */ jsx15("p", { className: "text-xs text-muted-foreground", children: subtitle })
        ] }),
        searchable && /* @__PURE__ */ jsx15("div", { className: "w-full sm:w-64 max-w-sm", children: /* @__PURE__ */ jsx15(
          Input,
          {
            type: "search",
            value: searchQuery,
            onChange: (e) => handleSearchChange(e.target.value),
            onClear: () => handleSearchChange(""),
            placeholder: searchPlaceholder,
            className: "text-xs"
          }
        ) }),
        filters && filters.length > 0 && /* @__PURE__ */ jsx15("div", { className: "flex flex-wrap items-center gap-2", children: filters.map((f) => {
          const currentVal = f.value !== void 0 ? f.value : filterValues[f.id] ?? "all";
          return /* @__PURE__ */ jsx15("div", { className: "flex items-center gap-1.5", children: /* @__PURE__ */ jsxs10(Select, { value: currentVal, onValueChange: (v) => handleFilterChange(f, v), children: [
            /* @__PURE__ */ jsxs10(
              SelectTrigger,
              {
                "aria-label": f.label,
                className: cn(
                  "h-8 w-auto gap-2",
                  currentVal !== "all" && "border-brand-sky/60 bg-brand-sky/10 text-brand-sky font-medium"
                ),
                children: [
                  /* @__PURE__ */ jsxs10("span", { className: "text-ink/50", children: [
                    f.label,
                    ":"
                  ] }),
                  /* @__PURE__ */ jsx15(SelectValue, {})
                ]
              }
            ),
            /* @__PURE__ */ jsxs10(SelectContent, { children: [
              /* @__PURE__ */ jsx15(SelectItem, { value: "all", children: "All" }),
              f.options.map((opt) => /* @__PURE__ */ jsx15(SelectItem, { value: opt.value, children: opt.label }, opt.value))
            ] })
          ] }) }, f.id);
        }) }),
        !hideCount && (isFiltered || !paginated) && /* @__PURE__ */ jsxs10("div", { className: "flex items-center gap-1.5", children: [
          isFiltered ? /* @__PURE__ */ jsxs10(Badge, { variant: "info", size: "sm", className: "text-xs", children: [
            rawItems.length,
            " of ",
            totalItems
          ] }) : /* @__PURE__ */ jsxs10("span", { className: "text-xs text-ink/50", children: [
            totalItems,
            " ",
            totalItems === 1 ? "record" : "records"
          ] }),
          isFiltered && /* @__PURE__ */ jsxs10(
            Button,
            {
              variant: "ghost",
              size: "xs",
              onClick: handleResetAll,
              className: "text-muted-foreground hover:text-ink gap-1",
              children: [
                /* @__PURE__ */ jsx15(X2, { className: "h-3 w-3" }),
                "Clear"
              ]
            }
          )
        ] })
      ] }),
      toolbarActions && /* @__PURE__ */ jsx15("div", { className: "flex items-center gap-2 shrink-0", children: toolbarActions })
    ] }),
    /* @__PURE__ */ jsxs10(Table, { className, containerClassName, children: [
      /* @__PURE__ */ jsx15(TableHeader, { className: "sticky top-0 z-10 bg-background-800/90 backdrop-blur border-b border-ink/10", children: /* @__PURE__ */ jsx15(TableRow, { children: columns.map((col, index) => {
        const sortable = isColumnSortable(col);
        const colKey = col.sortKey ?? col.accessorKey ?? (typeof col.header === "string" ? col.header : `col_${index}`);
        const colKeyStr = String(colKey);
        const isSortedAsc = isSortConfigMatch(sortConfig?.key, colKeyStr) && sortConfig?.direction === "asc";
        const isSortedDesc = isSortConfigMatch(sortConfig?.key, colKeyStr) && sortConfig?.direction === "desc";
        return /* @__PURE__ */ jsx15(
          TableHead,
          {
            className: cn(
              sortable && "cursor-pointer select-none hover:text-ink transition-colors group",
              col.className,
              col.headerClassName
            ),
            onClick: () => handleHeaderClick(col, index),
            children: /* @__PURE__ */ jsxs10("div", { className: "inline-flex items-center gap-1.5", children: [
              /* @__PURE__ */ jsx15("span", { children: col.header }),
              sortable && /* @__PURE__ */ jsx15("span", { className: "shrink-0", children: isSortedAsc ? /* @__PURE__ */ jsx15(ArrowUp, { className: "h-3.5 w-3.5 text-brand-sky" }) : isSortedDesc ? /* @__PURE__ */ jsx15(ArrowDown, { className: "h-3.5 w-3.5 text-brand-sky" }) : /* @__PURE__ */ jsx15(ArrowUpDown, { className: "h-3.5 w-3.5 opacity-25 group-hover:opacity-70 transition-opacity" }) })
            ] })
          },
          index
        );
      }) }) }),
      /* @__PURE__ */ jsx15(
        TableBody,
        {
          className: cn("transition-opacity duration-200", loading && "opacity-50 pointer-events-none"),
          children: paginatedData.length === 0 ? /* @__PURE__ */ jsx15(TableRow, { className: "hover:bg-transparent", children: /* @__PURE__ */ jsx15(TableCell, { colSpan: columns.length, className: "text-center py-12", children: isFiltered ? /* @__PURE__ */ jsxs10("div", { className: "flex flex-col items-center justify-center gap-2", children: [
            /* @__PURE__ */ jsx15("div", { className: "p-2.5 rounded-full bg-ink/5 border border-ink/10 text-muted-foreground", children: /* @__PURE__ */ jsx15(SearchX, { className: "h-5 w-5" }) }),
            /* @__PURE__ */ jsx15("span", { className: "text-xs font-medium text-foreground", children: "No matching records found" }),
            /* @__PURE__ */ jsx15("span", { className: "text-xs text-muted-foreground", children: "No items matched current search or filter criteria" }),
            /* @__PURE__ */ jsx15(
              Button,
              {
                variant: "outline",
                size: "xs",
                onClick: handleResetAll,
                className: "mt-2",
                children: "Reset filter"
              }
            )
          ] }) : /* @__PURE__ */ jsxs10("div", { className: "flex flex-col items-center justify-center gap-2", children: [
            /* @__PURE__ */ jsx15("div", { className: "p-2.5 rounded-full bg-ink/5 border border-ink/10 text-muted-foreground/50", children: /* @__PURE__ */ jsx15(Inbox, { className: "h-5 w-5" }) }),
            /* @__PURE__ */ jsx15("span", { className: "text-xs text-muted-foreground italic", children: emptyMessage ?? "Nothing here yet" }),
            emptyDescription && /* @__PURE__ */ jsx15("span", { className: "text-xs text-muted-foreground/70 max-w-md", children: emptyDescription })
          ] }) }) }) : paginatedData.map((item, itemIndex) => /* @__PURE__ */ jsx15(
            TableRow,
            {
              onClick: () => onRowClick && onRowClick(item),
              className: cn(
                onRowClick && "cursor-pointer hover:bg-ink/[0.04] active:bg-ink/[0.06] transition-colors"
              ),
              children: columns.map((col, index) => /* @__PURE__ */ jsx15(
                TableCell,
                {
                  className: cn("text-ink [&>p]:text-ink", col.className),
                  children: col.cell ? col.cell(item) : item[col.accessorKey]
                },
                index
              ))
            },
            keyExtractor ? keyExtractor(item) || `row_${itemIndex}` : `row_${itemIndex}`
          ))
        }
      )
    ] }),
    paginated && (totalItems > 0 || safeCurrentPage > 1) && /* @__PURE__ */ jsxs10("div", { className: "flex flex-col sm:flex-row items-center justify-between gap-3 px-3 py-2 text-xs text-muted-foreground border-t border-ink/10 bg-background-800/30 rounded-b-lg", children: [
      /* @__PURE__ */ jsx15("div", { className: "flex items-center gap-1.5", children: /* @__PURE__ */ jsxs10("span", { children: [
        "Showing ",
        /* @__PURE__ */ jsx15("span", { className: "font-medium text-brand-snow", children: startItem }),
        " to",
        " ",
        /* @__PURE__ */ jsx15("span", { className: "font-medium text-brand-snow", children: endItem }),
        totalCountValue !== void 0 ? /* @__PURE__ */ jsxs10(Fragment3, { children: [
          " of ",
          /* @__PURE__ */ jsx15("span", { className: "font-medium text-brand-snow", children: totalItems }),
          " ",
          totalItems === 1 ? "record" : "records"
        ] }) : hasMore ? /* @__PURE__ */ jsxs10(Fragment3, { children: [
          " of ",
          /* @__PURE__ */ jsxs10("span", { className: "font-medium text-brand-snow", children: [
            endItem,
            "+"
          ] }),
          " records"
        ] }) : " records"
      ] }) }),
      /* @__PURE__ */ jsxs10("div", { className: "flex items-center gap-4 flex-wrap", children: [
        /* @__PURE__ */ jsxs10("div", { className: "flex items-center gap-1.5", children: [
          /* @__PURE__ */ jsx15("span", { className: "text-xs text-muted-foreground", children: "Rows per page:" }),
          /* @__PURE__ */ jsxs10(
            Select,
            {
              value: String(activeItemsPerPage ?? -1),
              onValueChange: (v) => handlePageSizeChange(Number(v)),
              children: [
                /* @__PURE__ */ jsx15(SelectTrigger, { "aria-label": "Rows per page", className: "h-7 w-auto gap-2", children: /* @__PURE__ */ jsx15(SelectValue, {}) }),
                /* @__PURE__ */ jsxs10(SelectContent, { children: [
                  pageSizeOptions.map((opt) => /* @__PURE__ */ jsx15(SelectItem, { value: String(opt), children: opt }, opt)),
                  /* @__PURE__ */ jsx15(SelectItem, { value: "-1", children: "All" })
                ] })
              ]
            }
          )
        ] }),
        /* @__PURE__ */ jsxs10("div", { className: "flex items-center gap-1", children: [
          /* @__PURE__ */ jsxs10("span", { className: "text-xs mr-1", children: [
            "Page ",
            /* @__PURE__ */ jsx15("span", { className: "font-medium text-brand-snow", children: safeCurrentPage }),
            totalCountValue !== void 0 ? /* @__PURE__ */ jsxs10(Fragment3, { children: [
              " of ",
              /* @__PURE__ */ jsx15("span", { className: "font-medium text-brand-snow", children: totalPages })
            ] }) : hasMore ? null : /* @__PURE__ */ jsxs10(Fragment3, { children: [
              " of ",
              /* @__PURE__ */ jsx15("span", { className: "font-medium text-brand-snow", children: safeCurrentPage })
            ] })
          ] }),
          /* @__PURE__ */ jsx15(
            Button,
            {
              variant: "outline",
              size: "xs",
              disabled: safeCurrentPage <= 1,
              onClick: () => handlePageChange(1),
              className: "h-7 w-7 p-0",
              title: "First page",
              children: /* @__PURE__ */ jsx15(ChevronsLeft, { className: "h-3.5 w-3.5" })
            }
          ),
          /* @__PURE__ */ jsx15(
            Button,
            {
              variant: "outline",
              size: "xs",
              disabled: safeCurrentPage <= 1,
              onClick: () => handlePageChange(Math.max(1, safeCurrentPage - 1)),
              className: "h-7 w-7 p-0",
              title: "Previous page",
              children: /* @__PURE__ */ jsx15(ChevronLeft, { className: "h-3.5 w-3.5" })
            }
          ),
          /* @__PURE__ */ jsx15(
            Button,
            {
              variant: "outline",
              size: "xs",
              disabled: isServerDriven && totalCountValue === void 0 ? !hasMore : safeCurrentPage >= totalPages,
              onClick: () => handlePageChange(safeCurrentPage + 1),
              className: "h-7 w-7 p-0",
              title: "Next page",
              children: /* @__PURE__ */ jsx15(ChevronRight, { className: "h-3.5 w-3.5" })
            }
          ),
          (!isServerDriven || totalCountValue !== void 0) && /* @__PURE__ */ jsx15(
            Button,
            {
              variant: "outline",
              size: "xs",
              disabled: safeCurrentPage >= totalPages,
              onClick: () => handlePageChange(totalPages),
              className: "h-7 w-7 p-0",
              title: "Last page",
              children: /* @__PURE__ */ jsx15(ChevronsRight, { className: "h-3.5 w-3.5" })
            }
          )
        ] })
      ] })
    ] })
  ] });
}

// src/ui/skeleton.tsx
import { jsx as jsx16 } from "react/jsx-runtime";
function Skeleton({ className, ...props }) {
  return /* @__PURE__ */ jsx16("div", { className: cn("animate-pulse rounded-md bg-primary/10", className), ...props });
}

// src/ui/format-date.tsx
import dayjs from "dayjs";
import relativeTime from "dayjs/plugin/relativeTime";
import { jsx as jsx17 } from "react/jsx-runtime";
dayjs.extend(relativeTime);
var DATE_TIME_FORMAT = "YYYY-MM-DD HH:mm";
var DATE_FORMAT = "YYYY-MM-DD";
var FormatDate = ({ date, format = DATE_TIME_FORMAT }) => {
  if (!date) return /* @__PURE__ */ jsx17("span", { className: "text-ink/35", children: "\u2014" });
  return /* @__PURE__ */ jsx17("time", { dateTime: dayjs(date).toISOString(), title: dayjs(date).format("YYYY-MM-DD HH:mm:ss"), children: dayjs(date).format(format) });
};
var RelativeTime = ({ date, className }) => /* @__PURE__ */ jsx17(
  "time",
  {
    dateTime: dayjs(date).toISOString(),
    title: dayjs(date).format("YYYY-MM-DD HH:mm:ss"),
    className,
    children: dayjs(date).fromNow()
  }
);

// src/wallet/WalletInfoView.tsx
import { jsx as jsx18, jsxs as jsxs11 } from "react/jsx-runtime";
function WalletInfoView({ didTabHref = "/wallet/did" }) {
  const Link = useLink();
  const {
    data: response,
    isLoading,
    error
  } = useQuery2({
    queryKey: ["wallet-info-custom"],
    queryFn: () => customInstance("/wallet/info", { method: "GET" })
  });
  const info = response?.status === 200 ? response.data : null;
  if (isLoading) {
    return /* @__PURE__ */ jsxs11("div", { className: "space-y-8", children: [
      /* @__PURE__ */ jsx18(Skeleton, { className: "h-40 w-full" }),
      /* @__PURE__ */ jsx18(Skeleton, { className: "h-64 w-full" })
    ] });
  }
  if (error || !info) {
    return /* @__PURE__ */ jsx18("div", { className: "text-destructive font-mono text-xs", children: "Error loading wallet info" });
  }
  return /* @__PURE__ */ jsxs11("div", { className: "space-y-12 pb-20", children: [
    /* @__PURE__ */ jsx18(PageSection, { title: "General information", children: /* @__PURE__ */ jsx18(
      FieldGrid,
      {
        columns: 4,
        fields: [
          { label: "Wallet ID", value: /* @__PURE__ */ jsx18("span", { className: "font-mono text-xs", children: info.id }) },
          { label: "Name", value: info.name },
          { label: "Created", value: /* @__PURE__ */ jsx18(FormatDate, { date: info.createdOn || null }) },
          {
            label: "Permission level",
            value: /* @__PURE__ */ jsx18(Badge, { variant: "outline", children: info.permission })
          }
        ]
      }
    ) }),
    /* @__PURE__ */ jsxs11(PageSection, { title: "Associated DIDs", children: [
      /* @__PURE__ */ jsxs11("p", { className: "text-xs text-muted-foreground mb-4", children: [
        "Keys and the default DID are managed in the",
        " ",
        /* @__PURE__ */ jsx18(Link, { to: didTabHref, className: "font-medium text-brand-sky hover:underline", children: "DID" }),
        " ",
        "tab."
      ] }),
      /* @__PURE__ */ jsx18(
        DataTable,
        {
          searchable: false,
          data: info.dids,
          keyExtractor: (d) => d.did,
          columns: [
            {
              header: "Alias",
              accessorKey: "alias",
              cell: (d) => /* @__PURE__ */ jsx18("span", { className: "font-medium", children: d.alias })
            },
            {
              header: "DID",
              accessorKey: "did",
              cell: (d) => /* @__PURE__ */ jsx18(Identifier, { value: d.did, tail: 12 })
            },
            {
              header: "Type",
              accessorKey: "type",
              cell: (d) => /* @__PURE__ */ jsx18(Badge, { variant: "info", className: "font-mono", children: d.type })
            },
            {
              header: "Default",
              accessorKey: "default",
              cell: (d) => /* @__PURE__ */ jsx18(Badge, { variant: d.default ? "default" : "info", children: d.default ? "PRIMARY" : "SECONDARY" })
            },
            {
              header: "# Keys",
              cell: (d) => /* @__PURE__ */ jsx18("span", { className: "text-xs text-muted-foreground", children: d.keys.length })
            }
          ]
        }
      )
    ] })
  ] });
}

// src/wallet/WalletDidView.tsx
import { useState as useState5 } from "react";
import { useMutation as useMutation3, useQuery as useQuery4, useQueryClient as useQueryClient3 } from "@tanstack/react-query";
import {
  Check as Check2,
  ChevronRight as ChevronRight2,
  Copy,
  FileJson,
  Key,
  Loader2 as Loader24,
  Plus as Plus2,
  Star,
  Trash2 as Trash22
} from "lucide-react";
import { toast as toast3 } from "sonner";

// src/ui/checkbox.tsx
import * as React7 from "react";
import * as CheckboxPrimitive from "@radix-ui/react-checkbox";
import { Check } from "lucide-react";
import { jsx as jsx19 } from "react/jsx-runtime";
var Checkbox = React7.forwardRef(({ className, ...props }, ref) => /* @__PURE__ */ jsx19(
  CheckboxPrimitive.Root,
  {
    ref,
    className: cn(
      "peer h-4 w-4 shrink-0 rounded-sm border border-ink/20 bg-background-800 ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-brand-sky/60 disabled:cursor-not-allowed disabled:opacity-50 data-[state=checked]:bg-primary data-[state=checked]:border-primary data-[state=checked]:text-ink",
      className
    ),
    ...props,
    children: /* @__PURE__ */ jsx19(
      CheckboxPrimitive.Indicator,
      {
        className: cn("flex items-center justify-center text-current"),
        children: /* @__PURE__ */ jsx19(Check, { className: "h-3 w-3 stroke-[3]" })
      }
    )
  }
));
Checkbox.displayName = CheckboxPrimitive.Root.displayName;

// src/ui/label.tsx
import * as React8 from "react";
import * as LabelPrimitive from "@radix-ui/react-label";
import { cva as cva3 } from "class-variance-authority";
import { jsx as jsx20 } from "react/jsx-runtime";
var labelVariants = cva3(
  "text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70"
);
var Label2 = React8.forwardRef(({ className, ...props }, ref) => /* @__PURE__ */ jsx20(LabelPrimitive.Root, { ref, className: cn(labelVariants(), className), ...props }));
Label2.displayName = LabelPrimitive.Root.displayName;

// src/ui/dialog.tsx
import * as React9 from "react";
import * as DialogPrimitive from "@radix-ui/react-dialog";
import { Cross2Icon } from "@radix-ui/react-icons";
import { jsx as jsx21, jsxs as jsxs12 } from "react/jsx-runtime";
var Dialog = DialogPrimitive.Root;
var DialogTrigger = DialogPrimitive.Trigger;
var DialogPortal = DialogPrimitive.Portal;
var DialogClose = DialogPrimitive.Close;
var DialogOverlay = React9.forwardRef(({ className, ...props }, ref) => /* @__PURE__ */ jsx21(
  DialogPrimitive.Overlay,
  {
    ref,
    className: cn(
      "fixed inset-0 z-50 bg-black/80 flex data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0",
      className
    ),
    ...props
  }
));
DialogOverlay.displayName = DialogPrimitive.Overlay.displayName;
var DialogContent = React9.forwardRef(({ className, children, ...props }, ref) => /* @__PURE__ */ jsxs12(DialogPortal, { children: [
  /* @__PURE__ */ jsx21(DialogOverlay, {}),
  /* @__PURE__ */ jsxs12(
    DialogPrimitive.Content,
    {
      ref,
      className: cn(
        "sm:min-w-[600px] fixed left-[50%] top-[50%] z-50 w-full grid overflow-hidden max-w-lg translate-x-[-50%] translate-y-[-50%] gap-6 border bg-background p-6 text-text shadow-lg duration-200 data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95 data-[state=closed]:slide-out-to-left-1/2 data-[state=closed]:slide-out-to-top-[48%] data-[state=open]:slide-in-from-left-1/2 data-[state=open]:slide-in-from-top-[48%] sm:rounded-lg",
        className
      ),
      ...props,
      children: [
        children,
        /* @__PURE__ */ jsxs12(DialogPrimitive.Close, { className: "absolute right-4 top-4 rounded-sm opacity-70 ring-offset-background transition-opacity hover:opacity-100 focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 disabled:pointer-events-none data-[state=open]:bg-accent data-[state=open]:text-muted-foreground", children: [
          /* @__PURE__ */ jsx21(Cross2Icon, { className: "h-4 w-4" }),
          /* @__PURE__ */ jsx21("span", { className: "sr-only", children: "Close" })
        ] })
      ]
    }
  )
] }));
DialogContent.displayName = DialogPrimitive.Content.displayName;
var DialogHeader = ({ className, ...props }) => /* @__PURE__ */ jsx21("div", { className: cn("flex flex-col space-y-1.5 sm:text-left", className), ...props });
DialogHeader.displayName = "DialogHeader";
var DialogBody = ({ className, ...props }) => /* @__PURE__ */ jsx21("div", { className: cn("py-4 overflow-y-scroll px-6", className), ...props });
DialogBody.displayName = "DialogBody";
var DialogFooter = ({ className, ...props }) => /* @__PURE__ */ jsx21(
  "div",
  {
    className: cn("flex flex-col-reverse gap-2 xs:flex-row xs:justify-end xs:space-x-2", className),
    ...props
  }
);
DialogFooter.displayName = "DialogFooter";
var DialogTitle = React9.forwardRef(({ className, ...props }, ref) => /* @__PURE__ */ jsx21(
  DialogPrimitive.Title,
  {
    ref,
    className: cn(
      "mb-2 text-lg text-foreground-400 font-medium leading-none tracking-tight",
      className
    ),
    ...props
  }
));
DialogTitle.displayName = DialogPrimitive.Title.displayName;
var DialogDescription = React9.forwardRef(({ className, ...props }, ref) => /* @__PURE__ */ jsx21(
  DialogPrimitive.Description,
  {
    ref,
    className: cn("text-base text-muted-foreground", className),
    ...props
  }
));
DialogDescription.displayName = DialogPrimitive.Description.displayName;

// src/wallet/WalletKeysView.tsx
import { useState as useState4 } from "react";
import { useMutation as useMutation2, useQuery as useQuery3, useQueryClient as useQueryClient2 } from "@tanstack/react-query";
import { Loader2 as Loader23, Plus, Trash2 } from "lucide-react";
import { toast as toast2 } from "sonner";

// src/ui/textarea.tsx
import * as React10 from "react";
import { jsx as jsx22 } from "react/jsx-runtime";
var Textarea = React10.forwardRef(
  ({ className, ...props }, ref) => {
    return /* @__PURE__ */ jsx22(
      "textarea",
      {
        className: cn(
          "flex min-h-[60px] w-full rounded-md border border-ink/15 bg-background-800/90 px-3 py-2 text-xs text-brand-snow shadow-sm placeholder:text-muted-foreground/60 transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-brand-sky/60 focus-visible:border-brand-sky/60 disabled:cursor-not-allowed disabled:opacity-50 [color-scheme:dark]",
          className
        ),
        ref,
        ...props
      }
    );
  }
);
Textarea.displayName = "Textarea";

// src/ui/confirm-dialog.tsx
import { useState as useState3 } from "react";
import { AlertTriangle } from "lucide-react";
import { jsx as jsx23, jsxs as jsxs13 } from "react/jsx-runtime";
function ConfirmDialog({
  open,
  onOpenChange,
  title,
  description,
  children,
  confirmLabel = "Delete",
  destructive = true,
  onConfirm
}) {
  const [pending, setPending] = useState3(false);
  const [error, setError] = useState3(null);
  const change = (next) => {
    if (pending) return;
    if (!next) setError(null);
    onOpenChange(next);
  };
  const confirm = async () => {
    setPending(true);
    setError(null);
    try {
      await onConfirm();
      onOpenChange(false);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setPending(false);
    }
  };
  return /* @__PURE__ */ jsx23(Dialog, { open, onOpenChange: change, children: /* @__PURE__ */ jsxs13(DialogContent, { className: "sm:min-w-[480px] gap-4", children: [
    /* @__PURE__ */ jsxs13(DialogHeader, { children: [
      /* @__PURE__ */ jsxs13(DialogTitle, { className: "flex items-center gap-2", children: [
        destructive && /* @__PURE__ */ jsx23(AlertTriangle, { className: "h-5 w-5 text-danger-600" }),
        title
      ] }),
      description && /* @__PURE__ */ jsx23(DialogDescription, { className: "text-sm", children: description })
    ] }),
    children && /* @__PURE__ */ jsx23("div", { className: "text-sm text-ink/75", children }),
    error && /* @__PURE__ */ jsx23("p", { className: "rounded-md border border-danger-700/40 bg-danger-50 px-3 py-2 text-xs text-danger-700 dark:bg-danger-950/40 dark:text-danger-300", children: error }),
    /* @__PURE__ */ jsxs13(DialogFooter, { children: [
      /* @__PURE__ */ jsx23(Button, { variant: "outline", size: "sm", onClick: () => change(false), disabled: pending, children: "Cancel" }),
      /* @__PURE__ */ jsx23(
        Button,
        {
          variant: destructive ? "destructive" : "default",
          size: "sm",
          onClick: confirm,
          isLoading: pending,
          children: confirmLabel
        }
      )
    ] })
  ] }) });
}

// src/wallet/WalletKeysView.tsx
import { jsx as jsx24, jsxs as jsxs14 } from "react/jsx-runtime";
var WALLET_KEYS_QUERY_KEY = ["/wallet/keys"];
function WalletKeysView() {
  const queryClient = useQueryClient2();
  const [deleting, setDeleting] = useState4(null);
  const {
    data: response,
    isLoading,
    error
  } = useQuery3({
    queryKey: WALLET_KEYS_QUERY_KEY,
    queryFn: () => customInstance("/wallet/keys", { method: "GET" })
  });
  const { data: info } = useQuery3({
    queryKey: ["wallet-info-custom"],
    queryFn: () => customInstance("/wallet/info", { method: "GET" })
  });
  const deleteKey = useMutation2({
    mutationFn: ({ id }) => customInstance(`/wallet/key/${encodeURIComponent(id)}`, { method: "DELETE" })
  });
  const refresh = async () => {
    await queryClient.invalidateQueries({ queryKey: WALLET_KEYS_QUERY_KEY });
    await queryClient.invalidateQueries({ queryKey: ["wallet-keys-list"] });
  };
  const dids = info?.status === 200 ? info.data.dids : [];
  const usedBy = (key) => dids.filter((d) => d.keys.some((k) => k.internal === key.id));
  if (isLoading) {
    return /* @__PURE__ */ jsx24(PageSection, { title: "Keys", children: /* @__PURE__ */ jsx24(Skeleton, { className: "h-64 w-full" }) });
  }
  if (error || response?.status !== 200) {
    return /* @__PURE__ */ jsx24("p", { className: "text-sm text-danger-600", children: "Error loading keys." });
  }
  return /* @__PURE__ */ jsxs14(PageSection, { title: "Keys", action: /* @__PURE__ */ jsx24(ImportKeyDialog, { onImported: refresh }), children: [
    /* @__PURE__ */ jsx24(
      DataTable,
      {
        searchable: false,
        data: response.data,
        keyExtractor: (k) => k.id,
        emptyMessage: "No keys in this wallet",
        emptyDescription: "Import a PEM private key to create DIDs with it.",
        columns: [
          {
            header: "Alias",
            accessorKey: "alias",
            cell: (k) => /* @__PURE__ */ jsx24("span", { className: "font-medium", children: k.alias || "\u2014" })
          },
          { header: "Key ID", cell: (k) => /* @__PURE__ */ jsx24(Identifier, { value: k.id }) },
          { header: "Type", cell: (k) => /* @__PURE__ */ jsx24(Badge, { variant: "info", children: k.kty }) },
          {
            header: "Curve",
            cell: (k) => /* @__PURE__ */ jsx24("span", { className: "font-mono text-xs", children: k.crv ?? "\u2014" })
          },
          {
            header: "Used by",
            cell: (k) => {
              const users = usedBy(k);
              return users.length === 0 ? /* @__PURE__ */ jsx24("span", { className: "text-ink/40 text-xs", children: "No DID" }) : /* @__PURE__ */ jsx24("div", { className: "flex flex-wrap gap-1", children: users.map((d) => /* @__PURE__ */ jsx24(Badge, { variant: "outline", title: d.did, children: d.alias }, d.did)) });
            }
          },
          { header: "Created", cell: (k) => /* @__PURE__ */ jsx24(FormatDate, { date: k.created_at ?? null }) },
          {
            header: "",
            cell: (k) => /* @__PURE__ */ jsx24(
              Button,
              {
                size: "sm",
                variant: "ghost",
                "aria-label": `Delete key ${k.alias || k.id}`,
                disabled: usedBy(k).length > 0,
                title: usedBy(k).length > 0 ? "A DID still uses this key" : "Delete key",
                onClick: () => setDeleting(k),
                children: /* @__PURE__ */ jsx24(Trash2, { className: "h-3.5 w-3.5 text-danger-500" })
              }
            )
          }
        ]
      }
    ),
    /* @__PURE__ */ jsx24(
      ConfirmDialog,
      {
        open: deleting !== null,
        onOpenChange: (open) => !open && setDeleting(null),
        title: `Delete key ${deleting?.alias || deleting?.id}?`,
        description: "The private key is removed from the wallet and cannot be recovered.",
        onConfirm: async () => {
          await deleteKey.mutateAsync({ id: deleting.id });
          toast2.success("Key deleted");
          await refresh();
        }
      }
    )
  ] });
}
var ImportKeyDialog = ({ onImported }) => {
  const [open, setOpen] = useState4(false);
  const [pem, setPem] = useState4("");
  const [alias, setAlias] = useState4("");
  const register = useMutation2({
    mutationFn: (data) => customInstance("/wallet/key", { method: "POST", data })
  });
  const looksLikePem = /-----BEGIN [A-Z ]+-----[\s\S]+-----END [A-Z ]+-----/.test(pem.trim());
  const submit = async () => {
    try {
      const res = await register.mutateAsync({ pem, alias: alias || null });
      if (res.status >= 300) throw new Error("The wallet rejected the key");
      toast2.success("Key imported");
      setOpen(false);
      setPem("");
      setAlias("");
      await onImported();
    } catch (err) {
      toast2.error(err instanceof Error ? err.message : "Failed to import key");
    }
  };
  return /* @__PURE__ */ jsxs14(Dialog, { open, onOpenChange: setOpen, children: [
    /* @__PURE__ */ jsx24(DialogTrigger, { asChild: true, children: /* @__PURE__ */ jsxs14(Button, { size: "sm", children: [
      /* @__PURE__ */ jsx24(Plus, {}),
      "Import key"
    ] }) }),
    /* @__PURE__ */ jsxs14(DialogContent, { className: "max-w-2xl", children: [
      /* @__PURE__ */ jsxs14(DialogHeader, { children: [
        /* @__PURE__ */ jsx24(DialogTitle, { children: "Import a private key" }),
        /* @__PURE__ */ jsx24(DialogDescription, { children: "Paste a PEM private key. Type and curve are read from it." })
      ] }),
      /* @__PURE__ */ jsxs14("div", { className: "space-y-4", children: [
        /* @__PURE__ */ jsxs14("div", { className: "space-y-1.5", children: [
          /* @__PURE__ */ jsx24(Label2, { htmlFor: "key-alias", children: "Alias" }),
          /* @__PURE__ */ jsx24(
            Input,
            {
              id: "key-alias",
              placeholder: "e.g. signing-2026",
              value: alias,
              onChange: (e) => setAlias(e.target.value)
            }
          )
        ] }),
        /* @__PURE__ */ jsxs14("div", { className: "space-y-1.5", children: [
          /* @__PURE__ */ jsx24(Label2, { htmlFor: "key-pem", children: "PEM" }),
          /* @__PURE__ */ jsx24(
            Textarea,
            {
              id: "key-pem",
              className: "min-h-[200px] font-mono text-xs",
              placeholder: "-----BEGIN PRIVATE KEY-----\n\u2026\n-----END PRIVATE KEY-----",
              value: pem,
              onChange: (e) => setPem(e.target.value)
            }
          ),
          pem.trim() && !looksLikePem && /* @__PURE__ */ jsx24("p", { className: "text-xs text-warn-700 dark:text-warn-300", children: "The PEM needs its BEGIN and END lines." })
        ] })
      ] }),
      /* @__PURE__ */ jsxs14(DialogFooter, { children: [
        /* @__PURE__ */ jsx24(Button, { variant: "outline", onClick: () => setOpen(false), children: "Cancel" }),
        /* @__PURE__ */ jsxs14(Button, { disabled: !looksLikePem || register.isPending, onClick: submit, children: [
          register.isPending ? /* @__PURE__ */ jsx24(Loader23, { className: "animate-spin" }) : /* @__PURE__ */ jsx24(Plus, {}),
          "Import"
        ] })
      ] })
    ] })
  ] });
};

// src/wallet/WalletDidView.tsx
import { Fragment as Fragment4, jsx as jsx25, jsxs as jsxs15 } from "react/jsx-runtime";
var INFO_QUERY_KEY = ["wallet-info-custom"];
function reportError(err, fallback) {
  const msg = err instanceof Error ? err.message : fallback;
  toast3.error(msg);
}
function WalletDidView() {
  const queryClient = useQueryClient3();
  const {
    data: response,
    isLoading,
    error
  } = useQuery4({
    queryKey: INFO_QUERY_KEY,
    queryFn: () => customInstance("/wallet/info", { method: "GET" })
  });
  const invalidate = () => queryClient.invalidateQueries({ queryKey: INFO_QUERY_KEY });
  const setDefaultDid = useMutation3({
    mutationFn: (id) => customInstance(`/wallet/did/${encodeURIComponent(id)}/default`, { method: "POST" }),
    onSuccess: async () => {
      toast3.success("Default DID updated");
      await invalidate();
    },
    onError: (err) => reportError(err, "Failed to set default DID")
  });
  if (isLoading) {
    return /* @__PURE__ */ jsx25(PageSection, { title: "DIDs", children: /* @__PURE__ */ jsx25(Skeleton, { className: "h-64 w-full rounded-2xl" }) });
  }
  if (error || response?.status !== 200) {
    return /* @__PURE__ */ jsx25("div", { className: "text-destructive font-mono text-xs", children: "Error loading DIDs" });
  }
  const dids = response.data.dids;
  return /* @__PURE__ */ jsx25("div", { className: "space-y-8 pb-20", children: /* @__PURE__ */ jsx25(PageSection, { title: "DIDs", action: /* @__PURE__ */ jsx25(NewDidDialog, { onCreated: invalidate }), children: dids.length === 0 ? /* @__PURE__ */ jsxs15("div", { className: "flex flex-col items-center justify-center py-20 text-muted-foreground border border-dashed border-ink/10 rounded-2xl bg-ink/2", children: [
    /* @__PURE__ */ jsx25(Key, { className: "h-12 w-12 opacity-10 mb-4" }),
    /* @__PURE__ */ jsx25("p", { className: "text-sm font-medium", children: "No DIDs registered yet." })
  ] }) : /* @__PURE__ */ jsx25("div", { className: "space-y-4", children: dids.map((d) => /* @__PURE__ */ jsx25(
    DidPanel,
    {
      did: d,
      onSetDefault: () => setDefaultDid.mutate(d.id),
      isSettingDefault: setDefaultDid.isPending && setDefaultDid.variables === d.id,
      onChanged: invalidate
    },
    d.did
  )) }) }) });
}
var DidPanel = ({
  did,
  onSetDefault,
  isSettingDefault,
  onChanged
}) => {
  const [isOpen, setIsOpen] = useState5(false);
  const [newKeyId, setNewKeyId] = useState5("");
  const [copied, setCopied] = useState5(false);
  const isWeb = did.type === "Web";
  const addKey = useMutation3({
    mutationFn: (keyId) => customInstance(`/wallet/did/${encodeURIComponent(did.id)}/key/${encodeURIComponent(keyId)}`, {
      method: "POST"
    }),
    onSuccess: async () => {
      toast3.success("Key added to DID");
      setNewKeyId("");
      await onChanged();
    },
    onError: (err) => reportError(err, "Failed to add key")
  });
  const removeKey = useMutation3({
    mutationFn: (keyId) => customInstance(`/wallet/did/${encodeURIComponent(did.id)}/key/${encodeURIComponent(keyId)}`, {
      method: "DELETE"
    }),
    onSuccess: async () => {
      toast3.success("Key removed from DID");
      await onChanged();
    },
    onError: (err) => reportError(err, "Failed to remove key")
  });
  const setDefaultKey = useMutation3({
    mutationFn: (keyId) => customInstance(
      `/wallet/did/${encodeURIComponent(did.id)}/key/default/${encodeURIComponent(keyId)}`,
      { method: "POST" }
    ),
    onSuccess: async () => {
      toast3.success("Default key updated");
      await onChanged();
    },
    onError: (err) => reportError(err, "Failed to set default key")
  });
  const deleteDid = useMutation3({
    mutationFn: () => customInstance(`/wallet/did/${encodeURIComponent(did.id)}`, { method: "DELETE" }),
    onSuccess: async () => {
      toast3.success("DID deleted");
      await onChanged();
    },
    onError: (err) => reportError(err, "Failed to delete DID")
  });
  const formattedDoc = JSON.stringify(did.did_document, null, 2);
  const handleCopy = (e) => {
    e.stopPropagation();
    navigator.clipboard.writeText(formattedDoc);
    setCopied(true);
    setTimeout(() => setCopied(false), 2e3);
  };
  return /* @__PURE__ */ jsxs15("div", { className: "group border border-ink/10 rounded-xl overflow-hidden bg-ink/[0.02] transition-all hover:bg-ink/[0.04]", children: [
    /* @__PURE__ */ jsxs15("div", { className: "w-full flex items-center justify-between p-4 gap-3", children: [
      /* @__PURE__ */ jsxs15(
        "button",
        {
          type: "button",
          onClick: () => setIsOpen(!isOpen),
          className: "flex items-center gap-3 flex-1 min-w-0 text-left",
          children: [
            /* @__PURE__ */ jsx25(
              "div",
              {
                className: cn(
                  "transition-transform duration-200 shrink-0",
                  isOpen ? "rotate-90 text-primary" : "text-muted-foreground"
                ),
                children: /* @__PURE__ */ jsx25(ChevronRight2, { className: "h-5 w-5" })
              }
            ),
            /* @__PURE__ */ jsx25("span", { className: "text-sm font-semibold", children: did.alias }),
            did.default && /* @__PURE__ */ jsx25(Badge, { variant: "default", children: "PRIMARY" }),
            /* @__PURE__ */ jsx25(Badge, { variant: "info", className: "font-mono text-xs", children: did.type }),
            /* @__PURE__ */ jsx25(
              "span",
              {
                className: "text-xs text-muted-foreground/60 font-mono truncate max-w-[320px]",
                title: did.did,
                children: did.did
              }
            )
          ]
        }
      ),
      /* @__PURE__ */ jsxs15("div", { className: "flex items-center gap-2 shrink-0", children: [
        !did.default && /* @__PURE__ */ jsxs15(Button, { size: "sm", variant: "outline", disabled: isSettingDefault, onClick: onSetDefault, children: [
          isSettingDefault ? /* @__PURE__ */ jsx25(Loader24, { className: "h-3 w-3 animate-spin" }) : /* @__PURE__ */ jsx25(Star, { className: "h-3 w-3" }),
          /* @__PURE__ */ jsx25("span", { className: "ml-1", children: "Set default" })
        ] }),
        /* @__PURE__ */ jsx25(
          Button,
          {
            size: "sm",
            variant: "ghost",
            disabled: did.default || deleteDid.isPending,
            onClick: () => deleteDid.mutate(),
            title: did.default ? "Cannot delete the active default DID" : "Delete this DID",
            children: deleteDid.isPending ? /* @__PURE__ */ jsx25(Loader24, { className: "h-3 w-3 animate-spin" }) : /* @__PURE__ */ jsx25(Trash22, { className: "h-3 w-3 text-destructive" })
          }
        )
      ] })
    ] }),
    isOpen && /* @__PURE__ */ jsxs15("div", { className: "p-4 pt-0 space-y-6", children: [
      /* @__PURE__ */ jsxs15("div", { children: [
        /* @__PURE__ */ jsxs15("div", { className: "flex items-center gap-2 mb-3", children: [
          /* @__PURE__ */ jsx25(Key, { className: "h-4 w-4 text-primary" }),
          /* @__PURE__ */ jsx25("h5", { className: "text-xs uppercase tracking-widest font-bold text-foreground/70", children: "Attached Keys" })
        ] }),
        /* @__PURE__ */ jsx25(
          DataTable,
          {
            className: "text-sm",
            data: did.keys,
            keyExtractor: (k) => `${k.internal}-${k.fragment}`,
            hideToolbar: true,
            columns: [
              {
                header: "Fragment",
                accessorKey: "fragment",
                cell: (k) => /* @__PURE__ */ jsxs15("span", { className: "font-mono text-xs", children: [
                  "#",
                  k.fragment
                ] })
              },
              {
                header: "Internal id",
                accessorKey: "internal",
                cell: (k) => /* @__PURE__ */ jsx25("span", { className: "font-mono text-xs text-muted-foreground block max-w-[280px] truncate", children: k.internal })
              },
              {
                header: "Default",
                sortValue: (k) => k.internal === did.default_key?.internal,
                searchable: false,
                cell: (k) => k.internal === did.default_key?.internal ? /* @__PURE__ */ jsx25(Badge, { variant: "success", children: "Default" }) : /* @__PURE__ */ jsx25("span", { className: "text-xs text-muted-foreground/50", children: "\u2014" })
              },
              {
                header: "Actions",
                sortable: false,
                searchable: false,
                cell: (k) => /* @__PURE__ */ jsxs15("div", { className: "flex items-center gap-1", children: [
                  k.internal !== did.default_key?.internal && /* @__PURE__ */ jsx25(
                    Button,
                    {
                      size: "icon",
                      variant: "ghost",
                      className: "h-7 w-7",
                      disabled: !isWeb || setDefaultKey.isPending,
                      onClick: () => setDefaultKey.mutate(k.internal),
                      title: isWeb ? "Set as default signing key" : "did:jwk has a single key \u2014 cannot change default",
                      children: /* @__PURE__ */ jsx25(Star, { className: "h-3 w-3" })
                    }
                  ),
                  /* @__PURE__ */ jsx25(
                    Button,
                    {
                      size: "icon",
                      variant: "ghost",
                      className: "h-7 w-7",
                      disabled: !isWeb || did.keys.length === 1 || removeKey.isPending,
                      onClick: () => removeKey.mutate(k.internal),
                      title: !isWeb ? "did:jwk keys must be removed by deleting the DID itself" : did.keys.length === 1 ? "Cannot remove the only key of a DID" : "Remove key from DID",
                      children: /* @__PURE__ */ jsx25(Trash22, { className: "h-3 w-3 text-destructive" })
                    }
                  )
                ] })
              }
            ]
          }
        ),
        isWeb ? /* @__PURE__ */ jsx25(
          KeyPicker,
          {
            excludeIds: did.keys.map((k) => k.internal),
            value: newKeyId,
            onChange: setNewKeyId,
            disabled: addKey.isPending,
            onSubmit: () => addKey.mutate(newKeyId.trim()),
            isSubmitting: addKey.isPending
          }
        ) : /* @__PURE__ */ jsx25("p", { className: "mt-3 text-xs text-muted-foreground italic", children: "did:jwk DIDs hold a single key bound to their identifier. Create a new DID instead." })
      ] }),
      /* @__PURE__ */ jsx25("div", { children: /* @__PURE__ */ jsxs15("div", { className: "bg-sunken/40 rounded-xl border border-ink/5 overflow-hidden", children: [
        /* @__PURE__ */ jsxs15("div", { className: "flex items-center justify-between px-4 py-2 border-b border-ink/5 bg-ink/[0.02]", children: [
          /* @__PURE__ */ jsxs15("span", { className: "text-xs font-bold uppercase tracking-widest text-primary/60 flex items-center gap-2", children: [
            /* @__PURE__ */ jsx25(FileJson, { className: "h-3 w-3" }),
            "DID Document"
          ] }),
          /* @__PURE__ */ jsxs15(
            "button",
            {
              type: "button",
              onClick: handleCopy,
              className: "flex items-center gap-1.5 text-xs font-mono text-muted-foreground hover:text-primary transition-colors",
              children: [
                copied ? /* @__PURE__ */ jsx25(Check2, { className: "h-3 w-3 text-green-500" }) : /* @__PURE__ */ jsx25(Copy, { className: "h-3 w-3" }),
                copied ? "Copied!" : "Copy"
              ]
            }
          )
        ] }),
        /* @__PURE__ */ jsx25("div", { className: "p-4 overflow-x-auto", children: /* @__PURE__ */ jsx25("pre", { className: "font-mono text-xs text-muted-foreground/90 whitespace-pre-wrap break-all leading-relaxed", children: formattedDoc }) })
      ] }) })
    ] })
  ] });
};
var KeyPicker = ({
  excludeIds,
  value,
  onChange,
  disabled,
  onSubmit,
  isSubmitting
}) => {
  const { data: keysRes } = useQuery4({
    queryKey: WALLET_KEYS_QUERY_KEY,
    queryFn: () => customInstance("/wallet/keys", { method: "GET" })
  });
  const allKeys = keysRes?.status === 200 ? keysRes.data : [];
  const candidates = allKeys.filter((k) => !excludeIds.includes(k.id));
  return /* @__PURE__ */ jsxs15("div", { className: "mt-4 space-y-2", children: [
    /* @__PURE__ */ jsx25(Label2, { className: "text-xs uppercase tracking-widest text-muted-foreground/70", children: "Attach existing key" }),
    /* @__PURE__ */ jsxs15("div", { className: "flex items-center gap-2", children: [
      /* @__PURE__ */ jsxs15(
        Select,
        {
          value,
          onValueChange: onChange,
          disabled: disabled || candidates.length === 0,
          children: [
            /* @__PURE__ */ jsx25(SelectTrigger, { className: "flex-1 font-mono text-xs", children: /* @__PURE__ */ jsx25(SelectValue, { placeholder: "\u2014 Choose a key \u2014" }) }),
            /* @__PURE__ */ jsx25(SelectContent, { children: candidates.map((k) => /* @__PURE__ */ jsx25(SelectItem, { value: k.id, className: "font-mono text-xs", children: k.alias ? `${k.alias} (${k.id})` : k.id }, k.id)) })
          ]
        }
      ),
      /* @__PURE__ */ jsxs15(Button, { size: "sm", disabled: !value || disabled, onClick: onSubmit, children: [
        isSubmitting ? /* @__PURE__ */ jsx25(Loader24, { className: "h-3 w-3 animate-spin" }) : /* @__PURE__ */ jsx25(Plus2, { className: "h-3 w-3" }),
        /* @__PURE__ */ jsx25("span", { className: "ml-1", children: "Add key" })
      ] })
    ] }),
    candidates.length === 0 && /* @__PURE__ */ jsx25("p", { className: "text-xs text-muted-foreground italic", children: "No spare keys available. Create one from the Keys tab." })
  ] });
};
var SERVICE_TYPES = ["AuthorizationServer", "CredentialIssuer", "FederatedCatalog"];
var NewDidDialog = ({ onCreated }) => {
  const [open, setOpen] = useState5(false);
  const [kind, setKind] = useState5("jwk");
  const [alias, setAlias] = useState5("");
  const [domain, setDomain] = useState5("");
  const [path, setPath] = useState5("");
  const [port, setPort] = useState5("");
  const [keysId, setKeysId] = useState5([]);
  const [services, setServices] = useState5([]);
  const { data: keysRes } = useQuery4({
    queryKey: WALLET_KEYS_QUERY_KEY,
    queryFn: () => customInstance("/wallet/keys", { method: "GET" })
  });
  const allKeys = keysRes?.status === 200 ? keysRes.data : [];
  const resetForm = () => {
    setKind("jwk");
    setAlias("");
    setDomain("");
    setPath("");
    setPort("");
    setKeysId([]);
    setServices([]);
  };
  const register = useMutation3({
    mutationFn: () => {
      const builder = kind === "web" ? {
        Web: {
          domain: domain.trim(),
          path: path.trim() ? path.trim() : null,
          port: port.trim() ? port.trim() : null
        }
      } : { Jwk: { pem: "" } };
      const cleanedServices = services.map((s) => ({
        ...s.id ? { id: s.id } : {},
        type: s.type.trim(),
        serviceEndpoint: s.serviceEndpoint.trim()
      })).filter((s) => s.type && s.serviceEndpoint);
      return customInstance("/wallet/did", {
        method: "POST",
        data: {
          builder,
          keys_id: keysId,
          alias,
          service: cleanedServices.length > 0 ? cleanedServices : null
        }
      });
    },
    onSuccess: async () => {
      toast3.success("DID registered");
      setOpen(false);
      resetForm();
      await onCreated();
    },
    onError: (err) => reportError(err, "Failed to register DID")
  });
  const toggleKey = (id) => {
    if (kind === "jwk") {
      setKeysId([id]);
    } else {
      setKeysId((curr) => curr.includes(id) ? curr.filter((k) => k !== id) : [...curr, id]);
    }
  };
  const addService = () => setServices((s) => [...s, { type: "", serviceEndpoint: "" }]);
  const removeService = (idx) => setServices((s) => s.filter((_, i) => i !== idx));
  const patchService = (idx, patch) => setServices((s) => s.map((v, i) => i === idx ? { ...v, ...patch } : v));
  const canSubmit = Boolean(alias.trim()) && keysId.length > 0 && (kind === "jwk" || domain.trim() !== "") && !register.isPending;
  return /* @__PURE__ */ jsxs15(Dialog, { open, onOpenChange: setOpen, children: [
    /* @__PURE__ */ jsx25(DialogTrigger, { asChild: true, children: /* @__PURE__ */ jsxs15(Button, { size: "sm", children: [
      /* @__PURE__ */ jsx25(Plus2, { className: "h-3 w-3" }),
      /* @__PURE__ */ jsx25("span", { className: "ml-1", children: "New DID" })
    ] }) }),
    /* @__PURE__ */ jsxs15(DialogContent, { className: "max-w-2xl max-h-[85vh] overflow-y-auto", children: [
      /* @__PURE__ */ jsxs15(DialogHeader, { children: [
        /* @__PURE__ */ jsx25(DialogTitle, { children: "Register a new DID" }),
        /* @__PURE__ */ jsx25(DialogDescription, { children: "Pick a method, attach an existing wallet key, optionally declare service endpoints." })
      ] }),
      /* @__PURE__ */ jsxs15("div", { className: "space-y-5", children: [
        /* @__PURE__ */ jsxs15("div", { className: "space-y-1", children: [
          /* @__PURE__ */ jsx25(Label2, { className: "text-xs", children: "DID method" }),
          /* @__PURE__ */ jsxs15("div", { className: "flex gap-2", children: [
            /* @__PURE__ */ jsx25(
              Button,
              {
                size: "sm",
                variant: kind === "jwk" ? "default" : "outline",
                onClick: () => {
                  setKind("jwk");
                  setKeysId((curr) => curr.slice(0, 1));
                },
                children: "did:jwk"
              }
            ),
            /* @__PURE__ */ jsx25(
              Button,
              {
                size: "sm",
                variant: kind === "web" ? "default" : "outline",
                onClick: () => setKind("web"),
                children: "did:web"
              }
            )
          ] }),
          /* @__PURE__ */ jsx25("p", { className: "text-xs text-muted-foreground italic", children: kind === "jwk" ? "did:jwk derives the identifier from a single key's public material." : "did:web is hosted at the given URL; you can bind multiple keys." })
        ] }),
        /* @__PURE__ */ jsxs15("div", { className: "space-y-1", children: [
          /* @__PURE__ */ jsx25(Label2, { className: "text-xs", children: "Alias" }),
          /* @__PURE__ */ jsx25(
            Input,
            {
              placeholder: kind === "jwk" ? "my-jwk-did" : "my-web-did",
              value: alias,
              onChange: (e) => setAlias(e.target.value)
            }
          )
        ] }),
        kind === "web" && /* @__PURE__ */ jsxs15(Fragment4, { children: [
          /* @__PURE__ */ jsxs15("div", { className: "space-y-1", children: [
            /* @__PURE__ */ jsx25(Label2, { className: "text-xs", children: "Domain" }),
            /* @__PURE__ */ jsx25(
              Input,
              {
                placeholder: "example.com",
                value: domain,
                onChange: (e) => setDomain(e.target.value)
              }
            )
          ] }),
          /* @__PURE__ */ jsxs15("div", { className: "grid grid-cols-2 gap-3", children: [
            /* @__PURE__ */ jsxs15("div", { className: "space-y-1", children: [
              /* @__PURE__ */ jsx25(Label2, { className: "text-xs", children: "Path (optional)" }),
              /* @__PURE__ */ jsx25(
                Input,
                {
                  placeholder: "agents/01",
                  value: path,
                  onChange: (e) => setPath(e.target.value)
                }
              )
            ] }),
            /* @__PURE__ */ jsxs15("div", { className: "space-y-1", children: [
              /* @__PURE__ */ jsx25(Label2, { className: "text-xs", children: "Port (optional)" }),
              /* @__PURE__ */ jsx25(
                Input,
                {
                  placeholder: "8443",
                  value: port,
                  onChange: (e) => setPort(e.target.value)
                }
              )
            ] })
          ] })
        ] }),
        /* @__PURE__ */ jsxs15("div", { className: "space-y-1", children: [
          /* @__PURE__ */ jsx25(Label2, { className: "text-xs", children: kind === "jwk" ? "Bind key (single)" : "Attach keys (one or more)" }),
          allKeys.length === 0 ? /* @__PURE__ */ jsx25("p", { className: "text-xs text-muted-foreground italic", children: "No keys available. Create one from the Keys tab." }) : /* @__PURE__ */ jsx25("div", { className: "space-y-1 max-h-40 overflow-y-auto border border-ink/10 rounded-lg p-2", children: allKeys.map((k) => /* @__PURE__ */ jsxs15(
            "label",
            {
              className: "flex items-center gap-2 p-1 text-xs cursor-pointer hover:bg-ink/5 rounded",
              children: [
                kind === "jwk" ? /* @__PURE__ */ jsx25(
                  "input",
                  {
                    type: "radio",
                    name: "did-keys",
                    checked: keysId.includes(k.id),
                    onChange: () => toggleKey(k.id),
                    className: "accent-primary h-3.5 w-3.5 [color-scheme:dark]"
                  }
                ) : /* @__PURE__ */ jsx25(
                  Checkbox,
                  {
                    checked: keysId.includes(k.id),
                    onCheckedChange: () => toggleKey(k.id)
                  }
                ),
                /* @__PURE__ */ jsx25("span", { className: "font-mono", children: k.alias || k.id }),
                /* @__PURE__ */ jsx25("span", { className: "font-mono text-xs text-muted-foreground truncate", children: k.id })
              ]
            },
            k.id
          )) })
        ] }),
        /* @__PURE__ */ jsxs15("div", { className: "space-y-2", children: [
          /* @__PURE__ */ jsxs15("div", { className: "flex items-center justify-between", children: [
            /* @__PURE__ */ jsx25(Label2, { className: "text-xs", children: "Services (optional)" }),
            /* @__PURE__ */ jsxs15(Button, { size: "sm", variant: "ghost", onClick: addService, children: [
              /* @__PURE__ */ jsx25(Plus2, { className: "h-3 w-3" }),
              /* @__PURE__ */ jsx25("span", { className: "ml-1", children: "Add service" })
            ] })
          ] }),
          services.length === 0 ? /* @__PURE__ */ jsx25("p", { className: "text-xs text-muted-foreground italic", children: "Declare any service endpoint to expose in the DID document." }) : /* @__PURE__ */ jsx25("div", { className: "space-y-2", children: services.map((svc, idx) => /* @__PURE__ */ jsxs15(
            "div",
            {
              className: "border border-ink/10 rounded-lg p-3 space-y-2 bg-ink/[0.02]",
              children: [
                /* @__PURE__ */ jsxs15("div", { className: "flex items-center gap-2", children: [
                  /* @__PURE__ */ jsxs15("div", { className: "flex-1 space-y-1", children: [
                    /* @__PURE__ */ jsx25(Label2, { className: "text-xs text-muted-foreground", children: "Type" }),
                    /* @__PURE__ */ jsx25(
                      Input,
                      {
                        list: `service-types-${idx}`,
                        placeholder: "AuthorizationServer / CredentialIssuer / ...",
                        value: svc.type,
                        onChange: (e) => patchService(idx, { type: e.target.value }),
                        className: "font-mono text-xs"
                      }
                    ),
                    /* @__PURE__ */ jsx25("datalist", { id: `service-types-${idx}`, children: SERVICE_TYPES.map((t) => /* @__PURE__ */ jsx25("option", { value: t }, t)) })
                  ] }),
                  /* @__PURE__ */ jsx25(
                    Button,
                    {
                      size: "sm",
                      variant: "ghost",
                      onClick: () => removeService(idx),
                      className: "mt-4",
                      children: /* @__PURE__ */ jsx25(Trash22, { className: "h-3 w-3 text-destructive" })
                    }
                  )
                ] }),
                /* @__PURE__ */ jsxs15("div", { className: "space-y-1", children: [
                  /* @__PURE__ */ jsx25(Label2, { className: "text-xs text-muted-foreground", children: "Service Endpoint" }),
                  /* @__PURE__ */ jsx25(
                    Input,
                    {
                      placeholder: "https://example.com/oidc",
                      value: svc.serviceEndpoint,
                      onChange: (e) => patchService(idx, { serviceEndpoint: e.target.value })
                    }
                  )
                ] })
              ]
            },
            idx
          )) })
        ] })
      ] }),
      /* @__PURE__ */ jsxs15(DialogFooter, { children: [
        /* @__PURE__ */ jsx25(Button, { variant: "outline", onClick: () => setOpen(false), children: "Cancel" }),
        /* @__PURE__ */ jsxs15(Button, { disabled: !canSubmit, onClick: () => register.mutate(), children: [
          register.isPending ? /* @__PURE__ */ jsx25(Loader24, { className: "h-3 w-3 animate-spin mr-1" }) : /* @__PURE__ */ jsx25(Plus2, { className: "h-3 w-3 mr-1" }),
          "Register DID"
        ] })
      ] })
    ] })
  ] });
};

// src/wallet/WalletCredentialsView.tsx
import { useState as useState6 } from "react";
import { useQuery as useQuery5, useQueryClient as useQueryClient4 } from "@tanstack/react-query";
import {
  ArrowRight,
  Building2,
  Calendar,
  Check as Check3,
  ChevronDown,
  ChevronUp,
  FileJson as FileJson2,
  Fingerprint,
  Loader2 as Loader25,
  ShieldAlert,
  ShieldCheck,
  Sparkles,
  Trash2 as Trash23
} from "lucide-react";
import { toast as toast4 } from "sonner";
import { Fragment as Fragment5, jsx as jsx26, jsxs as jsxs16 } from "react/jsx-runtime";
var REGISTRATION_TYPES = [
  "gx:Eori",
  "gx:Euid",
  "gx:LeiCode",
  "gx:LocalRegistrationNumber",
  "gx:TaxId",
  "gx:VatId",
  "Eori",
  "TaxId",
  "VatId"
];
var LEGAL_PERSON_TYPES = ["gx:LegalPerson", "LegalPerson"];
var TC_TYPES = ["gx:TermsAndConditions", "TermsAndConditions"];
var LABEL_TYPES = ["gx:LabelCredential", "LabelCredential"];
var RECOMMENDED_REGISTRATION_VC = "gx_VatId_jwt_vc_json";
var RECOMMENDED_COMPLIANCE_VC = "gx_LabelCredential_jwt_vc_json";
function vcHasAny(vcs, targets) {
  return vcs.some((vc) => {
    const types = vc.parsed_document?.type || [];
    return types.some((t) => targets.includes(t));
  });
}
function WalletCredentialsView({
  showGaiaCompliance = true,
  requestVcHref = (rec) => `/authority/new?recommended=${encodeURIComponent(rec)}`
} = {}) {
  const queryClient = useQueryClient4();
  const {
    data: response,
    isLoading,
    error
  } = useQuery5({
    queryKey: ["wallet-vcs-custom"],
    queryFn: () => customInstance("/wallet/vcs", { method: "GET" })
  });
  const [isGenerating, setIsGenerating] = useState6(false);
  const credentials = response?.status === 200 ? response.data : [];
  const hasRegistration = vcHasAny(credentials, REGISTRATION_TYPES);
  const hasLegalPerson = vcHasAny(credentials, LEGAL_PERSON_TYPES);
  const hasTermsAndConditions = vcHasAny(credentials, TC_TYPES);
  const hasLabel = vcHasAny(credentials, LABEL_TYPES);
  const step = hasLabel ? 4 : hasRegistration && hasLegalPerson && hasTermsAndConditions ? 3 : hasRegistration ? 2 : 1;
  const handleGenerateGaia = async () => {
    setIsGenerating(true);
    try {
      await customInstance("/gaia/generate", { method: "POST" });
      toast4.success("Gaia-X self-issued credentials generated");
      await queryClient.invalidateQueries({ queryKey: ["wallet-vcs-custom"] });
    } catch (err) {
      const msg = err instanceof Error ? err.message : "Failed to generate Gaia-X credentials";
      toast4.error(msg);
    } finally {
      setIsGenerating(false);
    }
  };
  const handleDeleteVc = async (id) => {
    try {
      await customInstance(`/wallet/credential/${encodeURIComponent(id)}`, { method: "DELETE" });
      toast4.success("Credential deleted");
      await queryClient.invalidateQueries({ queryKey: ["wallet-vcs-custom"] });
    } catch (err) {
      const msg = err instanceof Error ? err.message : "Failed to delete credential";
      toast4.error(msg);
    }
  };
  if (isLoading) {
    return /* @__PURE__ */ jsx26(PageSection, { title: "Verifiable Credentials", children: /* @__PURE__ */ jsxs16("div", { className: "grid grid-cols-1 md:grid-cols-2 gap-6", children: [
      /* @__PURE__ */ jsx26(Skeleton, { className: "h-48 w-full rounded-xl" }),
      /* @__PURE__ */ jsx26(Skeleton, { className: "h-48 w-full rounded-xl" })
    ] }) });
  }
  if (error) {
    return /* @__PURE__ */ jsxs16("div", { className: "flex flex-col items-center justify-center py-20 text-destructive font-mono text-xs", children: [
      /* @__PURE__ */ jsx26(ShieldAlert, { className: "h-8 w-8 mb-2" }),
      "Error loading credentials"
    ] });
  }
  return /* @__PURE__ */ jsxs16("div", { className: "space-y-8 pb-20", children: [
    showGaiaCompliance && /* @__PURE__ */ jsxs16(PageSection, { title: "Gaia-X Compliance", children: [
      /* @__PURE__ */ jsx26(StepTrail, { step }),
      /* @__PURE__ */ jsx26(
        ComplianceStepCard,
        {
          step,
          isGenerating,
          onGenerate: handleGenerateGaia,
          requestVcHref,
          flags: { hasRegistration, hasLegalPerson, hasTermsAndConditions, hasLabel }
        }
      )
    ] }),
    /* @__PURE__ */ jsx26(PageSection, { title: "Verifiable Credentials", children: credentials.length === 0 ? /* @__PURE__ */ jsxs16("div", { className: "flex flex-col items-center justify-center py-20 text-muted-foreground border border-dashed border-ink/10 rounded-2xl bg-ink/2", children: [
      /* @__PURE__ */ jsx26(ShieldAlert, { className: "h-12 w-12 opacity-10 mb-4" }),
      /* @__PURE__ */ jsx26("p", { className: "text-sm font-medium", children: "No credentials found in this wallet." }),
      /* @__PURE__ */ jsx26("p", { className: "text-xs opacity-60", children: "Your claimed credentials will appear here." })
    ] }) : /* @__PURE__ */ jsx26("div", { className: "grid grid-cols-1 gap-6", children: credentials.map((vc, idx) => /* @__PURE__ */ jsx26(CredentialCard, { vc, onDelete: handleDeleteVc }, vc.id || idx)) }) })
  ] });
}
var CredentialCard = ({
  vc,
  onDelete
}) => {
  const [isExpanded, setIsExpanded] = useState6(false);
  const [isDeleting, setIsDeleting] = useState6(false);
  const parsed = vc.parsed_document || {};
  const issuerName = parsed.issuer?.name || parsed.issuer?.id || "Unknown Issuer";
  const types = (parsed.type || []).filter((t) => t !== "VerifiableCredential");
  const displayType = types.length > 0 ? types[types.length - 1] : "Credential";
  const addedRaw = vc.added_on ?? vc.addedOn ?? null;
  const addedDate = /* @__PURE__ */ jsx26(FormatDate, { date: addedRaw, format: DATE_FORMAT });
  const validRaw = vc.valid_until ?? parsed.validUntil ?? parsed.expirationDate;
  const validUntil = validRaw ? /* @__PURE__ */ jsx26(FormatDate, { date: validRaw, format: DATE_FORMAT }) : "Never";
  const handleDelete = async (e) => {
    e.stopPropagation();
    setIsDeleting(true);
    await onDelete(vc.id);
    setIsDeleting(false);
  };
  return /* @__PURE__ */ jsxs16(
    "div",
    {
      className: cn(
        "group relative overflow-hidden bg-ink/[0.03] border border-ink/10 rounded-2xl transition-all duration-300",
        isExpanded ? "border-primary/40 bg-ink/[0.06] ring-1 ring-primary/20" : "hover:border-ink/20 hover:bg-ink/[0.05]"
      ),
      children: [
        /* @__PURE__ */ jsxs16(
          "div",
          {
            onClick: () => setIsExpanded(!isExpanded),
            className: "cursor-pointer p-6 flex flex-col md:flex-row md:items-center justify-between gap-4",
            children: [
              /* @__PURE__ */ jsxs16("div", { className: "flex items-start gap-4", children: [
                /* @__PURE__ */ jsx26(
                  "div",
                  {
                    className: cn(
                      "p-3 rounded-xl transition-colors duration-300",
                      isExpanded ? "bg-primary/20 text-primary" : "bg-ink/5 text-muted-foreground group-hover:bg-ink/10"
                    ),
                    children: /* @__PURE__ */ jsx26(Fingerprint, { className: "h-6 w-6" })
                  }
                ),
                /* @__PURE__ */ jsxs16("div", { className: "space-y-1", children: [
                  /* @__PURE__ */ jsxs16("div", { className: "flex items-center gap-2", children: [
                    /* @__PURE__ */ jsx26("h4", { className: "font-bold text-lg tracking-tight", children: displayType }),
                    types.length > 1 && /* @__PURE__ */ jsxs16(Badge, { variant: "info", className: "text-xs h-4 py-0 font-mono opacity-60", children: [
                      "+",
                      types.length - 1,
                      " more"
                    ] })
                  ] }),
                  /* @__PURE__ */ jsxs16("div", { className: "flex flex-wrap items-center gap-y-1 gap-x-4 text-sm text-muted-foreground/80", children: [
                    /* @__PURE__ */ jsxs16("span", { className: "flex items-center gap-1.5", children: [
                      /* @__PURE__ */ jsx26(Building2, { className: "h-3.5 w-3.5 opacity-60" }),
                      issuerName
                    ] }),
                    /* @__PURE__ */ jsxs16("span", { className: "flex items-center gap-1.5 font-mono text-xs opacity-60", children: [
                      "ID: ",
                      vc.id?.substring(0, 16),
                      "..."
                    ] })
                  ] })
                ] })
              ] }),
              /* @__PURE__ */ jsxs16("div", { className: "flex items-center justify-between md:justify-end gap-6 border-t md:border-t-0 pt-4 md:pt-0 border-ink/5", children: [
                /* @__PURE__ */ jsxs16("div", { className: "flex items-center gap-4 text-xs text-muted-foreground/60", children: [
                  /* @__PURE__ */ jsxs16("div", { className: "flex flex-col items-end", children: [
                    /* @__PURE__ */ jsx26("span", { className: "uppercase text-xs font-bold tracking-widest opacity-40", children: "Added" }),
                    /* @__PURE__ */ jsxs16("span", { className: "flex items-center gap-1 mt-0.5", children: [
                      /* @__PURE__ */ jsx26(Calendar, { className: "h-3 w-3" }),
                      addedDate
                    ] })
                  ] }),
                  /* @__PURE__ */ jsxs16("div", { className: "flex flex-col items-end", children: [
                    /* @__PURE__ */ jsx26("span", { className: "uppercase text-xs font-bold tracking-widest opacity-40", children: "Expires" }),
                    /* @__PURE__ */ jsxs16("span", { className: "flex items-center gap-1 mt-0.5", children: [
                      /* @__PURE__ */ jsx26(ShieldCheck, { className: "h-3 w-3" }),
                      validUntil
                    ] })
                  ] })
                ] }),
                /* @__PURE__ */ jsx26("div", { className: "text-muted-foreground/40 group-hover:text-primary/60 transition-colors", children: isExpanded ? /* @__PURE__ */ jsx26(ChevronUp, { className: "h-5 w-5" }) : /* @__PURE__ */ jsx26(ChevronDown, { className: "h-5 w-5" }) })
              ] })
            ]
          }
        ),
        /* @__PURE__ */ jsx26(
          "div",
          {
            className: cn(
              "grid transition-all duration-500 ease-in-out",
              isExpanded ? "grid-rows-[1fr] border-t border-ink/10" : "grid-rows-[0fr]"
            ),
            children: /* @__PURE__ */ jsx26("div", { className: "overflow-hidden", children: /* @__PURE__ */ jsxs16("div", { className: "p-6 space-y-4", children: [
              /* @__PURE__ */ jsxs16("div", { className: "flex items-center justify-between", children: [
                /* @__PURE__ */ jsxs16("span", { className: "text-xs font-bold uppercase tracking-widest text-primary/60 flex items-center gap-2", children: [
                  /* @__PURE__ */ jsx26(FileJson2, { className: "h-3 w-3" }),
                  "Raw Credential Document"
                ] }),
                /* @__PURE__ */ jsxs16("div", { className: "flex items-center gap-2", children: [
                  /* @__PURE__ */ jsxs16(
                    Button,
                    {
                      variant: "ghost",
                      size: "sm",
                      className: "h-7 text-xs font-mono hover:bg-destructive/10 hover:text-destructive",
                      onClick: handleDelete,
                      disabled: isDeleting,
                      children: [
                        isDeleting ? /* @__PURE__ */ jsx26(Loader25, { className: "h-3 w-3 mr-1 animate-spin" }) : /* @__PURE__ */ jsx26(Trash23, { className: "h-3 w-3 mr-1" }),
                        "Delete"
                      ]
                    }
                  ),
                  /* @__PURE__ */ jsx26(
                    Button,
                    {
                      variant: "ghost",
                      size: "sm",
                      className: "h-7 text-xs font-mono hover:bg-primary/10 hover:text-primary",
                      onClick: (e) => {
                        e.stopPropagation();
                        navigator.clipboard.writeText(JSON.stringify(vc, null, 2));
                      },
                      children: "Copy JSON"
                    }
                  )
                ] })
              ] }),
              /* @__PURE__ */ jsx26("div", { className: "relative rounded-xl bg-sunken/40 border border-ink/5 p-4 font-mono text-xs leading-relaxed text-muted-foreground/90 overflow-x-auto shadow-inner", children: /* @__PURE__ */ jsx26("pre", { className: "whitespace-pre-wrap break-all", children: JSON.stringify(vc, null, 2) }) })
            ] }) })
          }
        )
      ]
    }
  );
};
var ComplianceStepCard = ({
  step,
  isGenerating,
  onGenerate,
  requestVcHref,
  flags
}) => {
  const wrapper = cn(
    "relative overflow-hidden p-6 rounded-2xl border transition-all duration-300",
    step === 4 ? "bg-green-500/10 border-green-500/20 shadow-lg" : step === 1 ? "bg-ink/2 border-ink/5 opacity-90" : "bg-primary/5 border-primary/20 shadow-lg shadow-primary/5"
  );
  return /* @__PURE__ */ jsxs16("div", { className: wrapper, children: [
    /* @__PURE__ */ jsxs16("div", { className: "flex flex-col md:flex-row md:items-center justify-between gap-6", children: [
      /* @__PURE__ */ jsxs16("div", { className: "space-y-2", children: [
        /* @__PURE__ */ jsxs16("div", { className: "flex items-center gap-2", children: [
          /* @__PURE__ */ jsx26(
            ShieldCheck,
            {
              className: cn(
                "h-5 w-5",
                step === 4 ? "text-green-500" : step === 1 ? "text-muted-foreground" : "text-primary"
              )
            }
          ),
          /* @__PURE__ */ jsx26("h3", { className: "font-semibold text-lg", children: stepTitle(step) }),
          /* @__PURE__ */ jsxs16(Badge, { variant: "info", className: "font-mono text-xs", children: [
            "Step ",
            step,
            " of 4"
          ] })
        ] }),
        /* @__PURE__ */ jsx26("p", { className: "text-sm text-muted-foreground max-w-xl", children: stepDescription(step) })
      ] }),
      /* @__PURE__ */ jsx26(
        StepCta,
        {
          step,
          isGenerating,
          onGenerate,
          requestVcHref
        }
      )
    ] }),
    step === 1 && /* @__PURE__ */ jsx26(
      Hint,
      {
        tone: "amber",
        text: "Request a registration credential (VatId, TaxId or LeiCode) from a legal authority."
      }
    ),
    step === 2 && !flags.hasLegalPerson && !flags.hasTermsAndConditions && /* @__PURE__ */ jsx26(
      Hint,
      {
        tone: "primary",
        text: "The wallet will self-issue gx:LegalPerson and gx:TermsAndConditions on top of your registration credential."
      }
    ),
    step === 3 && /* @__PURE__ */ jsx26(
      Hint,
      {
        tone: "primary",
        text: "Ask a clearing house to issue a LabelCredential signing your self-attested data."
      }
    )
  ] });
};
var StepCta = ({
  step,
  isGenerating,
  onGenerate,
  requestVcHref
}) => {
  const Link = useLink();
  if (step === 1) {
    return /* @__PURE__ */ jsx26(Link, { to: requestVcHref(RECOMMENDED_REGISTRATION_VC), children: /* @__PURE__ */ jsxs16(Button, { className: "md:min-w-[220px]", children: [
      "Request VC",
      /* @__PURE__ */ jsx26(ArrowRight, { className: "ml-2 h-4 w-4" })
    ] }) });
  }
  if (step === 2) {
    return /* @__PURE__ */ jsx26(Button, { disabled: isGenerating, onClick: onGenerate, className: "md:min-w-[220px]", children: isGenerating ? /* @__PURE__ */ jsxs16(Fragment5, { children: [
      /* @__PURE__ */ jsx26(Loader25, { className: "mr-2 h-4 w-4 animate-spin" }),
      "Generating..."
    ] }) : /* @__PURE__ */ jsxs16(Fragment5, { children: [
      /* @__PURE__ */ jsx26(Sparkles, { className: "mr-2 h-4 w-4" }),
      "Auto-generate VCs"
    ] }) });
  }
  if (step === 3) {
    return /* @__PURE__ */ jsx26(Link, { to: requestVcHref(RECOMMENDED_COMPLIANCE_VC), children: /* @__PURE__ */ jsxs16(Button, { className: "md:min-w-[220px]", children: [
      "Request Compliance",
      /* @__PURE__ */ jsx26(ArrowRight, { className: "ml-2 h-4 w-4" })
    ] }) });
  }
  return /* @__PURE__ */ jsxs16(Button, { variant: "outline", disabled: true, className: "md:min-w-[220px]", children: [
    /* @__PURE__ */ jsx26(Check3, { className: "mr-2 h-4 w-4 text-green-500" }),
    "Compliance Active"
  ] });
};
function stepTitle(step) {
  switch (step) {
    case 1:
      return "Get your legal identity credential";
    case 2:
      return "Self-issue Gaia-X base credentials";
    case 3:
      return "Request the compliance credential";
    case 4:
      return "You are Gaia-X compliant";
  }
}
function stepDescription(step) {
  switch (step) {
    case 1:
      return "Pick a registration credential (VatId, TaxId, LEI, EORI, ...) from a trusted authority. This anchors your legal identity.";
    case 2:
      return "Generate self-signed gx:LegalPerson and gx:TermsAndConditions credentials linked to your registration credential.";
    case 3:
      return "Submit your credentials to a Gaia-X Clearing House to receive your LabelCredential.";
    case 4:
      return "Your wallet holds a valid Gaia-X LabelCredential and is ready to participate in Gaia-X data spaces.";
  }
}
var Hint = ({ tone, text }) => {
  const cls = tone === "amber" ? "bg-warn-500/10 border-warn-500/30 text-warn-700 dark:text-warn-300" : "bg-primary/10 border-primary/20 text-primary";
  return /* @__PURE__ */ jsxs16("div", { className: cn("mt-4 p-3 rounded-lg text-sm flex items-center gap-2 border", cls), children: [
    /* @__PURE__ */ jsx26(ShieldAlert, { className: "h-4 w-4 shrink-0" }),
    text
  ] });
};
var StepTrail = ({ step }) => {
  const items = [
    { n: 1, label: "Registration" },
    { n: 2, label: "Self-issued" },
    { n: 3, label: "Compliance" },
    { n: 4, label: "Compliant" }
  ];
  return /* @__PURE__ */ jsx26("div", { className: "mb-3 flex items-center gap-2", children: items.map((it, idx) => {
    const done = it.n < step;
    const current = it.n === step;
    return /* @__PURE__ */ jsxs16("div", { className: "flex items-center gap-2", children: [
      /* @__PURE__ */ jsx26(
        "div",
        {
          className: cn(
            "h-6 w-6 rounded-full flex items-center justify-center text-xs font-bold border",
            done ? "bg-green-500/20 border-green-500/40 text-green-500" : current ? "bg-primary/20 border-primary/40 text-primary" : "bg-ink/5 border-ink/10 text-muted-foreground"
          ),
          children: done ? /* @__PURE__ */ jsx26(Check3, { className: "h-3 w-3" }) : it.n
        }
      ),
      /* @__PURE__ */ jsx26(
        "span",
        {
          className: cn(
            "text-xs uppercase tracking-widest",
            current ? "text-primary font-bold" : "text-muted-foreground"
          ),
          children: it.label
        }
      ),
      idx < items.length - 1 && /* @__PURE__ */ jsx26("div", { className: "h-px w-6 bg-ink/10", "aria-hidden": true })
    ] }, it.n);
  }) });
};

// src/wallet/WalletOidc4vpView.tsx
import { useState as useState7 } from "react";
import { useMutation as useMutation4 } from "@tanstack/react-query";
import { Loader2 as Loader26, ShieldCheck as ShieldCheck2 } from "lucide-react";
import { toast as toast5 } from "sonner";
import { jsx as jsx27, jsxs as jsxs17 } from "react/jsx-runtime";
function WalletOidc4vpView() {
  const [uri, setUri] = useState7("");
  const {
    mutateAsync: processVp,
    isPending,
    isSuccess,
    error
  } = useMutation4({
    mutationFn: (data) => customInstance("/wallet/oid4vp", { method: "POST", data }),
    onSuccess: () => toast5.success("Presentation submitted successfully"),
    onError: (err) => {
      const msg = err instanceof Error ? err.message : "Failed to process presentation request";
      toast5.error(msg);
    }
  });
  const handleProcess = async (e) => {
    e.preventDefault();
    if (!uri) return;
    try {
      await processVp({ uri });
      setUri("");
    } catch {
    }
  };
  return /* @__PURE__ */ jsx27(PageSection, { title: "Present credentials (OID4VP)", children: /* @__PURE__ */ jsxs17(
    "form",
    {
      onSubmit: handleProcess,
      className: "max-w-2xl space-y-4 rounded-lg border border-ink/10 p-5",
      children: [
        /* @__PURE__ */ jsxs17("div", { className: "flex items-start gap-3", children: [
          /* @__PURE__ */ jsx27(ShieldCheck2, { className: "mt-0.5 h-5 w-5 shrink-0 text-ink/40" }),
          /* @__PURE__ */ jsx27("p", { className: "text-sm text-ink/70", children: "Paste the OID4VP request URI a verifier gave you to present credentials from this wallet." })
        ] }),
        /* @__PURE__ */ jsxs17("div", { className: "space-y-1.5", children: [
          /* @__PURE__ */ jsx27(Label2, { htmlFor: "oidc4vp-uri", children: "Request URI" }),
          /* @__PURE__ */ jsx27(
            Input,
            {
              id: "oidc4vp-uri",
              placeholder: "openid-vc://\u2026",
              value: uri,
              onChange: (e) => setUri(e.target.value),
              className: "font-mono text-xs"
            }
          )
        ] }),
        /* @__PURE__ */ jsxs17("div", { className: "flex items-center gap-3", children: [
          /* @__PURE__ */ jsxs17(Button, { type: "submit", disabled: isPending || !uri, children: [
            isPending && /* @__PURE__ */ jsx27(Loader26, { className: "animate-spin" }),
            "Present credentials"
          ] }),
          isSuccess && /* @__PURE__ */ jsx27("span", { className: "text-sm text-success-700 dark:text-success-300", children: "Presentation sent to the verifier." }),
          !!error && /* @__PURE__ */ jsx27("span", { className: "text-sm text-danger-700 dark:text-danger-300", children: "The presentation request could not be processed." })
        ] })
      ]
    }
  ) });
}

// src/wallet/WalletOidc4vciView.tsx
import { useState as useState8 } from "react";
import { useMutation as useMutation5, useQueryClient as useQueryClient5 } from "@tanstack/react-query";
import { FileText, Loader2 as Loader27 } from "lucide-react";
import { toast as toast6 } from "sonner";
import { jsx as jsx28, jsxs as jsxs18 } from "react/jsx-runtime";
function WalletOidc4vciView() {
  const [uri, setUri] = useState8("");
  const queryClient = useQueryClient5();
  const {
    mutateAsync: processVci,
    isPending,
    isSuccess,
    error
  } = useMutation5({
    mutationFn: (data) => customInstance("/wallet/oid4vci", { method: "POST", data }),
    onSuccess: async () => {
      toast6.success("Credential issued successfully");
      await queryClient.invalidateQueries({ queryKey: ["wallet-vcs-custom"] });
    },
    onError: (err) => {
      const msg = err instanceof Error ? err.message : "Failed to process credential offer";
      toast6.error(msg);
    }
  });
  const handleProcess = async (e) => {
    e.preventDefault();
    if (!uri) return;
    try {
      await processVci({ uri });
      setUri("");
    } catch {
    }
  };
  return /* @__PURE__ */ jsx28(PageSection, { title: "Receive a credential (OID4VCI)", children: /* @__PURE__ */ jsxs18(
    "form",
    {
      onSubmit: handleProcess,
      className: "max-w-2xl space-y-4 rounded-lg border border-ink/10 p-5",
      children: [
        /* @__PURE__ */ jsxs18("div", { className: "flex items-start gap-3", children: [
          /* @__PURE__ */ jsx28(FileText, { className: "mt-0.5 h-5 w-5 shrink-0 text-ink/40" }),
          /* @__PURE__ */ jsx28("p", { className: "text-sm text-ink/70", children: "Paste the credential offer URI an issuer gave you to add the credential to this wallet." })
        ] }),
        /* @__PURE__ */ jsxs18("div", { className: "space-y-1.5", children: [
          /* @__PURE__ */ jsx28(Label2, { htmlFor: "oidc4vci-uri", children: "Credential offer URI" }),
          /* @__PURE__ */ jsx28(
            Input,
            {
              id: "oidc4vci-uri",
              placeholder: "openid-credential-offer://\u2026",
              value: uri,
              onChange: (e) => setUri(e.target.value),
              className: "font-mono text-xs"
            }
          )
        ] }),
        /* @__PURE__ */ jsxs18("div", { className: "flex items-center gap-3", children: [
          /* @__PURE__ */ jsxs18(Button, { type: "submit", disabled: isPending || !uri, children: [
            isPending && /* @__PURE__ */ jsx28(Loader27, { className: "animate-spin" }),
            "Request credential"
          ] }),
          isSuccess && /* @__PURE__ */ jsx28("span", { className: "text-sm text-success-700 dark:text-success-300", children: "Credential added to the wallet." }),
          !!error && /* @__PURE__ */ jsx28("span", { className: "text-sm text-danger-700 dark:text-danger-300", children: "The credential offer could not be processed." })
        ] })
      ]
    }
  ) });
}

export {
  LinkProvider,
  ActivePathProvider,
  useLink,
  useActivePath,
  NavigationProvider,
  mapToSortParam,
  useTableQueryParams,
  UNAUTHORIZED_EVENT,
  setApiGatewayBase,
  getApiGatewayBase,
  customInstance,
  badgeVariants,
  Badge,
  buttonVariants,
  Button,
  Checkbox,
  Dialog,
  DialogTrigger,
  DialogPortal,
  DialogClose,
  DialogOverlay,
  DialogContent,
  DialogHeader,
  DialogBody,
  DialogFooter,
  DialogTitle,
  DialogDescription,
  ConfirmDialog,
  CountBadge,
  Label2 as Label,
  DATE_TIME_FORMAT,
  DATE_FORMAT,
  FormatDate,
  RelativeTime,
  heading_default,
  Input,
  Select,
  SelectGroup,
  SelectValue,
  SelectTrigger,
  SelectScrollUpButton,
  SelectScrollDownButton,
  SelectContent,
  SelectLabel,
  SelectItem,
  SelectSeparator,
  Skeleton,
  Table,
  TableHeader,
  TableBody,
  TableFooter,
  TableRow,
  TableHead,
  TableCell,
  TableCaption,
  tabListClass,
  tabTriggerClass,
  tabActiveClass,
  RouteTabs,
  Textarea,
  FieldGrid,
  PageHeader,
  PageLayout,
  PageSection,
  DataTable,
  WalletLayout,
  WalletInfoView,
  WALLET_KEYS_QUERY_KEY,
  WalletKeysView,
  WalletDidView,
  WalletCredentialsView,
  WalletOidc4vpView,
  WalletOidc4vciView
};
//# sourceMappingURL=chunk-JSO64RSZ.js.map