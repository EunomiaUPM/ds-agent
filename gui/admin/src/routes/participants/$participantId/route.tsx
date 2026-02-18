import { createFileRoute, Outlet } from "@tanstack/react-router";
import { getGetParticipantByIdQueryOptions } from "shared/src/data/orval/participants/participants";

const NotFound = () => {
  return <div>not found</div>;
};

const RouteComponent = () => {
  return (
    <div className="mb-2">
      <header className="mb-2"></header>
      <Outlet />
    </div>
  );
};

/**
 * Route for specific participant details layout.
 */
export const Route = createFileRoute("/participants/$participantId")({
  component: RouteComponent,
  notFoundComponent: NotFound,
  loader: ({ context: { queryClient }, params: { participantId } }) => {
    return queryClient.ensureQueryData(getGetParticipantByIdQueryOptions(participantId));
  },
});
