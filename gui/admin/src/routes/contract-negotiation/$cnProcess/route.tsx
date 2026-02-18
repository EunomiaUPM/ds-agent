import { createFileRoute, Outlet } from "@tanstack/react-router";
import { Badge } from "shared/src/components/ui/badge.tsx";
import {
  getGetNegotiationProcessByIdQueryOptions,
  getGetNegotiationMessagesByProcessIdQueryOptions,
} from "shared/src/data/orval/negotiations/negotiations";
import { formatUrn } from "shared/src/lib/utils.ts";
import { PageHeader } from "shared/src/components/layout/PageHeader";

const NotFound = () => {
  return <div>not found</div>;
};

const RouteComponent = () => {
  const { cnProcess } = Route.useParams();
  return (
    <div>
      <PageHeader
        title="Contract negotiation process"
        badge={
          <Badge variant="info" size="lg">
            {formatUrn(cnProcess)}
          </Badge>
        }
      />
      <Outlet />
    </div>
  );
};

/**
 * Route for specific contract negotiation process layout.
 */
export const Route = createFileRoute("/contract-negotiation/$cnProcess")({
  component: RouteComponent,
  notFoundComponent: NotFound,
  loader: async ({ context: { queryClient }, params: { cnProcess } }) => {
    await queryClient.ensureQueryData(
      getGetNegotiationProcessByIdQueryOptions(cnProcess),
    );
    return queryClient.ensureQueryData(
      getGetNegotiationMessagesByProcessIdQueryOptions(cnProcess),
    );
  },
});
