import React, { createContext, useContext } from "react";

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

const DefaultLink = React.forwardRef<HTMLAnchorElement, LinkProps>(({ to, ...rest }, ref) => (
  <a ref={ref} href={to} {...rest} />
));
DefaultLink.displayName = "DefaultLink";

const LinkContext = createContext<LinkLike>(DefaultLink);
const ActivePathContext = createContext<string>("");

export const LinkProvider = LinkContext.Provider;
export const ActivePathProvider = ActivePathContext.Provider;

export const useLink = (): LinkLike => useContext(LinkContext);

export const useActivePath = (): string => {
  const ctxPath = useContext(ActivePathContext);
  if (ctxPath) return ctxPath;
  if (typeof window !== "undefined") return window.location.pathname;
  return "";
};

export interface NavigationProviderProps {
  link: LinkLike;
  activePath?: string;
  children: React.ReactNode;
}

export function NavigationProvider({ link, activePath = "", children }: NavigationProviderProps) {
  return (
    <LinkContext.Provider value={link}>
      <ActivePathContext.Provider value={activePath}>{children}</ActivePathContext.Provider>
    </LinkContext.Provider>
  );
}
