import * as react_jsx_runtime from 'react/jsx-runtime';
import * as React from 'react';
import React__default, { ReactNode, FC } from 'react';
import { ClassValue } from 'clsx';
import * as AccordionPrimitive from '@radix-ui/react-accordion';
import * as class_variance_authority_types from 'class-variance-authority/types';
import { VariantProps } from 'class-variance-authority';
import { SpringOptions } from 'motion/react';
import * as RechartsPrimitive from 'recharts';
import * as CheckboxPrimitive from '@radix-ui/react-checkbox';
import * as DialogPrimitive from '@radix-ui/react-dialog';
import { DialogProps } from '@radix-ui/react-dialog';
import { Drawer as Drawer$1 } from 'vaul';
import * as DropdownMenuPrimitive from '@radix-ui/react-dropdown-menu';
import * as _radix_ui_react_slot from '@radix-ui/react-slot';
import * as react_hook_form from 'react-hook-form';
import { FieldValues, FieldPath, ControllerProps, UseFormReturn, DefaultValues } from 'react-hook-form';
import * as LabelPrimitive from '@radix-ui/react-label';
import * as PopoverPrimitive from '@radix-ui/react-popover';
import * as SelectPrimitive from '@radix-ui/react-select';
import * as SeparatorPrimitive from '@radix-ui/react-separator';
import * as TooltipPrimitive from '@radix-ui/react-tooltip';
import { Toaster as Toaster$1 } from 'sonner';
import * as TabsPrimitive from '@radix-ui/react-tabs';
export { JsonTable, JsonTableProps, JsonViewer, JsonViewerProps, MonacoJsonProps } from './json/index.js';
export { WALLET_KEYS_QUERY_KEY, WalletCredentialsView, WalletCredentialsViewProps, WalletDidModel, WalletDidView, WalletInfoData, WalletInfoResponse, WalletInfoView, WalletKeyItem, WalletKeyRef, WalletKeysView, WalletLayout, WalletLayoutProps, WalletOidc4vciView, WalletOidc4vpView, WalletVcRecord } from './wallet/index.js';

declare const eunomiaLogoLight: string;
declare const eunomiaLogoDark: string;
declare const dsAgentLogoLight: string;
declare const dsAgentLogoDark: string;
declare const defaultAvatarImg: string;

interface LinkProps {
    to: string;
    className?: string;
    children?: React__default.ReactNode;
    title?: string;
    role?: string;
    "aria-selected"?: boolean;
    onClick?: React__default.MouseEventHandler<HTMLAnchorElement>;
}
type LinkLike = React__default.ComponentType<LinkProps & React__default.RefAttributes<HTMLAnchorElement>>;
declare const LinkProvider: React__default.Provider<LinkLike>;
declare const ActivePathProvider: React__default.Provider<string>;
declare const useLink: () => LinkLike;
declare const useActivePath: () => string;
interface NavigationProviderProps {
    link: LinkLike;
    activePath?: string;
    children: React__default.ReactNode;
}
declare function NavigationProvider({ link, activePath, children }: NavigationProviderProps): react_jsx_runtime.JSX.Element;

/** User-selectable preference; "system" follows the OS setting. */
type ThemePreference = "light" | "dark" | "system";
/** Theme actually painted on screen once "system" is resolved. */
type ResolvedTheme = "light" | "dark";
interface ThemeContextType {
    theme: ThemePreference;
    resolvedTheme: ResolvedTheme;
    setTheme: (theme: ThemePreference) => void;
    /** Flips between light and dark, resolving "system" first. */
    toggleTheme: () => void;
}
declare const THEME_STORAGE_KEY = "eunomia.theme";
declare const ThemeContext: React.Context<ThemeContextType | null>;
interface ThemeContextProviderProps {
    children: ReactNode;
    defaultTheme?: ThemePreference;
}
declare const ThemeContextProvider: ({ children, defaultTheme, }: ThemeContextProviderProps) => react_jsx_runtime.JSX.Element;

/** Reads the theme context; throws when used outside ThemeContextProvider. */
declare function useTheme(): ThemeContextType;

declare function useIsMobile(): boolean;

/**
 * useTableQueryParams.ts
 *
 * Hook for synchronizing DataTable state (pagination, sorting, filtering, and search)
 * with browser URL search parameters and backend QuerySpec specifications.
 */
interface TableQueryParams {
    limit: number;
    page: number;
    cursor?: string;
    sort: string;
    search: string;
    filters: Record<string, string>;
}
interface UseTableQueryParamsOptions {
    defaultLimit?: number;
    defaultSort?: string;
    defaultFilters?: Record<string, string>;
    syncUrl?: boolean;
}
/**
 * Maps frontend column key and direction to backend Sort enum representation.
 */
declare function mapToSortParam(key: string, direction: "asc" | "desc"): string;
/**
 * Hook for bidirectional synchronization of table queries with URL search params.
 */
declare function useTableQueryParams(options?: UseTableQueryParamsOptions): {
    params: TableQueryParams;
    apiParams: Record<string, any>;
    onQueryChange: (updated: Partial<TableQueryParams>) => void;
    setPage: (page: number) => void;
    setLimit: (limit: number) => void;
    setSort: (sort: string) => void;
    setSearch: (search: string) => void;
    setFilter: (id: string, value: string) => void;
    resetAll: () => void;
};

declare function cn(...inputs: ClassValue[]): string;
declare const generateRandomString: (length: number) => string;
declare const mergeStateAndAttribute: (state: string, attribute: string) => string;
declare const formatUrn: (urn: string | undefined, truncate?: boolean) => string;
/**
 * Formats structured identifiers (e.g., URNs, DIDs, or similar colon-separated strings)
 * into a shorter, human-readable form.
 *
 * The function extracts the most specific segment of the identifier (typically the last
 * portion after the final ":"), optionally truncating it to a configurable maximum length.
 *
 * This makes long technical identifiers easier to display in UI components while preserving
 * their uniqueness and recognizability.
 *
 * Examples:
 * - "urn:dataset:abcd123456789" - "abcd1234"
 * - "did:web:example.com:user:xyz987654" - "xyz98765"
 *
 * If the input does not contain a structured prefix, a fallback truncation strategy may be applied.
 *
 * @param value - The identifier string to format.
 * @param truncate - Whether to truncate the extracted segment (default: true).
 * @param maxLength - Maximum length of the returned string when truncating (default: 8).
 * @returns A shortened, display-friendly version of the identifier.
 */
declare const formatIdentifier: (value: string | undefined, nParts?: number, truncate?: boolean, maxLength?: number) => string;
/**
 * Utility function to convert technical VC type names into friendly readable names.
 * Rules:
 * - gx_ prefix -> Gaia-X
 * - CamelCase/PascalCase -> Split with spaces
 * - Add "Credential" if not present (unless it's a Participant type)
 * - jwt -> (JWT) suffix
 *
 * @example
 * gx_VatId_jwt_vc_json -> Gaia-X Vat Id Credential (JWT)
 */
declare const getFriendlyVCType: (type: string) => string;
/**
 * A path segment for ids that hold `:` or `/` (DIDs such as `did:web:host%3A3000`), as the auth
 * agent expects them in `/mates/{id}` and `/peer-connection/token/{id}`: base64url (RFC 4648 §5)
 * without padding, which survives any number of URL decodings on the way.
 */
declare const encodePathId: (id: string) => string;
declare const decodePathId: (encoded: string) => string;
declare const pathIdParams: <P extends string>(key: P) => {
    parse: (raw: Record<P, string>) => Record<P, string>;
    stringify: (params: Record<P, string>) => Record<P, string>;
};
declare const grantBadgeState: (status: string | undefined) => string | undefined;

/** Event fired when a call finds no session (401). */
declare const UNAUTHORIZED_EVENT = "eunomia:unauthorized";
declare const setApiGatewayBase: (url: string) => void;
declare const getApiGatewayBase: () => string;
type RequestConfig = RequestInit;
interface CustomInstanceOptions extends Partial<RequestConfig> {
    method?: string;
    headers?: HeadersInit;
    params?: Record<string, unknown>;
    data?: unknown;
    /** A 401 here does not end the session (used to ask whether there is one). */
    _noSessionRedirect?: boolean;
}
declare const customInstance: <T>(url: string, options?: CustomInstanceOptions) => Promise<T>;
type ErrorType<Error> = Error;
type BodyType<Body> = Body;

/** The session's user, as `GET /me` returns it. */
interface SessionUser {
    userId: string;
    username?: string | null;
    email?: string | null;
    /** Role path, e.g. `/admin/upm`. */
    role: string;
    /** Whether it is the root (`/admin`), who sees and acts on everything. */
    root: boolean;
}
/** The user of the session, once `loadSession` found one. */
declare const getSessionUser: () => SessionUser | null;
/** A name to show for the session's user. */
declare const sessionDisplayName: (user: SessionUser | null) => string;
/** Asks the gateway who the session is; `null` when there is none (401). */
declare const loadSession: () => Promise<SessionUser | null>;
/** Sends the browser to the proxy's sign-in, coming back to where it was. */
declare const signIn: () => void;
/** Ends the proxy's session and comes back to the console. */
declare const signOut: (redirectPath?: string) => void;

/**
 * A vertically stacked set of interactive headings that each reveal a section of content.
 */
declare const Accordion: React.ForwardRefExoticComponent<(AccordionPrimitive.AccordionSingleProps | AccordionPrimitive.AccordionMultipleProps) & React.RefAttributes<HTMLDivElement>>;
declare const AccordionItem: React.ForwardRefExoticComponent<Omit<AccordionPrimitive.AccordionItemProps & React.RefAttributes<HTMLDivElement>, "ref"> & React.RefAttributes<HTMLDivElement>>;
declare const AccordionTrigger: React.ForwardRefExoticComponent<Omit<AccordionPrimitive.AccordionTriggerProps & React.RefAttributes<HTMLButtonElement>, "ref"> & React.RefAttributes<HTMLButtonElement>>;
declare const AccordionContent: React.ForwardRefExoticComponent<Omit<AccordionPrimitive.AccordionContentProps & React.RefAttributes<HTMLDivElement>, "ref"> & React.RefAttributes<HTMLDivElement>>;

type AvatarProps = {
    src?: string;
    alt?: string;
    sizeClass?: string;
    wrapperClass?: string;
};
declare function Avatar({ src, alt, sizeClass, wrapperClass, }: AvatarProps): react_jsx_runtime.JSX.Element;

/**
 * Badge style variants using class-variance-authority.
 * Consolidated palette supporting states, roles, and enterprise semantics.
 */
declare const badgeVariants: (props?: ({
    variant?: "role" | "default" | "status" | "code" | "secondary" | "outline" | "destructive" | "danger" | "success" | "warning" | "warn" | "info" | "infoLighter" | "detail" | "wizard" | "wizardSuccess" | null | undefined;
    state?: "default" | "danger" | "success" | "warn" | "process" | "pause" | null | undefined;
    dsrole?: "Provider" | "Consumer" | "Business" | "Customer" | null | undefined;
    size?: "default" | "xs" | "sm" | "lg" | null | undefined;
} & class_variance_authority_types.ClassProp) | undefined) => string;
/** Available badge states for status variant */
type BadgeState = VariantProps<typeof badgeVariants>["state"];
/** Available role types for role variant */
type BadgeRole = VariantProps<typeof badgeVariants>["dsrole"];
/**
 * Props for the Badge component.
 * Extends span attributes with variant options.
 */
interface BadgeProps extends React.ComponentProps<"span">, Omit<VariantProps<typeof badgeVariants>, "state"> {
    /** Accept API status directly */
    state?: string;
    /** Explicit dot indicator */
    dot?: boolean;
    asChild?: boolean;
}
/**
 * Consolidated Badge component for statuses, roles, tags, and micro-labels.
 */
declare function Badge({ className, variant, state, size, dsrole, dot, asChild, children, ...props }: BadgeProps): react_jsx_runtime.JSX.Element;

interface BooleanBadgeProps {
    value: boolean | string | null | undefined;
    trueLabel?: string;
    falseLabel?: string;
}
declare function BooleanBadge({ value, trueLabel, falseLabel, }: BooleanBadgeProps): react_jsx_runtime.JSX.Element;

/**
 * Displays the path to the current resource using a hierarchy of links.
 */
declare const Breadcrumb: React.ForwardRefExoticComponent<Omit<React.DetailedHTMLProps<React.HTMLAttributes<HTMLElement>, HTMLElement>, "ref"> & {
    separator?: React.ReactNode;
} & React.RefAttributes<HTMLElement>>;
declare const BreadcrumbList: React.ForwardRefExoticComponent<Omit<React.DetailedHTMLProps<React.OlHTMLAttributes<HTMLOListElement>, HTMLOListElement>, "ref"> & React.RefAttributes<HTMLOListElement>>;
declare const BreadcrumbItem: React.ForwardRefExoticComponent<Omit<React.DetailedHTMLProps<React.LiHTMLAttributes<HTMLLIElement>, HTMLLIElement>, "ref"> & React.RefAttributes<HTMLLIElement>>;
declare const BreadcrumbLink: React.ForwardRefExoticComponent<Omit<React.DetailedHTMLProps<React.AnchorHTMLAttributes<HTMLAnchorElement>, HTMLAnchorElement>, "ref"> & {
    asChild?: boolean;
} & React.RefAttributes<HTMLAnchorElement>>;
declare const BreadcrumbPage: React.ForwardRefExoticComponent<Omit<React.DetailedHTMLProps<React.HTMLAttributes<HTMLSpanElement>, HTMLSpanElement>, "ref"> & React.RefAttributes<HTMLSpanElement>>;
declare const BreadcrumbSeparator: {
    ({ children, className, ...props }: React.ComponentProps<"li">): react_jsx_runtime.JSX.Element;
    displayName: string;
};
declare const BreadcrumbEllipsis: {
    ({ className, ...props }: React.ComponentProps<"span">): react_jsx_runtime.JSX.Element;
    displayName: string;
};

type BubbleColors = {
    first: string;
    second: string;
    third: string;
    fourth: string;
    fifth: string;
    sixth: string;
};
type BubbleBackgroundProps = React.ComponentProps<"div"> & {
    interactive?: boolean;
    transition?: SpringOptions;
    colors?: BubbleColors;
};
declare const BubbleBackground: React.ForwardRefExoticComponent<Omit<BubbleBackgroundProps, "ref"> & React.RefAttributes<HTMLDivElement>>;

declare const buttonVariants: (props?: ({
    variant?: "link" | "default" | "secondary" | "outline" | "destructive" | "success" | "outline_destructive" | "ghost" | "icon_destructive" | null | undefined;
    size?: "default" | "xs" | "sm" | "lg" | "icon" | "icon_sm" | null | undefined;
    policy?: "default" | "permission" | "obligation" | "prohibition" | null | undefined;
} & class_variance_authority_types.ClassProp) | undefined) => string;
type ButtonSizes = VariantProps<typeof buttonVariants>["size"];
interface ButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement>, VariantProps<typeof buttonVariants> {
    asChild?: boolean;
    isLoading?: boolean;
}
/**
 * Displays a button or a component that looks like a button.
 */
declare const Button: React.ForwardRefExoticComponent<ButtonProps & React.RefAttributes<HTMLButtonElement>>;

/**
 * Visual variants for Card container component.
 */
declare const cardVariants: (props?: ({
    variant?: "default" | "outline" | "interactive" | "ghost" | "glass" | "metric" | "subtle" | null | undefined;
} & class_variance_authority_types.ClassProp) | undefined) => string;
interface CardProps extends React.HTMLAttributes<HTMLDivElement>, VariantProps<typeof cardVariants> {
}
/**
 * Consolidated enterprise Card container component.
 */
declare const Card: React.ForwardRefExoticComponent<CardProps & React.RefAttributes<HTMLDivElement>>;
declare const CardHeader: React.ForwardRefExoticComponent<React.HTMLAttributes<HTMLDivElement> & React.RefAttributes<HTMLDivElement>>;
declare const CardTitle: React.ForwardRefExoticComponent<React.HTMLAttributes<HTMLHeadingElement> & React.RefAttributes<HTMLHeadingElement>>;
declare const CardDescription: React.ForwardRefExoticComponent<React.HTMLAttributes<HTMLParagraphElement> & React.RefAttributes<HTMLParagraphElement>>;
declare const CardContent: React.ForwardRefExoticComponent<React.HTMLAttributes<HTMLDivElement> & React.RefAttributes<HTMLDivElement>>;
declare const CardFooter: React.ForwardRefExoticComponent<React.HTMLAttributes<HTMLDivElement> & React.RefAttributes<HTMLDivElement>>;

declare const THEMES: {
    readonly light: "";
    readonly dark: ".dark";
};
type ChartConfig = {
    [k in string]: {
        label?: React.ReactNode;
        icon?: React.ComponentType;
    } & ({
        color?: string;
        theme?: never;
    } | {
        color?: never;
        theme: Record<keyof typeof THEMES, string>;
    });
};
/**
 * Container for the chart content.
 */
declare const ChartContainer: React.ForwardRefExoticComponent<Omit<React.ClassAttributes<HTMLDivElement> & React.HTMLAttributes<HTMLDivElement> & {
    config: ChartConfig;
    children: React.ComponentProps<typeof RechartsPrimitive.ResponsiveContainer>["children"];
}, "ref"> & React.RefAttributes<HTMLDivElement>>;
declare const ChartStyle: ({ id, config }: {
    id: string;
    config: ChartConfig;
}) => react_jsx_runtime.JSX.Element | null;
declare const ChartTooltip: typeof RechartsPrimitive.Tooltip;
type TooltipPayloadItem = {
    dataKey?: string | number;
    name?: string;
    value?: number;
    color?: string;
    payload?: Record<string, unknown>;
    [key: string]: unknown;
};
declare const ChartTooltipContent: React.ForwardRefExoticComponent<Omit<React.ClassAttributes<HTMLDivElement> & React.HTMLAttributes<HTMLDivElement> & {
    active?: boolean;
    payload?: TooltipPayloadItem[];
    label?: string;
    hideLabel?: boolean;
    hideIndicator?: boolean;
    indicator?: "line" | "dot" | "dashed";
    nameKey?: string;
    labelKey?: string;
    labelFormatter?: (value: unknown, payload: TooltipPayloadItem[]) => React.ReactNode;
    labelClassName?: string;
    formatter?: (value: unknown, name: string, item: TooltipPayloadItem, index: number, payload: unknown) => React.ReactNode;
    color?: string;
}, "ref"> & React.RefAttributes<HTMLDivElement>>;
declare const ChartLegend: typeof RechartsPrimitive.Legend;
type LegendPayloadItem = {
    dataKey?: string | number;
    value?: string;
    color?: string;
    [key: string]: unknown;
};
declare const ChartLegendContent: React.ForwardRefExoticComponent<Omit<React.ClassAttributes<HTMLDivElement> & React.HTMLAttributes<HTMLDivElement> & {
    payload?: LegendPayloadItem[];
    verticalAlign?: "top" | "bottom" | "middle";
    hideIcon?: boolean;
    nameKey?: string;
}, "ref"> & React.RefAttributes<HTMLDivElement>>;

/**
 * Enterprise shadcn Checkbox component.
 */
declare const Checkbox: React.ForwardRefExoticComponent<Omit<CheckboxPrimitive.CheckboxProps & React.RefAttributes<HTMLButtonElement>, "ref"> & React.RefAttributes<HTMLButtonElement>>;

/**
 * Command menu component for searching and executing commands.
 */
declare const Command: React.ForwardRefExoticComponent<Omit<{
    children?: React.ReactNode;
} & Pick<Pick<React.DetailedHTMLProps<React.HTMLAttributes<HTMLDivElement>, HTMLDivElement>, "key" | keyof React.HTMLAttributes<HTMLDivElement>> & {
    ref?: React.Ref<HTMLDivElement>;
} & {
    asChild?: boolean;
}, "key" | keyof React.HTMLAttributes<HTMLDivElement> | "asChild"> & {
    label?: string;
    shouldFilter?: boolean;
    filter?: (value: string, search: string, keywords?: string[]) => number;
    defaultValue?: string;
    value?: string;
    onValueChange?: (value: string) => void;
    loop?: boolean;
    disablePointerSelection?: boolean;
    vimBindings?: boolean;
} & React.RefAttributes<HTMLDivElement>, "ref"> & React.RefAttributes<HTMLDivElement>>;
declare const CommandDialog: ({ children, ...props }: DialogProps) => react_jsx_runtime.JSX.Element;
declare const CommandInput: React.ForwardRefExoticComponent<Omit<Omit<Pick<Pick<React.DetailedHTMLProps<React.InputHTMLAttributes<HTMLInputElement>, HTMLInputElement>, "key" | keyof React.InputHTMLAttributes<HTMLInputElement>> & {
    ref?: React.Ref<HTMLInputElement>;
} & {
    asChild?: boolean;
}, "key" | "asChild" | keyof React.InputHTMLAttributes<HTMLInputElement>>, "value" | "onChange" | "type"> & {
    value?: string;
    onValueChange?: (search: string) => void;
} & React.RefAttributes<HTMLInputElement>, "ref"> & React.RefAttributes<HTMLInputElement>>;
declare const CommandList: React.ForwardRefExoticComponent<Omit<{
    children?: React.ReactNode;
} & Pick<Pick<React.DetailedHTMLProps<React.HTMLAttributes<HTMLDivElement>, HTMLDivElement>, "key" | keyof React.HTMLAttributes<HTMLDivElement>> & {
    ref?: React.Ref<HTMLDivElement>;
} & {
    asChild?: boolean;
}, "key" | keyof React.HTMLAttributes<HTMLDivElement> | "asChild"> & {
    label?: string;
} & React.RefAttributes<HTMLDivElement>, "ref"> & React.RefAttributes<HTMLDivElement>>;
declare const CommandEmpty: React.ForwardRefExoticComponent<Omit<{
    children?: React.ReactNode;
} & Pick<Pick<React.DetailedHTMLProps<React.HTMLAttributes<HTMLDivElement>, HTMLDivElement>, "key" | keyof React.HTMLAttributes<HTMLDivElement>> & {
    ref?: React.Ref<HTMLDivElement>;
} & {
    asChild?: boolean;
}, "key" | keyof React.HTMLAttributes<HTMLDivElement> | "asChild"> & React.RefAttributes<HTMLDivElement>, "ref"> & React.RefAttributes<HTMLDivElement>>;
declare const CommandGroup: React.ForwardRefExoticComponent<Omit<{
    children?: React.ReactNode;
} & Omit<Pick<Pick<React.DetailedHTMLProps<React.HTMLAttributes<HTMLDivElement>, HTMLDivElement>, "key" | keyof React.HTMLAttributes<HTMLDivElement>> & {
    ref?: React.Ref<HTMLDivElement>;
} & {
    asChild?: boolean;
}, "key" | keyof React.HTMLAttributes<HTMLDivElement> | "asChild">, "heading" | "value"> & {
    heading?: React.ReactNode;
    value?: string;
    forceMount?: boolean;
} & React.RefAttributes<HTMLDivElement>, "ref"> & React.RefAttributes<HTMLDivElement>>;
declare const CommandSeparator: React.ForwardRefExoticComponent<Omit<Pick<Pick<React.DetailedHTMLProps<React.HTMLAttributes<HTMLDivElement>, HTMLDivElement>, "key" | keyof React.HTMLAttributes<HTMLDivElement>> & {
    ref?: React.Ref<HTMLDivElement>;
} & {
    asChild?: boolean;
}, "key" | keyof React.HTMLAttributes<HTMLDivElement> | "asChild"> & {
    alwaysRender?: boolean;
} & React.RefAttributes<HTMLDivElement>, "ref"> & React.RefAttributes<HTMLDivElement>>;
declare const CommandItem: React.ForwardRefExoticComponent<Omit<{
    children?: React.ReactNode;
} & Omit<Pick<Pick<React.DetailedHTMLProps<React.HTMLAttributes<HTMLDivElement>, HTMLDivElement>, "key" | keyof React.HTMLAttributes<HTMLDivElement>> & {
    ref?: React.Ref<HTMLDivElement>;
} & {
    asChild?: boolean;
}, "key" | keyof React.HTMLAttributes<HTMLDivElement> | "asChild">, "value" | "onSelect" | "disabled"> & {
    disabled?: boolean;
    onSelect?: (value: string) => void;
    value?: string;
    keywords?: string[];
    forceMount?: boolean;
} & React.RefAttributes<HTMLDivElement>, "ref"> & React.RefAttributes<HTMLDivElement>>;
declare const CommandShortcut: {
    ({ className, ...props }: React.HTMLAttributes<HTMLSpanElement>): react_jsx_runtime.JSX.Element;
    displayName: string;
};

interface ConfirmDialogProps {
    open: boolean;
    onOpenChange: (open: boolean) => void;
    title: React__default.ReactNode;
    description?: React__default.ReactNode;
    /** What else happens, shown under the description. */
    children?: React__default.ReactNode;
    confirmLabel?: string;
    destructive?: boolean;
    /** Runs on confirm; when it throws, the dialog stays open with the error. */
    onConfirm: () => Promise<void>;
}
/** "Are you sure?" before an action that cannot be undone. */
declare function ConfirmDialog({ open, onOpenChange, title, description, children, confirmLabel, destructive, onConfirm, }: ConfirmDialogProps): react_jsx_runtime.JSX.Element;

/** A small count next to a tab or title; renders nothing for zero unless `showZero`. */
declare function CountBadge({ n, showZero, className, }: {
    n?: number;
    showZero?: boolean;
    className?: string;
}): react_jsx_runtime.JSX.Element | null;

/**
 * Modal dialog component.
 */
declare const Dialog: React.FC<DialogPrimitive.DialogProps>;
declare const DialogTrigger: React.ForwardRefExoticComponent<DialogPrimitive.DialogTriggerProps & React.RefAttributes<HTMLButtonElement>>;
declare const DialogPortal: React.FC<DialogPrimitive.DialogPortalProps>;
declare const DialogClose: React.ForwardRefExoticComponent<DialogPrimitive.DialogCloseProps & React.RefAttributes<HTMLButtonElement>>;
declare const DialogOverlay: React.ForwardRefExoticComponent<Omit<DialogPrimitive.DialogOverlayProps & React.RefAttributes<HTMLDivElement>, "ref"> & React.RefAttributes<HTMLDivElement>>;
declare const DialogContent: React.ForwardRefExoticComponent<Omit<DialogPrimitive.DialogContentProps & React.RefAttributes<HTMLDivElement>, "ref"> & React.RefAttributes<HTMLDivElement>>;
declare const DialogHeader: {
    ({ className, ...props }: React.HTMLAttributes<HTMLDivElement>): react_jsx_runtime.JSX.Element;
    displayName: string;
};
declare const DialogBody: {
    ({ className, ...props }: React.HTMLAttributes<HTMLDivElement>): react_jsx_runtime.JSX.Element;
    displayName: string;
};
declare const DialogFooter: {
    ({ className, ...props }: React.HTMLAttributes<HTMLDivElement>): react_jsx_runtime.JSX.Element;
    displayName: string;
};
declare const DialogTitle: React.ForwardRefExoticComponent<Omit<DialogPrimitive.DialogTitleProps & React.RefAttributes<HTMLHeadingElement>, "ref"> & React.RefAttributes<HTMLHeadingElement>>;
declare const DialogDescription: React.ForwardRefExoticComponent<Omit<DialogPrimitive.DialogDescriptionProps & React.RefAttributes<HTMLParagraphElement>, "ref"> & React.RefAttributes<HTMLParagraphElement>>;

/**
 * Drawer component that slides in from the edge of the screen.
 */
declare function Drawer({ ...props }: React.ComponentProps<typeof Drawer$1.Root>): react_jsx_runtime.JSX.Element;
declare function DrawerTrigger({ ...props }: React.ComponentProps<typeof Drawer$1.Trigger>): react_jsx_runtime.JSX.Element;
declare function DrawerPortal({ ...props }: React.ComponentProps<typeof Drawer$1.Portal>): react_jsx_runtime.JSX.Element;
declare function DrawerClose({ ...props }: React.ComponentProps<typeof Drawer$1.Close>): react_jsx_runtime.JSX.Element;
declare function DrawerOverlay({ className, ...props }: React.ComponentProps<typeof Drawer$1.Overlay>): react_jsx_runtime.JSX.Element;
declare function DrawerContent({ className, children, ...props }: React.ComponentProps<typeof Drawer$1.Content>): react_jsx_runtime.JSX.Element;
declare function DrawerHeader({ className, ...props }: React.ComponentProps<"div">): react_jsx_runtime.JSX.Element;
declare function DrawerBody({ className, ...props }: React.ComponentProps<"div">): react_jsx_runtime.JSX.Element;
declare function DrawerFooter({ className, ...props }: React.ComponentProps<"div">): react_jsx_runtime.JSX.Element;
declare function DrawerTitle({ className, ...props }: React.ComponentProps<typeof Drawer$1.Title>): react_jsx_runtime.JSX.Element;
declare function DrawerDescription({ className, ...props }: React.ComponentProps<typeof Drawer$1.Description>): react_jsx_runtime.JSX.Element;

declare const DropdownMenu: React.FC<DropdownMenuPrimitive.DropdownMenuProps>;
declare const DropdownMenuTrigger: React.ForwardRefExoticComponent<DropdownMenuPrimitive.DropdownMenuTriggerProps & React.RefAttributes<HTMLButtonElement>>;
declare const DropdownMenuGroup: React.ForwardRefExoticComponent<DropdownMenuPrimitive.DropdownMenuGroupProps & React.RefAttributes<HTMLDivElement>>;
declare const DropdownMenuPortal: React.FC<DropdownMenuPrimitive.DropdownMenuPortalProps>;
declare const DropdownMenuSub: React.FC<DropdownMenuPrimitive.DropdownMenuSubProps>;
declare const DropdownMenuRadioGroup: React.ForwardRefExoticComponent<DropdownMenuPrimitive.DropdownMenuRadioGroupProps & React.RefAttributes<HTMLDivElement>>;
declare const DropdownMenuSubTrigger: React.ForwardRefExoticComponent<Omit<DropdownMenuPrimitive.DropdownMenuSubTriggerProps & React.RefAttributes<HTMLDivElement>, "ref"> & {
    inset?: boolean;
} & React.RefAttributes<HTMLDivElement>>;
declare const DropdownMenuSubContent: React.ForwardRefExoticComponent<Omit<DropdownMenuPrimitive.DropdownMenuSubContentProps & React.RefAttributes<HTMLDivElement>, "ref"> & React.RefAttributes<HTMLDivElement>>;
declare const DropdownMenuContent: React.ForwardRefExoticComponent<Omit<DropdownMenuPrimitive.DropdownMenuContentProps & React.RefAttributes<HTMLDivElement>, "ref"> & React.RefAttributes<HTMLDivElement>>;
declare const DropdownMenuItem: React.ForwardRefExoticComponent<Omit<DropdownMenuPrimitive.DropdownMenuItemProps & React.RefAttributes<HTMLDivElement>, "ref"> & {
    inset?: boolean;
} & React.RefAttributes<HTMLDivElement>>;
declare const DropdownMenuCheckboxItem: React.ForwardRefExoticComponent<Omit<DropdownMenuPrimitive.DropdownMenuCheckboxItemProps & React.RefAttributes<HTMLDivElement>, "ref"> & React.RefAttributes<HTMLDivElement>>;
declare const DropdownMenuRadioItem: React.ForwardRefExoticComponent<Omit<DropdownMenuPrimitive.DropdownMenuRadioItemProps & React.RefAttributes<HTMLDivElement>, "ref"> & React.RefAttributes<HTMLDivElement>>;
declare const DropdownMenuLabel: React.ForwardRefExoticComponent<Omit<DropdownMenuPrimitive.DropdownMenuLabelProps & React.RefAttributes<HTMLDivElement>, "ref"> & {
    inset?: boolean;
} & React.RefAttributes<HTMLDivElement>>;
declare const DropdownMenuSeparator: React.ForwardRefExoticComponent<Omit<DropdownMenuPrimitive.DropdownMenuSeparatorProps & React.RefAttributes<HTMLDivElement>, "ref"> & React.RefAttributes<HTMLDivElement>>;
declare const DropdownMenuShortcut: {
    ({ className, ...props }: React.HTMLAttributes<HTMLSpanElement>): react_jsx_runtime.JSX.Element;
    displayName: string;
};

/**
 * Wrapper for form components using react-hook-form.
 */
declare const Form: <TFieldValues extends FieldValues, TContext = any, TTransformedValues = TFieldValues>(props: react_hook_form.FormProviderProps<TFieldValues, TContext, TTransformedValues>) => React.JSX.Element;
declare const FormField: <TFieldValues extends FieldValues = FieldValues, TName extends FieldPath<TFieldValues> = FieldPath<TFieldValues>>({ ...props }: ControllerProps<TFieldValues, TName>) => react_jsx_runtime.JSX.Element;
declare const useFormField: () => {
    invalid: boolean;
    isDirty: boolean;
    isTouched: boolean;
    isValidating: boolean;
    error?: react_hook_form.FieldError;
    id: string;
    name: string;
    formItemId: string;
    formDescriptionId: string;
    formMessageId: string;
};
declare const FormItem: React.ForwardRefExoticComponent<React.HTMLAttributes<HTMLDivElement> & React.RefAttributes<HTMLDivElement>>;
declare const FormLabel: React.ForwardRefExoticComponent<Omit<LabelPrimitive.LabelProps & React.RefAttributes<HTMLLabelElement>, "ref"> & React.RefAttributes<HTMLLabelElement>>;
declare const FormControl: React.ForwardRefExoticComponent<Omit<_radix_ui_react_slot.SlotProps & React.RefAttributes<HTMLElement>, "ref"> & React.RefAttributes<HTMLElement>>;
declare const FormDescription: React.ForwardRefExoticComponent<React.HTMLAttributes<HTMLParagraphElement> & React.RefAttributes<HTMLParagraphElement>>;
declare const FormMessage: React.ForwardRefExoticComponent<React.HTMLAttributes<HTMLParagraphElement> & React.RefAttributes<HTMLParagraphElement>>;

/** The one date-time format of the GUI: sortable and free of day/month ambiguity. */
declare const DATE_TIME_FORMAT = "YYYY-MM-DD HH:mm";
declare const DATE_FORMAT = "YYYY-MM-DD";
interface FormatDateProps {
    date: string | Date | undefined | null;
    format?: string;
}
declare const FormatDate: ({ date, format }: FormatDateProps) => react_jsx_runtime.JSX.Element;
/** "3 minutes ago", with the exact time on hover. */
declare const RelativeTime: ({ date, className }: {
    date: string | Date;
    className?: string;
}) => react_jsx_runtime.JSX.Element;

declare const formatKey: (text: any) => string;

declare const formatOperator: (text: any) => string;

declare const formatValue: (text: any) => string;

/**
 * heading.tsx
 *
 * Universal heading component with consistent styling across all heading levels.
 * Automatically applies appropriate typography, spacing, and visual hierarchy.
 *
 * Supports semantic heading levels (h1-h6) plus custom variants for
 * specific contexts like tables, small titles, and subtitles.
 *
 * @example
 * // Page title
 * <Heading level="h1">Dashboard</Heading>
 *
 * @example
 * // Section heading
 * <Heading level="h4">Recent Activities</Heading>
 *
 * @example
 * // Table header
 * <Heading level="table">Status</Heading>
 */

/**
 * Available heading levels and variants.
 *
 * Standard levels:
 * - `h1`: Page titles (largest)
 * - `h2`: Major sections
 * - `h3`: Subsections
 * - `h4`: Card/panel titles
 * - `h5`: Small section headers
 * - `h6`: Smallest, uppercase labels
 *
 * Custom variants:
 * - `table`: Table column headers
 * - `title-sm`: Small titles for compact spaces
 * - `subtitle`: Descriptive subtitle text
 */
type HeadingLevel = "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "table" | "title-sm" | "subtitle";
/**
 * Props for the Heading component.
 */
interface HeadingProps {
    /** Heading level or variant determining size and styling */
    level: HeadingLevel;
    /** Content to display */
    children: React__default.ReactNode;
    /** Additional CSS classes */
    className?: string;
    /** Ref for the heading element */
    ref?: React__default.Ref<HTMLHeadingElement>;
}
/**
 * Semantic heading component with consistent typography.
 *
 * Provides a unified interface for all heading needs across the application,
 * ensuring visual consistency while maintaining semantic HTML structure.
 *
 * @param props - Heading properties
 * @returns A styled heading element
 */
declare const Heading: FC<HeadingProps>;

/**
 * Supported value types for info list items.
 *
 * - `string`: Plain text displayed in an info badge
 * - `date`: Date value formatted as "DD/MM/YY HH:mm"
 * - `status`: Process state displayed with colored status badge
 * - `role`: User role displayed with role-colored badge
 * - `urn`: URN/UUID displayed truncated in an info badge
 * - `custom`: Custom React content rendered directly
 */
type InfoItemValue = string | undefined | null | {
    type: "date";
    value: string | Date;
} | {
    type: "status";
    value: string;
} | {
    type: "role";
    value: string;
} | {
    type: "urn";
    value: string | undefined;
} | {
    type: "text";
    value: string | undefined;
} | {
    type: "custom";
    content: React__default.ReactNode;
};
/**
 * Props for a single info list item.
 */
interface InfoItemProps {
    /** Label displayed above the value */
    label: string;
    /** Value to display (see InfoItemValue for supported types) */
    value: InfoItemValue;
    /** Additional class for the item container */
    className?: string;
    /** Additional class for the label element */
    keyClassName?: string;
}
/**
 * Props for the InfoList component.
 */
interface InfoListProps {
    /** Array of info items to display */
    items: InfoItemProps[];
    /** Additional class for the list container */
    className?: string;
}
/**
 * Displays a vertical list of labeled information items.
 *
 * Each item shows a small label above a formatted value.
 * Items are separated by subtle borders.
 *
 * @param props - InfoList properties
 * @returns A styled list of info items
 */
declare const InfoList: ({ items, className }: InfoListProps) => react_jsx_runtime.JSX.Element;
/**
 * Single info list item component.
 * Renders a label and automatically formats the value based on its type.
 */
declare const InfoListItem: ({ label, value, className, keyClassName }: InfoItemProps) => react_jsx_runtime.JSX.Element | null;

interface InputProps extends React.ComponentProps<"input"> {
    startIcon?: React.ReactNode;
    endIcon?: React.ReactNode;
    containerClassName?: string;
    onClear?: () => void;
}
/**
 * Enterprise shadcn Input component with dark styling, icons, and clear button support.
 */
declare const Input: React.ForwardRefExoticComponent<Omit<InputProps, "ref"> & React.RefAttributes<HTMLInputElement>>;

/**
 * Label component for form inputs.
 */
declare const Label: React.ForwardRefExoticComponent<Omit<LabelPrimitive.LabelProps & React.RefAttributes<HTMLLabelElement>, "ref"> & VariantProps<(props?: class_variance_authority_types.ClassProp | undefined) => string> & React.RefAttributes<HTMLLabelElement>>;

/**
 * Wrapper for a list of items.
 */
declare const List: ({ className, children, ...props }: {
    className?: string;
    children: React.ReactNode;
    props?: any;
}) => react_jsx_runtime.JSX.Element;
declare const ListItem: ({ className, children, ...props }: {
    className?: string;
    children: React.ReactNode;
    props?: any;
}) => react_jsx_runtime.JSX.Element;
declare const ListItemKey: ({ className, children, ...props }: {
    className?: string;
    children: React.ReactNode;
    props?: any;
}) => react_jsx_runtime.JSX.Element;
declare const ListItemDate: ({ className, children, ...props }: {
    className?: string;
    children: React.ReactNode;
    props?: any;
}) => react_jsx_runtime.JSX.Element;

type RoleType = "Provider" | "Consumer" | "Business" | "Customer";
declare const mapRoleToVariant: (role: RoleType) => "default" | "Provider" | "Consumer";
declare const MessageLog: React.ForwardRefExoticComponent<React.HTMLAttributes<HTMLDivElement> & {
    variant: RoleType;
} & React.RefAttributes<HTMLDivElement>>;

declare const roleVariants: (props?: ({
    variant?: "default" | "Provider" | "Consumer" | null | undefined;
} & class_variance_authority_types.ClassProp) | undefined) => string;
declare const RoleHeader: React.ForwardRefExoticComponent<React.HTMLAttributes<HTMLDivElement> & {
    from: RoleType;
} & React.RefAttributes<HTMLDivElement>>;
declare const MessageBody: React.ForwardRefExoticComponent<React.HTMLAttributes<HTMLDivElement> & {
    variant: RoleType;
} & React.RefAttributes<HTMLDivElement>>;
declare const MessageTitle: React.ForwardRefExoticComponent<React.HTMLAttributes<HTMLHeadingElement> & React.RefAttributes<HTMLHeadingElement>>;
declare const MessageTimestamp: React.ForwardRefExoticComponent<{
    created_at: string;
} & React.HTMLAttributes<HTMLParagraphElement> & React.RefAttributes<HTMLParagraphElement>>;

declare const MessageContent: React.ForwardRefExoticComponent<React.HTMLAttributes<HTMLDivElement> & React.RefAttributes<HTMLDivElement>>;

declare const MessageMetaContainer: React.ForwardRefExoticComponent<React.HTMLAttributes<HTMLUListElement> & React.RefAttributes<HTMLUListElement>>;
interface MessageMetaProps extends React.HTMLAttributes<HTMLLIElement> {
    label: string;
    value: string;
}
declare const MessageMeta: React.ForwardRefExoticComponent<MessageMetaProps & React.RefAttributes<HTMLLIElement>>;

/**
 * Component to display a message indicating no further actions are available.
 */
declare const NoFurtherActions: () => react_jsx_runtime.JSX.Element;

/**
 * Popover component for displaying content in a floating layer.
 */
declare const Popover: React.FC<PopoverPrimitive.PopoverProps>;
declare const PopoverTrigger: React.ForwardRefExoticComponent<PopoverPrimitive.PopoverTriggerProps & React.RefAttributes<HTMLButtonElement>>;
declare const PopoverAnchor: React.ForwardRefExoticComponent<PopoverPrimitive.PopoverAnchorProps & React.RefAttributes<HTMLDivElement>>;
declare const PopoverContent: React.ForwardRefExoticComponent<Omit<PopoverPrimitive.PopoverContentProps & React.RefAttributes<HTMLDivElement>, "ref"> & React.RefAttributes<HTMLDivElement>>;

interface SegmentedOption<T extends string> {
    value: T;
    label: string;
    /** Tooltip explaining the option. */
    hint?: string;
}
/** A small set of mutually exclusive options shown side by side. */
declare function SegmentedControl<T extends string>({ value, onChange, options, className, ariaLabel, }: {
    value: T;
    onChange: (value: T) => void;
    options: SegmentedOption<T>[];
    className?: string;
    ariaLabel?: string;
}): react_jsx_runtime.JSX.Element;

/**
 * Select component for choosing a value from a list of options.
 */
declare const Select: React.FC<SelectPrimitive.SelectProps>;
declare const SelectGroup: React.ForwardRefExoticComponent<SelectPrimitive.SelectGroupProps & React.RefAttributes<HTMLDivElement>>;
declare const SelectValue: React.ForwardRefExoticComponent<SelectPrimitive.SelectValueProps & React.RefAttributes<HTMLSpanElement>>;
declare const SelectTrigger: React.ForwardRefExoticComponent<Omit<SelectPrimitive.SelectTriggerProps & React.RefAttributes<HTMLButtonElement>, "ref"> & React.RefAttributes<HTMLButtonElement>>;
declare const SelectScrollUpButton: React.ForwardRefExoticComponent<Omit<SelectPrimitive.SelectScrollUpButtonProps & React.RefAttributes<HTMLDivElement>, "ref"> & React.RefAttributes<HTMLDivElement>>;
declare const SelectScrollDownButton: React.ForwardRefExoticComponent<Omit<SelectPrimitive.SelectScrollDownButtonProps & React.RefAttributes<HTMLDivElement>, "ref"> & React.RefAttributes<HTMLDivElement>>;
declare const SelectContent: React.ForwardRefExoticComponent<Omit<SelectPrimitive.SelectContentProps & React.RefAttributes<HTMLDivElement>, "ref"> & React.RefAttributes<HTMLDivElement>>;
declare const SelectLabel: React.ForwardRefExoticComponent<Omit<SelectPrimitive.SelectLabelProps & React.RefAttributes<HTMLDivElement>, "ref"> & React.RefAttributes<HTMLDivElement>>;
declare const SelectItem: React.ForwardRefExoticComponent<Omit<SelectPrimitive.SelectItemProps & React.RefAttributes<HTMLDivElement>, "ref"> & React.RefAttributes<HTMLDivElement>>;
declare const SelectSeparator: React.ForwardRefExoticComponent<Omit<SelectPrimitive.SelectSeparatorProps & React.RefAttributes<HTMLDivElement>, "ref"> & React.RefAttributes<HTMLDivElement>>;

/**
 * Visual separator component.
 */
declare const Separator: React.ForwardRefExoticComponent<Omit<SeparatorPrimitive.SeparatorProps & React.RefAttributes<HTMLDivElement>, "ref"> & React.RefAttributes<HTMLDivElement>>;

/**
 * Sheet component that slides in from the side of the screen.
 */
declare const Sheet: React.FC<DialogPrimitive.DialogProps>;
declare const SheetTrigger: React.ForwardRefExoticComponent<DialogPrimitive.DialogTriggerProps & React.RefAttributes<HTMLButtonElement>>;
declare const SheetClose: React.ForwardRefExoticComponent<DialogPrimitive.DialogCloseProps & React.RefAttributes<HTMLButtonElement>>;
declare const SheetPortal: React.FC<DialogPrimitive.DialogPortalProps>;
declare const SheetOverlay: React.ForwardRefExoticComponent<Omit<DialogPrimitive.DialogOverlayProps & React.RefAttributes<HTMLDivElement>, "ref"> & React.RefAttributes<HTMLDivElement>>;
declare const sheetVariants: (props?: ({
    side?: "bottom" | "left" | "right" | "top" | null | undefined;
} & class_variance_authority_types.ClassProp) | undefined) => string;
interface SheetContentProps extends React.ComponentPropsWithoutRef<typeof DialogPrimitive.Content>, VariantProps<typeof sheetVariants> {
}
declare const SheetContent: React.ForwardRefExoticComponent<SheetContentProps & React.RefAttributes<HTMLDivElement>>;
declare const SheetHeader: {
    ({ className, ...props }: React.HTMLAttributes<HTMLDivElement>): react_jsx_runtime.JSX.Element;
    displayName: string;
};
declare const SheetFooter: {
    ({ className, ...props }: React.HTMLAttributes<HTMLDivElement>): react_jsx_runtime.JSX.Element;
    displayName: string;
};
declare const SheetTitle: React.ForwardRefExoticComponent<Omit<DialogPrimitive.DialogTitleProps & React.RefAttributes<HTMLHeadingElement>, "ref"> & React.RefAttributes<HTMLHeadingElement>>;
declare const SheetDescription: React.ForwardRefExoticComponent<Omit<DialogPrimitive.DialogDescriptionProps & React.RefAttributes<HTMLParagraphElement>, "ref"> & React.RefAttributes<HTMLParagraphElement>>;

declare const TooltipProvider: React.FC<TooltipPrimitive.TooltipProviderProps>;
/**
 * Tooltip component for displaying information on hover.
 */
declare const Tooltip: React.FC<TooltipPrimitive.TooltipProps>;
declare const TooltipTrigger: React.ForwardRefExoticComponent<TooltipPrimitive.TooltipTriggerProps & React.RefAttributes<HTMLButtonElement>>;
declare const TooltipContent: React.ForwardRefExoticComponent<Omit<TooltipPrimitive.TooltipContentProps & React.RefAttributes<HTMLDivElement>, "ref"> & React.RefAttributes<HTMLDivElement>>;

type SidebarContextProps = {
    state: "expanded" | "collapsed";
    open: boolean;
    setOpen: (open: boolean) => void;
    openMobile: boolean;
    setOpenMobile: (open: boolean) => void;
    isMobile: boolean;
    toggleSidebar: () => void;
};
declare function useSidebar(): SidebarContextProps;
/**
 * Provider for sidebar context/state.
 */
declare const SidebarProvider: React.ForwardRefExoticComponent<Omit<React.ClassAttributes<HTMLDivElement> & React.HTMLAttributes<HTMLDivElement> & {
    defaultOpen?: boolean;
    open?: boolean;
    onOpenChange?: (open: boolean) => void;
}, "ref"> & React.RefAttributes<HTMLDivElement>>;
declare const Sidebar: React.ForwardRefExoticComponent<Omit<React.ClassAttributes<HTMLDivElement> & React.HTMLAttributes<HTMLDivElement> & {
    side?: "left" | "right";
    variant?: "sidebar" | "floating" | "inset";
    collapsible?: "offcanvas" | "icon" | "none";
}, "ref"> & React.RefAttributes<HTMLDivElement>>;
declare const SidebarTrigger: React.ForwardRefExoticComponent<Omit<ButtonProps & React.RefAttributes<HTMLButtonElement>, "ref"> & React.RefAttributes<HTMLButtonElement>>;
declare const SidebarRail: React.ForwardRefExoticComponent<Omit<React.DetailedHTMLProps<React.ButtonHTMLAttributes<HTMLButtonElement>, HTMLButtonElement>, "ref"> & React.RefAttributes<HTMLButtonElement>>;
declare const SidebarInset: React.ForwardRefExoticComponent<Omit<React.DetailedHTMLProps<React.HTMLAttributes<HTMLElement>, HTMLElement>, "ref"> & React.RefAttributes<HTMLDivElement>>;
declare const SidebarInput: React.ForwardRefExoticComponent<Omit<Omit<InputProps, "ref"> & React.RefAttributes<HTMLInputElement>, "ref"> & React.RefAttributes<HTMLInputElement>>;
declare const SidebarHeader: React.ForwardRefExoticComponent<Omit<React.DetailedHTMLProps<React.HTMLAttributes<HTMLDivElement>, HTMLDivElement>, "ref"> & React.RefAttributes<HTMLDivElement>>;
declare const SidebarFooter: React.ForwardRefExoticComponent<Omit<React.DetailedHTMLProps<React.HTMLAttributes<HTMLDivElement>, HTMLDivElement>, "ref"> & React.RefAttributes<HTMLDivElement>>;
declare const SidebarSeparator: React.ForwardRefExoticComponent<Omit<Omit<SeparatorPrimitive.SeparatorProps & React.RefAttributes<HTMLDivElement>, "ref"> & React.RefAttributes<HTMLDivElement>, "ref"> & React.RefAttributes<HTMLDivElement>>;
declare const SidebarContent: React.ForwardRefExoticComponent<Omit<React.DetailedHTMLProps<React.HTMLAttributes<HTMLDivElement>, HTMLDivElement>, "ref"> & React.RefAttributes<HTMLDivElement>>;
declare const SidebarGroup: React.ForwardRefExoticComponent<Omit<React.DetailedHTMLProps<React.HTMLAttributes<HTMLDivElement>, HTMLDivElement>, "ref"> & React.RefAttributes<HTMLDivElement>>;
declare const SidebarGroupLabel: React.ForwardRefExoticComponent<Omit<React.ClassAttributes<HTMLDivElement> & React.HTMLAttributes<HTMLDivElement> & {
    asChild?: boolean;
}, "ref"> & React.RefAttributes<HTMLDivElement>>;
declare const SidebarGroupAction: React.ForwardRefExoticComponent<Omit<React.ClassAttributes<HTMLButtonElement> & React.ButtonHTMLAttributes<HTMLButtonElement> & {
    asChild?: boolean;
}, "ref"> & React.RefAttributes<HTMLButtonElement>>;
declare const SidebarGroupContent: React.ForwardRefExoticComponent<Omit<React.DetailedHTMLProps<React.HTMLAttributes<HTMLDivElement>, HTMLDivElement>, "ref"> & React.RefAttributes<HTMLDivElement>>;
declare const SidebarMenu: React.ForwardRefExoticComponent<Omit<React.DetailedHTMLProps<React.HTMLAttributes<HTMLUListElement>, HTMLUListElement>, "ref"> & React.RefAttributes<HTMLUListElement>>;
declare const SidebarMenuItem: React.ForwardRefExoticComponent<Omit<React.DetailedHTMLProps<React.LiHTMLAttributes<HTMLLIElement>, HTMLLIElement>, "ref"> & React.RefAttributes<HTMLLIElement>>;
declare const SidebarMenuButton: React.ForwardRefExoticComponent<Omit<React.ClassAttributes<HTMLButtonElement> & React.ButtonHTMLAttributes<HTMLButtonElement> & {
    asChild?: boolean;
    isActive?: boolean;
    tooltip?: string | React.ComponentProps<typeof TooltipContent>;
} & VariantProps<(props?: ({
    variant?: "default" | "outline" | null | undefined;
    size?: "default" | "sm" | "lg" | null | undefined;
} & class_variance_authority_types.ClassProp) | undefined) => string>, "ref"> & React.RefAttributes<HTMLButtonElement>>;
declare const SidebarMenuAction: React.ForwardRefExoticComponent<Omit<React.ClassAttributes<HTMLButtonElement> & React.ButtonHTMLAttributes<HTMLButtonElement> & {
    asChild?: boolean;
    showOnHover?: boolean;
}, "ref"> & React.RefAttributes<HTMLButtonElement>>;
declare const SidebarMenuBadge: React.ForwardRefExoticComponent<Omit<React.DetailedHTMLProps<React.HTMLAttributes<HTMLDivElement>, HTMLDivElement>, "ref"> & React.RefAttributes<HTMLDivElement>>;
declare const SidebarMenuSkeleton: React.ForwardRefExoticComponent<Omit<React.ClassAttributes<HTMLDivElement> & React.HTMLAttributes<HTMLDivElement> & {
    showIcon?: boolean;
}, "ref"> & React.RefAttributes<HTMLDivElement>>;
declare const SidebarMenuSub: React.ForwardRefExoticComponent<Omit<React.DetailedHTMLProps<React.HTMLAttributes<HTMLUListElement>, HTMLUListElement>, "ref"> & React.RefAttributes<HTMLUListElement>>;
declare const SidebarMenuSubItem: React.ForwardRefExoticComponent<Omit<React.DetailedHTMLProps<React.LiHTMLAttributes<HTMLLIElement>, HTMLLIElement>, "ref"> & React.RefAttributes<HTMLLIElement>>;
declare const SidebarMenuSubButton: React.ForwardRefExoticComponent<Omit<React.ClassAttributes<HTMLAnchorElement> & React.AnchorHTMLAttributes<HTMLAnchorElement> & {
    asChild?: boolean;
    size?: "sm" | "md";
    isActive?: boolean;
}, "ref"> & React.RefAttributes<HTMLAnchorElement>>;

/**
 * Component for displaying a placeholder skeleton while content is loading.
 */
declare function Skeleton({ className, ...props }: React__default.HTMLAttributes<HTMLDivElement>): react_jsx_runtime.JSX.Element;

type ToasterProps = React__default.ComponentProps<typeof Toaster$1>;
declare const Toaster: ({ ...props }: ToasterProps) => react_jsx_runtime.JSX.Element;

/**
 * Table component for displaying tabular data.
 */
declare const Table: React.ForwardRefExoticComponent<React.HTMLAttributes<HTMLTableElement> & {
    containerClassName?: string;
} & React.RefAttributes<HTMLTableElement>>;
declare const TableHeader: React.ForwardRefExoticComponent<React.HTMLAttributes<HTMLTableSectionElement> & React.RefAttributes<HTMLTableSectionElement>>;
declare const TableBody: React.ForwardRefExoticComponent<React.HTMLAttributes<HTMLTableSectionElement> & React.RefAttributes<HTMLTableSectionElement>>;
declare const TableFooter: React.ForwardRefExoticComponent<React.HTMLAttributes<HTMLTableSectionElement> & React.RefAttributes<HTMLTableSectionElement>>;
declare const TableRow: React.ForwardRefExoticComponent<React.HTMLAttributes<HTMLTableRowElement> & React.RefAttributes<HTMLTableRowElement>>;
declare const TableHead: React.ForwardRefExoticComponent<React.ThHTMLAttributes<HTMLTableCellElement> & React.RefAttributes<HTMLTableCellElement>>;
declare const TableCell: React.ForwardRefExoticComponent<React.TdHTMLAttributes<HTMLTableCellElement> & React.RefAttributes<HTMLTableCellElement>>;
declare const TableCaption: React.ForwardRefExoticComponent<React.HTMLAttributes<HTMLTableCaptionElement> & React.RefAttributes<HTMLTableCaptionElement>>;

/**
 * Tabs component for switching between different views.
 */
declare function Tabs({ className, ...props }: React.ComponentProps<typeof TabsPrimitive.Root>): react_jsx_runtime.JSX.Element;
declare function TabsList({ className, ...props }: React.ComponentProps<typeof TabsPrimitive.List>): react_jsx_runtime.JSX.Element;
declare function TabsTrigger({ className, ...props }: React.ComponentProps<typeof TabsPrimitive.Trigger>): react_jsx_runtime.JSX.Element;
declare function TabsContent({ className, ...props }: React.ComponentProps<typeof TabsPrimitive.Content>): react_jsx_runtime.JSX.Element;

interface TagOption {
    value: string;
    label: string;
}
interface TagInputProps {
    value: string[];
    onChange: (value: string[]) => void;
    /** Suggested values; typed text that matches none is added as typed. */
    options?: TagOption[];
    placeholder?: string;
    className?: string;
}
/** A list of values as removable chips; Enter or comma adds what is typed. */
declare function TagInput({ value, onChange, options, placeholder, className }: TagInputProps): react_jsx_runtime.JSX.Element;

/**
 * Textarea component for multi-line text input.
 */
declare const Textarea: React.ForwardRefExoticComponent<Omit<React.DetailedHTMLProps<React.TextareaHTMLAttributes<HTMLTextAreaElement>, HTMLTextAreaElement>, "ref"> & React.RefAttributes<HTMLTextAreaElement>>;

/**
 * theme-toggle.tsx
 *
 * Light / system / dark selector backed by ThemeContext.
 */
/** Segmented control with the three theme preferences. */
declare const ThemeToggle: ({ className }: {
    className?: string;
}) => react_jsx_runtime.JSX.Element;
/** Single button that flips between light and dark, for tight spots. */
declare const ThemeToggleCompact: ({ className }: {
    className?: string;
}) => react_jsx_runtime.JSX.Element;

interface Field {
    label: string;
    value: React__default.ReactNode;
    /** Columns the field spans on wide screens. */
    span?: 1 | 2 | 3 | 4;
    hidden?: boolean;
}
/** Label/value pairs in a bordered grid: the summary block at the top of detail pages. */
declare function FieldGrid({ fields, columns, className, }: {
    fields: Field[];
    columns?: 2 | 3 | 4;
    className?: string;
}): react_jsx_runtime.JSX.Element;

/**
 * Props for the InfoGrid component.
 * Extends standard HTML div attributes.
 */
interface InfoGridProps extends React__default.HTMLAttributes<HTMLDivElement> {
}
/**
 * Responsive two-column grid for information display.
 *
 * Layout behavior:
 * - Desktop (md+): Two equal-width columns
 * - Mobile: Single column (stacked)
 *
 * @param props - Standard div props including className and children
 * @returns A responsive grid container
 */
declare function InfoGrid({ className, children, ...props }: InfoGridProps): react_jsx_runtime.JSX.Element;

interface InnerSidebarLayoutProps extends React__default.HTMLAttributes<HTMLDivElement> {
    children: React__default.ReactNode;
}
/**
 * Dual-pane layout container inside the main content area.
 * Renders a secondary sidebar alongside the main body (main[sidebar sec - body]).
 */
declare function InnerSidebarLayout({ className, children, ...props }: InnerSidebarLayoutProps): react_jsx_runtime.JSX.Element;
interface InnerSidebarProps extends React__default.HTMLAttributes<HTMLElement> {
    children: React__default.ReactNode;
    widthClass?: string;
}
/**
 * Secondary sidebar pane inside the main area.
 * Sticky on desktop screens with standard width.
 */
declare function InnerSidebar({ className, widthClass, children, ...props }: InnerSidebarProps): react_jsx_runtime.JSX.Element;
interface InnerBodyProps extends React__default.HTMLAttributes<HTMLDivElement> {
    children: React__default.ReactNode;
}
/**
 * Main content body pane inside the dual-pane layout.
 * Expands to fill available width.
 */
declare function InnerBody({ className, children, ...props }: InnerBodyProps): react_jsx_runtime.JSX.Element;
interface InnerSidebarSectionProps extends React__default.HTMLAttributes<HTMLDivElement> {
    title?: string;
    action?: React__default.ReactNode;
    children: React__default.ReactNode;
}
/**
 * Visual section card inside the secondary sidebar.
 */
declare function InnerSidebarSection({ title, action, className, children, ...props }: InnerSidebarSectionProps): react_jsx_runtime.JSX.Element;
interface InnerSidebarNavProps extends React__default.HTMLAttributes<HTMLElement> {
    children: React__default.ReactNode;
}
/**
 * Navigation list container for the secondary sidebar.
 */
declare function InnerSidebarNav({ className, children, ...props }: InnerSidebarNavProps): react_jsx_runtime.JSX.Element;
interface InnerSidebarNavItemProps extends React__default.ButtonHTMLAttributes<HTMLButtonElement> {
    active?: boolean;
    icon?: React__default.ComponentType<{
        className?: string;
    }>;
    label: string;
    count?: number | string;
}
/**
 * Interactive navigation item within the secondary sidebar.
 */
declare function InnerSidebarNavItem({ active, icon: Icon, label, count, className, ...props }: InnerSidebarNavItemProps): react_jsx_runtime.JSX.Element;

interface PageHeaderProps extends Omit<React__default.HTMLAttributes<HTMLDivElement>, "title"> {
    title: React__default.ReactNode;
    /** Badge shown next to the title (state, role...). */
    badge?: React__default.ReactNode;
    /** Small line above the title, e.g. the kind of record. */
    eyebrow?: React__default.ReactNode;
    /** One line under the title. */
    description?: React__default.ReactNode;
    /** Buttons aligned to the right of the title. */
    actions?: React__default.ReactNode;
}
declare function PageHeader({ title, badge, eyebrow, description, actions, className, children, ...props }: PageHeaderProps): react_jsx_runtime.JSX.Element;

/**
 * Props for the PageLayout component.
 * Extends standard HTML div attributes.
 */
interface PageLayoutProps extends React__default.HTMLAttributes<HTMLDivElement> {
}
/**
 * Root page container with consistent spacing.
 *
 * Provides:
 * - Horizontal padding (px-3)
 * - Vertical spacing between children (space-y-2)
 * - Bottom padding to prevent content from touching edges
 * - Full width by default
 *
 * @param props - Standard div props plus custom className
 * @returns A styled page container
 */
declare function PageLayout({ className, children, ...props }: PageLayoutProps): react_jsx_runtime.JSX.Element;

/**
 * Props for the PageSection component.
 */
interface PageSectionProps extends React__default.HTMLAttributes<HTMLDivElement> {
    /** Optional section title displayed as a small uppercase heading */
    title?: string;
    /** Optional action element (button, link) displayed next to the title */
    action?: React__default.ReactNode;
}
/**
 * Section wrapper for organizing page content.
 *
 * Use this component to:
 * - Group related content under a section heading
 * - Maintain consistent vertical spacing between sections
 * - Add section-level actions (e.g., "Add" buttons)
 *
 * @param props - PageSection properties
 * @returns A styled section container
 */
declare function PageSection({ title, action, className, children, ...props }: PageSectionProps): react_jsx_runtime.JSX.Element;

/** Classes shared by route tabs and the Radix tabs in `ui/tabs.tsx`, so both look the same. */
declare const tabListClass = "flex w-full items-end gap-1 border-b border-ink/10 overflow-x-auto";
declare const tabTriggerClass: string;
declare const tabActiveClass = "text-ink font-semibold border-brand-sky bg-brand-sky/[0.08]";
interface RouteTab {
    label: string;
    to: string;
    icon?: React__default.ComponentType<{
        className?: string;
    }>;
    count?: number;
}
/** Tabs that switch between child routes; the active one follows the current path. */
declare function RouteTabs({ tabs, activePath, className, }: {
    tabs: RouteTab[];
    activePath?: string;
    className?: string;
}): react_jsx_runtime.JSX.Element;

interface FeatureCardProps {
    icon: React__default.ComponentType<{
        className?: string;
    }>;
    title: string;
    description: string;
    /** Standards or technologies behind the feature, shown as chips. */
    tags?: string[];
    to?: string;
    action?: string;
}
/** A capability of the node with an optional link to where it is managed. */
declare function FeatureCard({ icon: Icon, title, description, tags, to, action, }: FeatureCardProps): react_jsx_runtime.JSX.Element;

interface StatTileProps {
    label: string;
    value?: number;
    hint?: React__default.ReactNode;
    icon: React__default.ComponentType<{
        className?: string;
    }>;
    to?: string;
}
/** One headline number with its label; a link when `to` is set. */
declare function StatTile({ label, value, hint, icon: Icon, to }: StatTileProps): react_jsx_runtime.JSX.Element;

/**
 * Single option within a table filter.
 */
interface TableFilterOption {
    label: string;
    value: string;
}
/**
 * Declarative filter definition for the DataTable toolbar.
 *
 * @template T - The type of data items in the table
 */
interface TableFilter<T = any> {
    /** Unique identifier for the filter */
    id: string;
    /** Display label for the filter dropdown */
    label: string;
    /** Selectable options. Use value "all" for matching all items */
    options: TableFilterOption[];
    /** Controlled value for the filter */
    value?: string;
    /** Default selected value when uncontrolled */
    defaultValue?: string;
    /** Callback fired when the filter value changes */
    onChange?: (value: string) => void;
    /** Field key on data item to filter against when filterFn is omitted */
    accessorKey?: keyof T;
    /** Custom filter logic */
    filterFn?: (item: T, selectedValue: string) => boolean;
}
/**
 * Column definition for the DataTable.
 *
 * @template T - The type of data items in the table
 */
interface Column<T> {
    /** Header text or component displayed in the table header */
    header: React__default.ReactNode | string;
    /**
     * Key to access the value from the data item.
     * Used when no custom cell renderer is provided.
     */
    accessorKey?: keyof T;
    /**
     * Custom cell renderer function.
     * Takes precedence over accessorKey when provided.
     */
    cell?: (item: T) => React__default.ReactNode;
    /** Additional CSS class for the cells */
    className?: string;
    /** Additional CSS class for the header cell */
    headerClassName?: string;
    /** Only creation-date columns sort; `false` turns that off too. */
    sortable?: boolean;
    /** Custom sort key name */
    sortKey?: string;
    /** Custom sort value extractor */
    sortValue?: (item: T) => string | number | boolean | Date | null | undefined;
    /** Whether this column should be searched during filtering. Defaults to true */
    searchable?: boolean;
    /** Custom search text extractor for this column */
    searchValue?: (item: T) => string;
}
/**
 * Props for the DataTable component.
 *
 * @template T - The type of data items in the table
 */
interface DataTableProps<T> {
    /** Column definitions */
    columns: Column<T>[];
    /** Array of data items or Paginated response envelope */
    data: T[] | {
        items: T[];
        total?: number;
        nextCursor?: string;
    } | null | undefined;
    /** Function to extract a unique key from each item */
    keyExtractor: (item: T) => string;
    /** Optional handler for row clicks */
    onRowClick?: (item: T) => void;
    /** Additional CSS class for the table */
    className?: string;
    /** Additional CSS class for the table container */
    containerClassName?: string;
    /**
     * Message shown when `data` is empty. Defaults to a generic
     * "No records found". Pass a domain-specific phrase for clarity.
     */
    emptyMessage?: React__default.ReactNode;
    /**
     * Optional secondary description shown below `emptyMessage` when `data` is empty.
     */
    emptyDescription?: React__default.ReactNode;
    /** Table title displayed in the toolbar */
    title?: React__default.ReactNode;
    /** Subtitle displayed under the title */
    subtitle?: React__default.ReactNode;
    /** Extra actions/buttons displayed on the toolbar */
    toolbarActions?: React__default.ReactNode;
    /** Whether client-side search is enabled. Defaults to true. */
    searchable?: boolean;
    /** Custom placeholder for search input */
    searchPlaceholder?: string;
    /** Custom filter function for live text search */
    filterFn?: (item: T, query: string) => boolean;
    /** Available declarative filters displayed in the toolbar */
    filters?: TableFilter<T>[];
    /** Default column key to sort by */
    defaultSortKey?: string;
    /** Default sort direction */
    defaultSortDirection?: "asc" | "desc";
    /** Whether client-side pagination is enabled. Defaults to true. */
    paginated?: boolean;
    /** Initial or fixed items per page. Defaults to 10. Pass -1 or Infinity to show all. */
    pageSize?: number;
    /** Available options in the rows-per-page dropdown. Defaults to [10, 20, 50, 100]. */
    pageSizeOptions?: number[];
    /** Whether to hide the search and filter toolbar */
    hideToolbar?: boolean;
    /** Whether to hide the record count badge */
    hideCount?: boolean;
    /** Enable server-side pagination, sorting, and filtering via query parameters */
    serverSide?: boolean;
    /** Total number of records across all pages when using server-side pagination */
    totalCount?: number;
    /** Next cursor for cursor-based pagination */
    nextCursor?: string;
    /** Controlled query state object */
    queryParams?: Partial<TableQueryParams>;
    /** Callback fired when sorting, filtering, pagination, or search changes */
    onQueryChange?: (params: Partial<TableQueryParams>) => void;
    /** Automatically synchronize query parameters with the browser URL. Defaults to true */
    syncUrlParams?: boolean;
    /** Whether the table is currently fetching new data in background */
    loading?: boolean;
}
/**
 * Enterprise table component with built-in client/server sorting, filtering, and pagination.
 */
declare function DataTable<T extends Record<string, any> = any>({ columns, data, keyExtractor, onRowClick, className, containerClassName, emptyMessage, emptyDescription, title, subtitle, toolbarActions, searchable, searchPlaceholder, filterFn, filters, defaultSortKey, defaultSortDirection, paginated, pageSize, pageSizeOptions, hideToolbar, hideCount, serverSide, totalCount, nextCursor, queryParams, onQueryChange, syncUrlParams, loading, }: DataTableProps<T>): react_jsx_runtime.JSX.Element;

type SortDirection = "asc" | "desc";
interface SortConfig<K extends string> {
    key: K;
    direction: SortDirection;
}
interface SortableHeaderProps<K extends string> {
    label: string;
    sortKey: K;
    sortConfig: SortConfig<K> | null;
    onSort: (key: K) => void;
    className?: string;
}
declare function SortableHeader<K extends string>({ label, sortKey, sortConfig, onSort, className, }: SortableHeaderProps<K>): react_jsx_runtime.JSX.Element;

interface IdentifierProps {
    value?: string | null;
    className?: string;
    /** Characters kept from the end of a URN/DID; the full value stays in the tooltip. */
    tail?: number;
    copyable?: boolean;
}
/** A long URN/DID shown short in monospace, full on hover, with a copy button. */
declare function Identifier({ value, className, tail, copyable }: IdentifierProps): react_jsx_runtime.JSX.Element;
declare namespace Identifier {
    var short: (value: string, tail?: number) => string;
}

interface ParticipantDirectoryInfo {
    nick?: string | null;
    isSelf?: boolean;
}
type ParticipantResolverFn = (id: string) => ParticipantDirectoryInfo | undefined;
declare const ParticipantDirectoryProvider: React__default.Provider<ParticipantResolverFn>;
declare const useParticipantResolver: () => ParticipantResolverFn;
interface ParticipantLabelProps {
    id?: string | null;
    nick?: string | null;
    isSelf?: boolean;
    className?: string;
    /** Hides the identifier line under the name. */
    compact?: boolean;
}
/** A participant by its nickname, marked when it is this agent, with its DID underneath. */
declare function ParticipantLabel({ id, nick: explicitNick, isSelf: explicitIsSelf, className, compact, }: ParticipantLabelProps): react_jsx_runtime.JSX.Element;

interface StateStepperProps {
    /** The happy path of the protocol, in order. */
    steps: string[];
    /** Current state of the process; may be off the path (TERMINATED, SUSPENDED). */
    current: string;
    /** States the process went through, from its message history; unknown while it loads. */
    visited?: Set<string>;
    /** Off-path states and the path step they hang from. */
    detours?: Record<string, {
        at?: string;
        tone: "danger" | "warn";
    }>;
}
/** The protocol path as steps: done, current, skipped or pending, plus a terminal detour. */
declare function StateStepper({ steps, current, visited, detours }: StateStepperProps): react_jsx_runtime.JSX.Element;

type Props = {
    open: boolean;
    onClose: () => void;
    anchorRef?: React__default.RefObject<HTMLElement>;
    width?: number | string;
    align?: "center" | "left";
    children?: React__default.ReactNode;
    content: React__default.ReactNode;
    title: String;
    sectionTitle: String;
    step?: String;
};
declare function WizardDialog({ open, step, onClose, anchorRef, width, align, content, title, sectionTitle, children, }: Props): react_jsx_runtime.JSX.Element | null;

interface WizardEndDialogProps {
    open: boolean;
    onClose?: () => void;
    title?: React__default.ReactNode;
    content?: React__default.ReactNode;
    actionHref?: string;
    actionLabel?: string;
    sectionTitle: string;
}
declare function WizardEndDialog({ open, title, content, actionHref, actionLabel, sectionTitle, }: WizardEndDialogProps): react_jsx_runtime.JSX.Element | null;

type ButtonVariant = "default" | "destructive" | "outline" | "secondary" | "ghost" | "link";
interface BaseProcessDialogProps<TFormValues extends FieldValues = FieldValues> {
    /** Dialog title displayed in the header */
    title: string;
    /** Description text below the title */
    description: string | ReactNode;
    /** Info items to display in the InfoList component */
    infoItems: InfoItemProps[];
    /** Label for the submit button */
    submitLabel: string;
    /** Visual variant for the submit button */
    submitVariant?: ButtonVariant;
    /** Callback when form is submitted */
    onSubmit: (data: TFormValues, form: UseFormReturn<TFormValues>) => void | Promise<void>;
    /** Optional default values for the form */
    defaultValues?: DefaultValues<TFormValues>;
    /** Optional content to render before the InfoList */
    beforeInfoContent?: ReactNode;
    /** Optional content to render after the InfoList */
    afterInfoContent?: ReactNode;
    /** Optional form fields to render (for dialogs with input fields) */
    formFields?: ReactNode;
    /** Custom className for the DialogContent */
    contentClassName?: string;
    /** Whether to use scrollable body layout */
    scrollable?: boolean;
    /** Label for the cancel button (defaults to "Cancel") */
    cancelLabel?: string;
    /** Whether to hide the info list entirely */
    hideInfoList?: boolean;
    /** Whether to disable the submit button */
    disabledSubmit?: boolean;
    /** Optional external form instance to use (for shared state with custom fields) */
    form?: UseFormReturn<TFormValues>;
}
/**
 * Base dialog component for process-related actions.
 *
 * Provides a consistent structure for dialogs that:
 * 1. Display process information via InfoList
 * 2. Handle form submission
 * 3. Have Cancel/Submit action buttons
 */
declare function BaseProcessDialog<TFormValues extends FieldValues = FieldValues>({ title, description, infoItems, submitLabel, submitVariant, onSubmit, defaultValues, beforeInfoContent, afterInfoContent, formFields, contentClassName, scrollable, cancelLabel, hideInfoList, disabledSubmit, form: externalForm, }: BaseProcessDialogProps<TFormValues>): react_jsx_runtime.JSX.Element;

interface NavItem {
    /** Display title for the menu item */
    title: string;
    /** Route URL to navigate to */
    url: string;
    /** Lucide icon component to display */
    icon: React__default.ComponentType<{
        className?: string;
    }>;
    /** Path prefix that marks the item active; defaults to `url` */
    section?: string;
}
interface NavGroup {
    title: string;
    items: NavItem[];
}
interface AppShellSidebarProps {
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
    footerSlot?: React__default.ReactNode;
    footerClassName?: string;
}
declare function AppShellSidebar({ productName, homeTo, navGroups, activePath, logoOnDark, logoOnLight, footerSlot, footerClassName, }: AppShellSidebarProps): react_jsx_runtime.JSX.Element;

interface BreadcrumbItemData {
    key: string;
    href: string;
    label: string;
    isDynamic: boolean;
}
declare const DEFAULT_ROUTE_LABELS: Record<string, string>;
interface AppShellHeaderProps {
    user?: SessionUser | null;
    onSignOut?: () => void;
    activePath?: string;
    routeLabels?: Record<string, string>;
    rootBreadcrumb?: {
        label: string;
        href: string;
    };
    /** Extra action widgets rendered left of ThemeToggle (e.g., NotificationBell) */
    rightActionsSlot?: React__default.ReactNode;
    /** Optional quick links rendered inside the user popover */
    quickLinksSlot?: React__default.ReactNode;
}
declare function AppShellHeader({ user, onSignOut, activePath, routeLabels, rootBreadcrumb, rightActionsSlot, quickLinksSlot, }: AppShellHeaderProps): react_jsx_runtime.JSX.Element;

interface SignInScreenProps {
    title?: string;
    description?: string;
    tagline?: string;
    pillars?: string;
    logoSrc?: string;
    onSignIn?: () => void;
}
declare function SignInScreen({ title, description, tagline, pillars, logoSrc, onSignIn, }: SignInScreenProps): react_jsx_runtime.JSX.Element;

interface GeneralErrorComponentProps {
    error: Error;
    reset?: () => void;
    onRetry?: () => void;
}
/**
 * Component for displaying a general error message.
 */
declare function GeneralErrorComponent({ error, reset, onRetry }: GeneralErrorComponentProps): react_jsx_runtime.JSX.Element;

/**
 * Global loading indicator component that shows when queries are fetching
 * or any mutation is in flight.
 */
declare function GlobalLoadingIndicator(): react_jsx_runtime.JSX.Element | null;

export { Accordion, AccordionContent, AccordionItem, AccordionTrigger, ActivePathProvider, AppShellHeader, type AppShellHeaderProps, AppShellSidebar, type AppShellSidebarProps, Avatar, Avatar as AvatarImg, Badge, type BadgeProps, type BadgeRole, type BadgeState, BaseProcessDialog, type BaseProcessDialogProps, type BodyType, BooleanBadge, type BooleanBadgeProps, Breadcrumb, BreadcrumbEllipsis, BreadcrumbItem, type BreadcrumbItemData, BreadcrumbLink, BreadcrumbList, BreadcrumbPage, BreadcrumbSeparator, BubbleBackground, type BubbleBackgroundProps, type BubbleColors, Button, type ButtonProps, type ButtonSizes, type ButtonVariant, Card, CardContent, CardDescription, CardFooter, CardHeader, type CardProps, CardTitle, type ChartConfig, ChartContainer, ChartLegend, ChartLegendContent, ChartStyle, ChartTooltip, ChartTooltipContent, Checkbox, type Column, Command, CommandDialog, CommandEmpty, CommandGroup, CommandInput, CommandItem, CommandList, CommandSeparator, CommandShortcut, ConfirmDialog, type ConfirmDialogProps, CountBadge, type CustomInstanceOptions, DATE_FORMAT, DATE_TIME_FORMAT, DEFAULT_ROUTE_LABELS, DataTable, type DataTableProps, Dialog, DialogBody, DialogClose, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogOverlay, DialogPortal, DialogTitle, DialogTrigger, Drawer, DrawerBody, DrawerClose, DrawerContent, DrawerDescription, DrawerFooter, DrawerHeader, DrawerOverlay, DrawerPortal, DrawerTitle, DrawerTrigger, DropdownMenu, DropdownMenuCheckboxItem, DropdownMenuContent, DropdownMenuGroup, DropdownMenuItem, DropdownMenuLabel, DropdownMenuPortal, DropdownMenuRadioGroup, DropdownMenuRadioItem, DropdownMenuSeparator, DropdownMenuShortcut, DropdownMenuSub, DropdownMenuSubContent, DropdownMenuSubTrigger, DropdownMenuTrigger, type ErrorType, FeatureCard, type FeatureCardProps, type Field, FieldGrid, Form, FormControl, FormDescription, FormField, FormItem, FormLabel, FormMessage, FormatDate, GeneralErrorComponent, type GeneralErrorComponentProps, GlobalLoadingIndicator, Heading, type HeadingLevel, type HeadingProps, Identifier, type IdentifierProps, InfoGrid, type InfoGridProps, type InfoItemProps, type InfoItemValue, InfoList, InfoListItem, type InfoListProps, InnerBody, type InnerBodyProps, InnerSidebar, InnerSidebarLayout, type InnerSidebarLayoutProps, InnerSidebarNav, InnerSidebarNavItem, type InnerSidebarNavItemProps, type InnerSidebarNavProps, type InnerSidebarProps, InnerSidebarSection, type InnerSidebarSectionProps, Input, type InputProps, Label, type LinkLike, type LinkProps, LinkProvider, List, ListItem, ListItemDate, ListItemKey, MessageBody, MessageContent, MessageLog, MessageMeta, MessageMetaContainer, type MessageMetaProps, MessageTimestamp, MessageTitle, type NavGroup, type NavItem, NavigationProvider, type NavigationProviderProps, NoFurtherActions, PageHeader, type PageHeaderProps, PageLayout, type PageLayoutProps, PageSection, type PageSectionProps, type ParticipantDirectoryInfo, ParticipantDirectoryProvider, ParticipantLabel, type ParticipantLabelProps, type ParticipantResolverFn, Popover, PopoverAnchor, PopoverContent, PopoverTrigger, RelativeTime, type RequestConfig, type ResolvedTheme, RoleHeader, type RoleType, type RouteTab, RouteTabs, SegmentedControl, type SegmentedOption, Select, SelectContent, SelectGroup, SelectItem, SelectLabel, SelectScrollDownButton, SelectScrollUpButton, SelectSeparator, SelectTrigger, SelectValue, Separator, type SessionUser, Sheet, SheetClose, SheetContent, SheetDescription, SheetFooter, SheetHeader, SheetOverlay, SheetPortal, SheetTitle, SheetTrigger, Sidebar, SidebarContent, SidebarFooter, SidebarGroup, SidebarGroupAction, SidebarGroupContent, SidebarGroupLabel, SidebarHeader, SidebarInput, SidebarInset, SidebarMenu, SidebarMenuAction, SidebarMenuBadge, SidebarMenuButton, SidebarMenuItem, SidebarMenuSkeleton, SidebarMenuSub, SidebarMenuSubButton, SidebarMenuSubItem, SidebarProvider, SidebarRail, SidebarSeparator, SidebarTrigger, SignInScreen, type SignInScreenProps, Skeleton, type SortConfig, type SortDirection, SortableHeader, StatTile, type StatTileProps, StateStepper, type StateStepperProps, THEME_STORAGE_KEY, Table, TableBody, TableCaption, TableCell, type TableFilter, type TableFilterOption, TableFooter, TableHead, TableHeader, type TableQueryParams, TableRow, Tabs, TabsContent, TabsList, TabsTrigger, TagInput, type TagInputProps, type TagOption, Textarea, ThemeContext, ThemeContextProvider, type ThemeContextProviderProps, type ThemeContextType, type ThemePreference, ThemeToggle, ThemeToggleCompact, Toaster, Tooltip, TooltipContent, TooltipProvider, TooltipTrigger, UNAUTHORIZED_EVENT, type UseTableQueryParamsOptions, WizardDialog, WizardEndDialog, type WizardEndDialogProps, badgeVariants, buttonVariants, cardVariants, cn, customInstance, decodePathId, defaultAvatarImg, dsAgentLogoDark, dsAgentLogoLight, encodePathId, eunomiaLogoDark, eunomiaLogoLight, formatIdentifier, formatKey, formatOperator, formatUrn, formatValue, generateRandomString, getApiGatewayBase, getFriendlyVCType, getSessionUser, grantBadgeState, loadSession, mapRoleToVariant, mapToSortParam, mergeStateAndAttribute, pathIdParams, roleVariants, sessionDisplayName, setApiGatewayBase, signIn, signOut, tabActiveClass, tabListClass, tabTriggerClass, useActivePath, useFormField, useIsMobile, useLink, useParticipantResolver, useSidebar, useTableQueryParams, useTheme };
