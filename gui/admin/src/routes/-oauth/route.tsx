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

import { createFileRoute, Link, Outlet, useRouterState } from "@tanstack/react-router";
import { PageLayout } from "shared/src/components/layout/PageLayout";
import { PageHeader } from "shared/src/components/layout/PageHeader";
import { cn } from "shared/src/lib/utils";

const OAuthLayout = () => {
  const routerState = useRouterState();
  const pathname = routerState.location.pathname;

  const tabs = [
    { label: "Clients", to: "/oauth/clients" },
    { label: "Personal Access Tokens", to: "/oauth/pats" },
  ];

  return (
    <PageLayout>
      <PageHeader title="OAuth 2.0 & Access Security" />

      <div className="flex gap-1 border-b border-ink/10 mb-6 w-full">
        {tabs.map((tab) => {
          const isActive = pathname === tab.to || pathname.startsWith(tab.to + "/");
          return (
            <Link
              key={tab.to}
              to={tab.to}
              className={cn(
                "px-4 py-2 text-sm font-medium border-b-2 transition-all",
                isActive
                  ? "border-primary text-primary"
                  : "border-transparent text-muted-foreground hover:text-foreground",
              )}
            >
              {tab.label}
            </Link>
          );
        })}
      </div>

      <Outlet />
    </PageLayout>
  );
};

export const Route = createFileRoute("/oauth")({
  component: OAuthLayout,
});
