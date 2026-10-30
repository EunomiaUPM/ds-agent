import React, { createContext, useContext } from "react";
import { Link as TanStackLink, useRouterState } from "@tanstack/react-router";

export interface LinkProps {
  to: string;
  className?: string;
  children?: React.ReactNode;
  title?: string;
  role?: string;
  "aria-selected"?: boolean;
  onClick?: React.MouseEventHandler<HTMLAnchorElement>;
}

export type LinkLike = React.ComponentType<LinkProps & React.RefAttributes<HTMLAnchorElement>>;

const DefaultTanStackLink = React.forwardRef<HTMLAnchorElement, LinkProps>(
  ({ to, ...rest }, ref) => <TanStackLink ref={ref} to={to as any} {...rest} />,
);
DefaultTanStackLink.displayName = "DefaultTanStackLink";

const LinkContext = createContext<LinkLike>(DefaultTanStackLink);
const ActivePathContext = createContext<string | null>(null);

export const LinkProvider = LinkContext.Provider;
export const ActivePathProvider = ActivePathContext.Provider;

export const useLink = (): LinkLike => useContext(LinkContext);

export const useActivePath = (): string => {
  const ctxPath = useContext(ActivePathContext);
  const routerPath = useRouterState({ select: (s) => s.location.pathname });
  return ctxPath ?? routerPath ?? "";
};

export interface NavigationProviderProps {
  link?: LinkLike;
  activePath?: string;
  children: React.ReactNode;
}

export function NavigationProvider({
  link = DefaultTanStackLink,
  activePath,
  children,
}: NavigationProviderProps) {
  return (
    <LinkContext.Provider value={link}>
      <ActivePathContext.Provider value={activePath ?? null}>
        {children}
      </ActivePathContext.Provider>
    </LinkContext.Provider>
  );
}
