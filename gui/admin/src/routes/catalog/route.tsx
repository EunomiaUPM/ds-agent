import { createFileRoute, Outlet } from "@tanstack/react-router";
import {
  getGetMainCatalogsQueryOptions,
  getGetCatalogsQueryOptions
} from "shared/src/data/orval/catalogs/catalogs";

const NotFound = () => {
  return <div>not found</div>;
};

const RouteComponent = () => {
  return (
    <>
      <Outlet />
    </>
  );
};

/**
 * Catalog route layout and data loading.
 */
export const Route = createFileRoute("/catalog")({
  component: RouteComponent,
  notFoundComponent: NotFound,
  loader: async ({ context: { queryClient } }) => {
    await queryClient.ensureQueryData(
      getGetMainCatalogsQueryOptions()
    );

    return queryClient.ensureQueryData(
      getGetCatalogsQueryOptions()
    );
  },
});
