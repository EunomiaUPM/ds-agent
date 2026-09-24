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

import { useMemo, useState } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { Building2 } from "lucide-react";
import { listOAuthUsers } from "shared/src/data/orval/oauth/oauth";
import { OAuthUser } from "shared/src/data/orval/model";
import { decodeJwtPayload, getSessionToken } from "shared/src/lib/session";
import { getActingTenant, setActingTenant } from "shared/src/lib/tenant";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "shared/src/components/ui/select";

const ALL_TENANTS = "__all__";

/**
 * Lets an admin act on one tenant or on all of them. Tenants are the ones of the OAuth users,
 * since there is no tenant registry. Hidden for owners and readers, who have a single tenant.
 */
export const TenantSelector = () => {
  const queryClient = useQueryClient();
  const token = getSessionToken();
  const claims = token ? decodeJwtPayload<{ sub?: string; role?: string }>(token) : null;
  const isAdmin = claims?.role?.toLowerCase() === "admin";
  const [tenant, setTenant] = useState<string | null>(getActingTenant());

  const { data: usersResponse } = useQuery({
    queryKey: ["tenant-selector", "oauth-users"],
    queryFn: () => listOAuthUsers({ limit: 100 }, { _allTenants: true } as RequestInit),
    enabled: isAdmin,
  });

  const tenants = useMemo(() => {
    const users: OAuthUser[] =
      usersResponse?.status === 200 && Array.isArray(usersResponse.data) ? usersResponse.data : [];
    const ids = new Set(users.map((u) => u.tenantId).filter(Boolean));
    if (claims?.sub) ids.add(claims.sub);
    if (tenant) ids.add(tenant);
    return Array.from(ids).sort();
  }, [usersResponse, claims?.sub, tenant]);

  if (!isAdmin) return null;

  const onChange = (value: string) => {
    const next = value === ALL_TENANTS ? null : value;
    setActingTenant(next);
    setTenant(next);
    queryClient.invalidateQueries();
  };

  return (
    <Select value={tenant ?? ALL_TENANTS} onValueChange={onChange}>
      <SelectTrigger className="h-7 w-44 text-xs" title="Tenant you are acting on">
        <Building2 className="h-3.5 w-3.5 mr-1 shrink-0" />
        <SelectValue />
      </SelectTrigger>
      <SelectContent>
        <SelectItem value={ALL_TENANTS} className="text-xs">
          All tenants
        </SelectItem>
        {tenants.map((t) => (
          <SelectItem key={t} value={t} className="text-xs font-mono">
            {t}
          </SelectItem>
        ))}
      </SelectContent>
    </Select>
  );
};
