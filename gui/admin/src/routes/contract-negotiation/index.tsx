import { createFileRoute, Link } from "@tanstack/react-router";
import { formatIdentifier } from "shared/src/lib/utils";
import { DataTable } from "shared/src/components/DataTable";
import { FormatDate } from "shared/src/components/ui/format-date";
import { Button } from "shared/src/components/ui/button";
import { Badge } from "shared/src/components/ui/badge";
import { useGetNegotiationProcesses } from "shared/src/data/orval/negotiations/negotiations";
import { ContractNegotiationActions } from "shared/src/components/actions/ContractNegotiationActions";
import { ContractNegotiationBusinessActions } from "shared/src/components/actions/ContractNegotiationBusinessActions";
import { useMemo, useEffect, useState } from "react";
import { ArrowRight } from "lucide-react";
import { PageLayout } from "shared/src/components/layout/PageLayout";
import { PageHeader } from "shared/src/components/layout/PageHeader";
import { PageSection } from "shared/src/components/layout/PageSection";
import { useGetAllParticipants } from "shared/data/orval/participants/participants";
import { Dataset, RpcCatalogResponseMessageDto } from "shared/data/orval/model";
import { useRpcSetupCatalogRequest } from "shared/src/data/orval/catalog-rp-c/catalog-rp-c";
import { keepPreviousData } from "@tanstack/react-query";
import { useTableQueryParams } from "shared/src/hooks/useTableQueryParams";

type ActionsMode = "business" | "standard";

const RouteComponent = () => {
  const { params: queryParams, apiParams, onQueryChange } = useTableQueryParams({
    defaultLimit: 10,
    defaultSort: "created_at_desc",
  });

  const { data: cnProcessesData, isFetching } = useGetNegotiationProcesses({
    query: {
      queryKey: ["/negotiations/negotiation-processes", apiParams],
      placeholderData: keepPreviousData,
    },
    request: {
      params: apiParams,
    },
  });
  const { data: participants } = useGetAllParticipants();
  const [mode, setMode] = useState<ActionsMode>("business");

  //para la notificacion burbuja: "you completed a contract neg. w/ dataset"
  const [_bubbleFeedbackAction, setBubbleFeedbackAction] = useState(false);
  const [requestedDatasetId, setRequestedDatasetId] = useState<string | null>(null);
  const [requestedParticipantId, setRequestedParticipantId] = useState<string | null>(null);

  const { mutate, data, isPending, error } = useRpcSetupCatalogRequest();

  // encontrar el dataset por el id que se le pasa en localstorage
  const currentDatasetNegoc = useMemo(() => {
    if (!data) return null;
    const response = (data?.data as RpcCatalogResponseMessageDto).response!;
    const dataset = response.dataset!.find((d) => d["@id"] === requestedDatasetId);
    return dataset as Dataset;
  }, [data]);

  // encontrar el participant por el id que se le pasa en localstorage
  const currentParticipantNegoc = Array.isArray(participants?.data)
    ? participants?.data.find((p) => p.participant_id === requestedParticipantId)
    : undefined;

  // useEffect(() => {
  //   if (bubbleFeedbackAction && currentDatasetNegoc && currentParticipantNegoc) {
  //     toast("Contract Request Sent", {
  //       description: (
  //         <span>
  //           You sent a contract negotiation request for <b>{currentDatasetNegoc?.title}</b> to participant <b>{currentParticipantNegoc?.participant_nick}</b>.
  //         </span>
  //       ),
  //       position: "top-center",
  //       duration: 8000
  //     });
  //     setBubbleFeedbackAction(false);
  //   }
  // }, [bubbleFeedbackAction, currentDatasetNegoc, currentParticipantNegoc]);

  useEffect(() => {
    if (!requestedParticipantId || !requestedDatasetId) return;
    mutate({
      data: {
        associatedAgentPeer: requestedParticipantId,
        filter: [],
        noCache: true,
      },
    });
  }, [requestedParticipantId, requestedDatasetId, mutate]);

  //obtener de local storage la info de la acción que se acaba de hacer
  useEffect(() => {
    try {
      const justSentContract = sessionStorage.getItem("justSentContract");
      const datasetId = sessionStorage.getItem("datasetId");
      const participantId = sessionStorage.getItem("participantId");
      if (justSentContract === "true") {
        setBubbleFeedbackAction(true);
        setRequestedDatasetId(datasetId);
        setRequestedParticipantId(participantId);
        sessionStorage.removeItem("justSentContract");
      }
    } catch (e) {
      // ignore storage errors
    }
  }, []);

  const cnProcesses = cnProcessesData?.status === 200 ? cnProcessesData.data : undefined;

  return (
    <PageLayout>
      <PageHeader title="Contract Negotiations" className="flex items-center justify-between">
        <div className="flex gap-1 mt-2 p-0.5 rounded-md bg-ink/5 w-fit text-xs">
          <button
            onClick={() => setMode("business")}
            className={`px-3 py-1 rounded transition-colors ${
              mode === "business"
                ? "bg-ink/15 text-ink font-medium"
                : "text-ink/50 hover:text-ink/80"
            }`}
          >
            Business
          </button>
          <button
            onClick={() => setMode("standard")}
            className={`px-3 py-1 rounded transition-colors ${
              mode === "standard"
                ? "bg-ink/15 text-ink font-medium"
                : "text-ink/50 hover:text-ink/80"
            }`}
          >
            Standard
          </button>
        </div>
      </PageHeader>

      <PageSection>
        <DataTable
          className="text-sm"
          data={cnProcesses}
          serverSide={true}
          loading={isFetching}
          queryParams={queryParams}
          onQueryChange={onQueryChange}
          keyExtractor={(p) => p.id}
          searchPlaceholder="Filter negotiations by process ID, peer, or state..."
          filters={[
            {
              id: "state",
              label: "State",
              options: [
                { label: "All States", value: "all" },
                { label: "Requested", value: "REQUESTED" },
                { label: "Offered", value: "OFFERED" },
                { label: "Accepted", value: "ACCEPTED" },
                { label: "Agreed", value: "AGREED" },
                { label: "Verified", value: "VERIFIED" },
                { label: "Finalized", value: "FINALIZED" },
                { label: "Terminated", value: "TERMINATED" },
              ],
              filterFn: (item, val) => item.state?.toUpperCase().includes(val) ?? false,
            },
            {
              id: "role",
              label: "Role",
              accessorKey: "role",
              options: [
                { label: "All Roles", value: "all" },
                { label: "Provider", value: "Provider" },
                { label: "Consumer", value: "Consumer" },
              ],
            },
          ]}
          columns={[
            {
              header: "Process ID",
              accessorKey: "id",
              cell: (p) => <Badge variant="info">{formatIdentifier(p.id)}</Badge>,
            },
            {
              header: "Peer",
              accessorKey: "associatedAgentPeer",
              cell: (p) => (
                <p className="flex gap-2 items-baseline">
                  <span className="capitalize min-w-fit">
                    {formatIdentifier(p.associatedAgentPeer, 3)}
                  </span>
                  <span className="text-ink/70">as</span>
                  <Badge className="h-fit">{p.role === "Provider" ? "Provider" : "Consumer"}</Badge>
                </p>
              ),
            },
            {
              header: "State",
              accessorKey: "state",
              cell: (p) => (
                <Badge variant="status" state={p.state}>
                  {p.state.replace("dspace:", "")}
                </Badge>
              ),
            },
            {
              header: "Created At",
              accessorKey: "createdAt",
              sortKey: "created_at",
              sortValue: (p) => new Date(p.createdAt).getTime(),
              cell: (p) => <FormatDate date={p.createdAt} />,
            },
            {
              header: "Actions",
              sortable: false,
              searchable: false,
              cell: (p) =>
                mode === "business" ? (
                  <ContractNegotiationBusinessActions process={p} tiny={true} />
                ) : (
                  <ContractNegotiationActions process={p} tiny={true} />
                ),
            },
            {
              header: "Link",
              sortable: false,
              searchable: false,
              cell: (p) => (
                <Link to="/contract-negotiation/$cnProcess" params={{ cnProcess: p.id }}>
                  <Button variant="link">
                    See details
                    <ArrowRight />
                  </Button>
                </Link>
              ),
            },
          ]}
        />
      </PageSection>
    </PageLayout>
  );
};

/**
 * Route for listing contract negotiation processes.
 */
export const Route = createFileRoute("/contract-negotiation/")({
  component: RouteComponent,
  pendingComponent: () => <div>Loading...</div>,
});
