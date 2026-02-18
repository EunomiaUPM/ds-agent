import { createFileRoute, Outlet, useRouterState } from "@tanstack/react-router";
import Heading from "shared/src/components/ui/heading.tsx";
import { getGetAgreementsQueryOptions } from "shared/src/data/orval/negotiations/negotiations";

const NotFound = () => {
  return <div>not found</div>;
};

/**
 * Agreements route component.
 */
const RouteComponent = () => {
  const routerState = useRouterState();
  return (
    <>
      {routerState.location.pathname !== "/agreements" ? null : (
        <>
          <div className="mb-6">
            <Heading level="h3" className="flex gap-2 items-center">
              Agreements
            </Heading>
          </div>
        </>
      )}
      <Outlet />
    </>
  );
};

/**
 * Agreements route layout.
 */
export const Route = createFileRoute("/agreements")({
  component: RouteComponent,
  notFoundComponent: NotFound,
  loader: ({ context: { queryClient } }) => {
    return queryClient.ensureQueryData(getGetAgreementsQueryOptions());
  },
});
