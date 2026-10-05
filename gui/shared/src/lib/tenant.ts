// Acting tenant of the former tenant model, sent as `x-tenant-id`. The agents no longer read
// it: each user sees what its role lets it. Kept as it was, out of use.
//
// /*
//  * Copyright (C) 2026 - Universidad Politécnica de Madrid - UPM
//  *
//  * This program is free software: you can redistribute it and/or modify
//  * it under the terms of the GNU General Public License as published by
//  * the Free Software Foundation, either version 3 of the License, or
//  * (at your option) any later version.
//  *
//  * This program is distributed in the hope that it will be useful,
//  * but WITHOUT ANY WARRANTY; without even the implied warranty of
//  * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
//  * GNU General Public License for more details.
//  *
//  * You should have received a copy of the GNU General Public License
//  * along with this program. If not, see <https://www.gnu.org/licenses/>.
//  */
//
// // Tenant an admin acts on; unset means every tenant. Owners and readers act on the
// // tenant of their token, and the backend rejects any other.
//
// const ACTING_TENANT_KEY = "eunomia_acting_tenant";
//
// export const TENANT_CHANGED_EVENT = "eunomia:tenant-changed";
//
// export const getActingTenant = (): string | null => {
//   if (typeof localStorage === "undefined") return null;
//   return localStorage.getItem(ACTING_TENANT_KEY);
// };
//
// export const setActingTenant = (tenant: string | null): void => {
//   if (typeof localStorage === "undefined") return;
//   if (tenant) {
//     localStorage.setItem(ACTING_TENANT_KEY, tenant);
//   } else {
//     localStorage.removeItem(ACTING_TENANT_KEY);
//   }
//   if (typeof window !== "undefined") {
//     window.dispatchEvent(new Event(TENANT_CHANGED_EVENT));
//   }
// };

// Keeps the file a module (it only holds commented-out code).
export {};
