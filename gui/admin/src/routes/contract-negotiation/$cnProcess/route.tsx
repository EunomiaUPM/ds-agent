import { createFileRoute, Outlet } from "@tanstack/react-router";
import {
  getGetNegotiationProcessByIdQueryOptions,
  getGetNegotiationMessagesByProcessIdQueryOptions,
} from "shared/src/data/orval/negotiations/negotiations";
import { GeneralErrorComponent } from "@eunomia/ui";

/** Loads the negotiation and its messages; each child page draws its own header. */
export const Route = createFileRoute("/contract-negotiation/$cnProcess")({
  component: Outlet,
  notFoundComponent: () => (
    <GeneralErrorComponent error={new Error("Contract negotiation not found")} reset={() => {}} />
  ),
  loader: async ({ context: { queryClient }, params: { cnProcess } }) => {
    await queryClient.ensureQueryData(getGetNegotiationProcessByIdQueryOptions(cnProcess));
    return queryClient.ensureQueryData(getGetNegotiationMessagesByProcessIdQueryOptions(cnProcess));
  },
});
