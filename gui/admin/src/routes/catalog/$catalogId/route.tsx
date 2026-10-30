import { createFileRoute, Outlet } from "@tanstack/react-router";
import { getGetCatalogByIdQueryOptions } from "shared/src/data/orval/catalogs/catalogs";
import { getGetDatasetsByCatalogIdQueryOptions } from "shared/src/data/orval/datasets/datasets";
import { getGetDataServicesByCatalogIdQueryOptions } from "shared/src/data/orval/data-services/data-services";

const NotFound = () => {
  return <div>not found</div>;
};

/**
 * Route for specific catalog layout.
 */
export const Route = createFileRoute("/catalog/$catalogId")({
  component: RouteComponent,
  notFoundComponent: NotFound,
  loader: async ({ context: { queryClient }, params: { catalogId } }) => {
    await queryClient.ensureQueryData(getGetCatalogByIdQueryOptions(catalogId));
    await queryClient.ensureQueryData(getGetDatasetsByCatalogIdQueryOptions(catalogId));
    return queryClient.ensureQueryData(getGetDataServicesByCatalogIdQueryOptions(catalogId));
  },
});

function RouteComponent() {
  return <Outlet />;
}
