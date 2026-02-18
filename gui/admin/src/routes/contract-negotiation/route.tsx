import { createFileRoute, Outlet, useRouterState } from "@tanstack/react-router";
import Heading from "../../../../shared/src/components/ui/heading.tsx";
import { getGetNegotiationProcessesQueryOptions } from "shared/src/data/orval/negotiations/negotiations";

const NotFound = () => {
  return <div>not found</div>;
};

const RouteComponent = () => {
  const routerState = useRouterState();
  return (
    <>
      {routerState.location.pathname !== "/contract-negotiation" ? null : (
        <div className="mb-6">
          <Heading level="h3" className="flex gap-2 items-center">
            Contract Negotiations
          </Heading>
        </div>
      )}
      <Outlet />
    </>
  );
};

/**
 * Contract Negotiation route layout and data loading.
 */
export const Route = createFileRoute("/contract-negotiation")({
  component: RouteComponent,
  notFoundComponent: NotFound,
  loader: async ({ context: { queryClient } }) => {
    return queryClient.ensureQueryData(getGetNegotiationProcessesQueryOptions());
  },
});
