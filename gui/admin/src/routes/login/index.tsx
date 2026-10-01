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

import React, { useEffect } from "react";
import { createFileRoute, useNavigate } from "@tanstack/react-router";
import { isSessionActive } from "../../lib/session";
import { Loader2 } from "lucide-react";

export const Route = createFileRoute("/login/")({
  component: LoginComponent,
});

function LoginComponent() {
  const navigate = useNavigate();

  useEffect(() => {
    if (isSessionActive()) {
      navigate({ to: "/" });
      const timer = setTimeout(() => {
        if (window.location.pathname.includes("/login")) {
          window.location.replace("/admin/");
        }
      }, 300);
      return () => clearTimeout(timer);
    }
  }, [navigate]);

  return (
    <div className="flex h-[60vh] w-full items-center justify-center">
      <div className="flex flex-col items-center gap-3">
        <Loader2 className="h-8 w-8 animate-spin text-brand-sky" />
        <p className="text-sm text-muted-foreground font-mono">Redirecting to console...</p>
      </div>
    </div>
  );
}
