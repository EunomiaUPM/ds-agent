/*
 * Copyright (C) 2026 - Universidad Politécnica de Madrid - UPM
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program. If not, see <https://www.gnu.org/licenses/>.
 */

import React from "react";
import { cn } from "shared/src/lib/utils";

export interface InnerSidebarLayoutProps extends React.HTMLAttributes<HTMLDivElement> {
  children: React.ReactNode;
}

/**
 * Dual-pane layout container inside the main content area.
 * Renders a secondary sidebar alongside the main body (main[sidebar sec - body]).
 */
export function InnerSidebarLayout({ className, children, ...props }: InnerSidebarLayoutProps) {
  return (
    <div
      className={cn(
        "flex flex-col lg:flex-row gap-6 w-full items-start min-w-0 transition-all duration-200",
        className,
      )}
      {...props}
    >
      {children}
    </div>
  );
}

export interface InnerSidebarProps extends React.HTMLAttributes<HTMLElement> {
  children: React.ReactNode;
  widthClass?: string;
}

/**
 * Secondary sidebar pane inside the main area.
 * Sticky on desktop screens with standard width.
 */
export function InnerSidebar({
  className,
  widthClass = "w-full lg:w-80 xl:w-96",
  children,
  ...props
}: InnerSidebarProps) {
  return (
    <aside
      className={cn(
        "flex-shrink-0 flex flex-col gap-4 lg:sticky lg:top-4 h-fit min-w-0",
        widthClass,
        className,
      )}
      {...props}
    >
      {children}
    </aside>
  );
}

export interface InnerBodyProps extends React.HTMLAttributes<HTMLDivElement> {
  children: React.ReactNode;
}

/**
 * Main content body pane inside the dual-pane layout.
 * Expands to fill available width.
 */
export function InnerBody({ className, children, ...props }: InnerBodyProps) {
  return (
    <div className={cn("flex-1 min-w-0 w-full flex flex-col gap-6", className)} {...props}>
      {children}
    </div>
  );
}

export interface InnerSidebarSectionProps extends React.HTMLAttributes<HTMLDivElement> {
  title?: string;
  action?: React.ReactNode;
  children: React.ReactNode;
}

/**
 * Visual section card inside the secondary sidebar.
 */
export function InnerSidebarSection({
  title,
  action,
  className,
  children,
  ...props
}: InnerSidebarSectionProps) {
  return (
    <div
      className={cn(
        "rounded-xl border border-ink/10 bg-card/60 backdrop-blur-sm p-4 flex flex-col gap-3 shadow-sm",
        className,
      )}
      {...props}
    >
      {(title || action) && (
        <div className="flex items-center justify-between gap-2 pb-2 border-b border-ink/5">
          {title && (
            <h4 className="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
              {title}
            </h4>
          )}
          {action && <div>{action}</div>}
        </div>
      )}
      {children}
    </div>
  );
}

export interface InnerSidebarNavProps extends React.HTMLAttributes<HTMLElement> {
  children: React.ReactNode;
}

/**
 * Navigation list container for the secondary sidebar.
 */
export function InnerSidebarNav({ className, children, ...props }: InnerSidebarNavProps) {
  return (
    <nav className={cn("flex flex-col gap-1", className)} {...props}>
      {children}
    </nav>
  );
}

export interface InnerSidebarNavItemProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  active?: boolean;
  icon?: React.ComponentType<{ className?: string }>;
  label: string;
  count?: number | string;
}

/**
 * Interactive navigation item within the secondary sidebar.
 */
export function InnerSidebarNavItem({
  active,
  icon: Icon,
  label,
  count,
  className,
  ...props
}: InnerSidebarNavItemProps) {
  return (
    <button
      type="button"
      className={cn(
        "flex items-center justify-between w-full px-3 py-2 rounded-lg text-xs font-medium transition-all duration-150 border",
        active
          ? "bg-primary/20 text-brand-sky border-primary/40 shadow-sm"
          : "text-muted-foreground hover:text-foreground hover:bg-ink/5 border-transparent",
        className,
      )}
      {...props}
    >
      <div className="flex items-center gap-2.5 truncate">
        {Icon && (
          <Icon
            className={cn(
              "h-4 w-4 flex-shrink-0",
              active ? "text-brand-sky" : "text-muted-foreground",
            )}
          />
        )}
        <span className="truncate">{label}</span>
      </div>
      {count !== undefined && (
        <span
          className={cn(
            "text-xs font-mono px-2 py-0.5 rounded-full border",
            active
              ? "bg-primary/30 text-brand-sky border-primary/50"
              : "bg-ink/5 text-muted-foreground border-ink/5",
          )}
        >
          {count}
        </span>
      )}
    </button>
  );
}
