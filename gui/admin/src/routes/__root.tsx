import React, { useCallback, useEffect, useState } from "react";
import { QueryClient } from "@tanstack/react-query";
import { createRootRouteWithContext, Outlet } from "@tanstack/react-router";
import { TanStackRouterDevtools } from "@tanstack/router-devtools";
import { Loader2 } from "lucide-react";
import {
  GeneralErrorComponent,
  GlobalLoadingIndicator,
  loadSession,
  ParticipantDirectoryProvider,
  type SessionUser,
  SidebarInset,
  SidebarProvider,
  SignInScreen,
  signOut,
  Toaster,
  UNAUTHORIZED_EVENT,
} from "@eunomia/ui";
import { AppSidebar } from "shared/src/components/AppSidebar";
import { Header } from "shared/src/components/header";
import { LiveContextProvider } from "shared/src/context/LiveContext";
import { useParticipantDirectory } from "shared/src/data/useParticipantDirectory";

export const Route = createRootRouteWithContext<{
  queryClient: QueryClient;
  api_gateway: string;
}>()({
  component: RootComponent,
  errorComponent: GeneralErrorComponent,
});

/** `undefined` while asking the gateway; `null` when there is no session. */
type SessionState = SessionUser | null | undefined;

function AuthenticatedDirectoryBridge({ children }: { children: React.ReactNode }) {
  const { byId, isSelf } = useParticipantDirectory();
  const resolveParticipant = useCallback(
    (id: string) => {
      const found = byId.get(id);
      return {
        nick: found?.participant_nick,
        isSelf: isSelf(id),
      };
    },
    [byId, isSelf],
  );

  return (
    <ParticipantDirectoryProvider value={resolveParticipant}>
      {children}
    </ParticipantDirectoryProvider>
  );
}

function RootComponent() {
  const [user, setUser] = useState<SessionState>(undefined);

  useEffect(() => {
    loadSession().then(setUser);
    const handleUnauthorized = () => setUser(null);
    window.addEventListener(UNAUTHORIZED_EVENT, handleUnauthorized);
    return () => window.removeEventListener(UNAUTHORIZED_EVENT, handleUnauthorized);
  }, []);

  if (user === undefined) {
    return (
      <div className="flex h-screen w-full items-center justify-center">
        <Loader2 className="h-8 w-8 animate-spin text-brand-sky" />
      </div>
    );
  }

  if (user === null) {
    return <SignInScreen title="Eunomia DS-Agent Admin" />;
  }

  return (
    <AuthenticatedDirectoryBridge>
      <LiveContextProvider>
        <SidebarProvider>
          <AppSidebar />
          <SidebarInset>
            <Header user={user} onSignOut={signOut} />
            <div className="flex flex-1 flex-col gap-4 p-8 items-start justify-start w-full h-full min-w-0">
              <Outlet />
            </div>
          </SidebarInset>
        </SidebarProvider>
        <Toaster />
        <GlobalLoadingIndicator />
        <TanStackRouterDevtools />
      </LiveContextProvider>
    </AuthenticatedDirectoryBridge>
  );
}
