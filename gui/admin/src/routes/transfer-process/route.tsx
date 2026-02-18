import { createFileRoute, Outlet, useRouterState } from "@tanstack/react-router";
import Heading from "../../../../shared/src/components/ui/heading.tsx";
import { getGetTransferProcessesQueryOptions } from "shared/src/data/orval/transfers/transfers";

const NotFound = () => {
  return <div>not found</div>;
};

const RouteComponent = () => {
  const routerState = useRouterState();
  return (
    <>
      {routerState.location.pathname !== "/transfer-process" ? null : (
        <>
          <div className="mb-6">
            <Heading level="h3" className="flex gap-2 items-center">
              Transfer Processes
            </Heading>
          </div>
        </>
      )}
      <Outlet />
    </>
  );
};

/**
 * Transfer Process route layout and data loading.
 */
export const Route = createFileRoute("/transfer-process")({
  component: RouteComponent,
  notFoundComponent: NotFound,
  loader: ({ context: { queryClient } }) => {
    return queryClient.ensureQueryData(getGetTransferProcessesQueryOptions());
  },
});
