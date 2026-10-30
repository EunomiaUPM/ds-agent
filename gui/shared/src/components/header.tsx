/**
 * header.tsx
 *
 * Main application header component with dynamic breadcrumb navigation
 * and user action controls, powered by @eunomia/ui's AppShellHeader.
 */

import React, { useContext } from "react";
import { Link } from "@tanstack/react-router";
import { ExternalLink, Radio } from "lucide-react";
import { AppShellHeader, type SessionUser } from "@eunomia/ui";
import { AuthContext, AuthContextType } from "shared/src/context/AuthContext";
import { NotificationBell } from "shared/src/components/NotificationBell";

export const Header = ({
  user,
  onSignOut,
}: { user?: SessionUser | null; onSignOut?: () => void } = {}) => {
  const authCtx = useContext<AuthContextType | null>(AuthContext);

  const handleSignOut = () => {
    authCtx?.unsetAuthentication?.();
    onSignOut?.();
  };

  return (
    <AppShellHeader
      user={user}
      onSignOut={handleSignOut}
      rightActionsSlot={<NotificationBell />}
      quickLinksSlot={
        <Link
          to="/events/feed"
          className="flex items-center justify-between px-2.5 py-1.5 rounded-md text-xs text-foreground/80 hover:text-ink hover:bg-ink/5 transition-colors"
        >
          <span className="flex items-center gap-2">
            <Radio className="h-3.5 w-3.5 text-emerald-700 dark:text-emerald-400" />
            Live Event Bus
          </span>
          <ExternalLink className="h-3 w-3 opacity-40" />
        </Link>
      }
    />
  );
};
