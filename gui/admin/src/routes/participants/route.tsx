import { createFileRoute, Outlet } from "@tanstack/react-router";
import { getGetAllParticipantsQueryOptions } from "shared/src/data/orval/participants/participants";

const RouteComponent = () => {
  return <Outlet />;
};

/**
 * Participants route layout and data loading.
 */
export const Route = createFileRoute("/participants")({
  component: RouteComponent,
  loader: ({ context: { queryClient } }) => {
    return queryClient.ensureQueryData(getGetAllParticipantsQueryOptions());
  },
});
