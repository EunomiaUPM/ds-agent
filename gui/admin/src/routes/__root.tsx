import { QueryClient } from "@tanstack/react-query";
import { createRootRouteWithContext, Outlet } from "@tanstack/react-router";
import { TanStackRouterDevtools } from "@tanstack/router-devtools";
import { SidebarInset, SidebarProvider } from "../../../shared/src/components/ui/sidebar";
import React from "react";
import { AppSidebar } from "shared/src/components/AppSidebar.tsx";
import { Header } from "shared/src/components/header.tsx";

import { GeneralErrorComponent } from "../components/GeneralErrorComponent";

import { GlobalLoadingIndicator } from "../components/GlobalLoadingIndicator";

/**
 * Root route component providing application shell, sidebar, and global providers.
 */
export const Route = createRootRouteWithContext<{
  queryClient: QueryClient;
  api_gateway: string;
}>()({
  component: () => {
    return (
      <>
        <SidebarProvider>
          <AppSidebar />
          <SidebarInset>
            <Header />
            <div className="flex flex-1 flex-col gap-4 p-8 items-start justify-start w-full h-full min-w-0">
              <Outlet />
            </div>
          </SidebarInset>
        </SidebarProvider>
        <GlobalLoadingIndicator />
        <TanStackRouterDevtools />
      </>
    );
  },
  errorComponent: GeneralErrorComponent,
});
