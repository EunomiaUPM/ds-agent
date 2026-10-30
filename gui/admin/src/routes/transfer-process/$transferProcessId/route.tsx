import { createFileRoute, Outlet } from "@tanstack/react-router";
import { getGetTransferProcessByIdQueryOptions } from "shared/src/data/orval/transfers/transfers";
import { GeneralErrorComponent } from "@eunomia/ui";

/** Loads the transfer process; each child page draws its own header. */
export const Route = createFileRoute("/transfer-process/$transferProcessId")({
  component: Outlet,
  notFoundComponent: () => (
    <GeneralErrorComponent error={new Error("Transfer process not found")} reset={() => {}} />
  ),
  loader: ({ context: { queryClient }, params: { transferProcessId } }) =>
    queryClient.ensureQueryData(getGetTransferProcessByIdQueryOptions(transferProcessId)),
});
