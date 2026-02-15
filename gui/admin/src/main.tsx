import ReactDOM from "react-dom/client";
import { useContext } from "react";
import "shared/index.css";
import { createRouter, RouterProvider } from "@tanstack/react-router";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { routeTree } from "./routeTree.gen";
import { AuthContextProvider } from "shared/src/context/AuthContext.tsx";
import {
  GlobalInfoContextProvider,
  GlobalInfoContext,
} from "shared/src/context/GlobalInfoContext.tsx";
import { SSIAuthContextProvider } from "shared/src/context/SSIAuthContext.tsx";

/**
 * Global QueryClient instance.
 */
export const queryClient = new QueryClient();

import { GeneralErrorComponent } from "./components/GeneralErrorComponent";

// Create a new router instance
const router = createRouter({
  routeTree,
  context: { queryClient, api_gateway: "" },
  basepath: "/admin",
  defaultPreload: "intent",
  defaultErrorComponent: GeneralErrorComponent,
});

const App = () => {
  const globalInfo = useContext(GlobalInfoContext);
  return <RouterProvider router={router} context={{ api_gateway: globalInfo!.api_gateway }} />;
};

ReactDOM.createRoot(document.getElementById("root")!).render(
  <GlobalInfoContextProvider>
    <QueryClientProvider client={queryClient}>
      <AuthContextProvider>
        <SSIAuthContextProvider>
          <App />
        </SSIAuthContextProvider>
      </AuthContextProvider>
    </QueryClientProvider>
  </GlobalInfoContextProvider>,
);
