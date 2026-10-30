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

import { createFileRoute, Outlet } from "@tanstack/react-router";
import { RouteTabs } from "@eunomia/ui";
import { PageLayout } from "@eunomia/ui";
import { PageHeader } from "@eunomia/ui";

const EventsLayout = () => {
  const tabs = [
    { label: "Feed", to: "/events/feed" },
    { label: "Webhook subscriptions", to: "/events/subscriptions" },
    { label: "Dead letters", to: "/events/dlq" },
  ];

  return (
    <PageLayout>
      <PageHeader
        title="Events"
        description="Domain events published on the bus by every agent of this connector."
      />

      <RouteTabs tabs={tabs} />

      <Outlet />
    </PageLayout>
  );
};

export const Route = createFileRoute("/events")({
  component: EventsLayout,
});
