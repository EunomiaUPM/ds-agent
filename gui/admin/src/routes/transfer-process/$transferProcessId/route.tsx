import { createFileRoute, Outlet } from "@tanstack/react-router";
import Heading from "../../../../../shared/src/components/ui/heading.tsx";
import { Badge } from "shared/src/components/ui/badge.tsx";
import { getGetTransferProcessByIdQueryOptions } from "shared/src/data/orval/transfers/transfers";
import { formatUrn } from "shared/src/lib/utils.ts";
import { PageHeader } from "shared/src/components/layout/PageHeader";

const NotFound = () => {
  return <div>not found</div>;
};

const RouteComponent = () => {
  const { transferProcessId } = Route.useParams();
  return (
    <div className="w-full mb-2">
      <PageHeader
        className="mb-6"
        title="Transfer Process"
        badge={
          <Badge variant="info" size="lg">
            {formatUrn(transferProcessId)}
          </Badge>
        }
      />
      <Outlet />
    </div>
  );
};

/**
 * Route for specific transfer process details layout.
 */
export const Route = createFileRoute("/transfer-process/$transferProcessId")({
  component: RouteComponent,
  notFoundComponent: NotFound,
  loader: ({ context: { queryClient }, params: { transferProcessId } }) => {
    return queryClient.ensureQueryData(
      getGetTransferProcessByIdQueryOptions(transferProcessId),
    );
  },
});
