import { createFileRoute, Outlet, useRouterState } from "@tanstack/react-router";
import Heading from "shared/src/components/ui/heading";

const NotFound = () => {
  return <div>not found</div>;
};

const RouteComponent = () => {
  const routerState = useRouterState();
  return (
    <>
      {routerState.location.pathname !== "/ssi-auth" ? null : (
        <>
          <div className="mb-6">
            <Heading level="h3" className="flex gap-2 items-center">
              SSI Auth
            </Heading>
          </div>
        </>
      )}
      <Outlet />
    </>
  );
};

export const Route = createFileRoute("/ssi-auth")({
  component: RouteComponent,
  notFoundComponent: NotFound,
});
