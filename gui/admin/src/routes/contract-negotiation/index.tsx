import { createFileRoute, Link } from "@tanstack/react-router";
import { formatIdentifier } from "shared/src/lib/utils";
import { DataTable } from "shared/src/components/DataTable";
import { FormatDate } from "shared/src/components/ui/format-date";
import { Button } from "shared/src/components/ui/button.tsx";
import { toast } from "sonner";
import { Badge, BadgeState } from "shared/src/components/ui/badge.tsx";
import { Input } from "shared/src/components/ui/input.tsx";
import { useGetNegotiationProcesses } from "shared/src/data/orval/negotiations/negotiations";
import { ContractNegotiationActions } from "shared/src/components/actions/ContractNegotiationActions";
import { useMemo, useEffect, useState } from "react";
import { ArrowRight } from "lucide-react";
import { PageLayout } from "shared/src/components/layout/PageLayout";
import { PageHeader } from "shared/src/components/layout/PageHeader";
import { PageSection } from "shared/src/components/layout/PageSection";
import { useGetAllParticipants } from "shared/data/orval/participants/participants";
import { Dataset, ParticipantDto, RpcCatalogResponseMessageDto } from "shared/data/orval/model";
import {
  useRpcSetupCatalogRequest,
} from "shared/src/data/orval/catalog-rp-c/catalog-rp-c";

const RouteComponent = () => {
  const { data: cnProcessesData } = useGetNegotiationProcesses();
  const { data: participants } = useGetAllParticipants();


  //para la notificacion burbuja: "you completed a contract neg. w/ dataset"
  const [bubbleFeedbackAction, setBubbleFeedbackAction] = useState(false)
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
  const currentParticipantNegoc = Array.isArray(participants?.data) ? participants?.data.find((p) =>
    p.participant_id === requestedParticipantId) : undefined;

  useEffect(() => {
    if (bubbleFeedbackAction && currentDatasetNegoc && currentParticipantNegoc) {
      toast("Contract Request Sent", {
        description: (
          <span>
            You sent a contract negotiation request for <b>{currentDatasetNegoc?.title}</b> to participant <b>{currentParticipantNegoc?.participant_slug}</b>.
          </span>
        ),
        position: "top-center",
        duration: 8000
      });
      setBubbleFeedbackAction(false);
    }
  }, [bubbleFeedbackAction, currentDatasetNegoc, currentParticipantNegoc]);

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


  console.log(currentParticipantNegoc?.participant_slug, " currentParticipantNegoc?")
  console.log(currentDatasetNegoc, " currentDatasetNegoc?")

  //obtener de local storage la info de la acción que se acaba de hacer
  useEffect(() => {
    try {
      const justSentContract = sessionStorage.getItem("justSentContract")
      const datasetId = sessionStorage.getItem("datasetId")
      const participantId = sessionStorage.getItem("participantId")
      if (justSentContract === "true") {
        setBubbleFeedbackAction(true)
        setRequestedDatasetId(datasetId);
        setRequestedParticipantId(participantId);
        sessionStorage.removeItem("justSentContract");
      }
    }

    catch (e) {
      // ignore storage errors
    }
  }, [])



  const cnProcesses = cnProcessesData?.status === 200 ? cnProcessesData.data : [];
  const cnProcessesSorted = useMemo(() => {
    if (!cnProcesses) return [];
    return [...cnProcesses].sort((a, b) => {
      // @ts-ignore
      return new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime();
    });
  }, [cnProcesses]);


  return (
    <PageLayout>
      <PageHeader title="Contract Negotiations" />

      <PageSection>
        <DataTable
          className="text-sm"
          data={cnProcessesSorted ?? []}
          keyExtractor={(p) => p.id}
          columns={[
            {
              header: "Process id",
              cell: (p) => <Badge variant={"info"}>{formatIdentifier(p.id)}</Badge>,
            },
            // {
            //   header: "Your Role",
            //   cell: (p) => <Badge variant={"info"}>{p.role}</Badge>,
            // },
            {
              header: "Peer",
              cell: (p) => (
                <p className="flex gap-2 items-baseline">
                  <span className="capitalize min-w-fit">
                    {formatIdentifier(p.associatedAgentPeer, 3)}
                  </span>
                  <span className="text-white/70">as</span>
                  <Badge className="h-fit">{p.role == "provider" ? "Provider" : "Consumer"}</Badge>
                </p>
              ),
            },
            {
              header: "State",
              cell: (p) => (
                <Badge variant={"status"} state={p.state}>
                  {p.state.replace("dspace:", "")}
                </Badge>
              ),
            },
            {
              header: "Created At",
              cell: (p) => <FormatDate date={p.createdAt} />,
            },
            {
              header: "Actions",
              cell: (p) => <ContractNegotiationActions process={p} tiny={true} />,
            },
            {
              header: "Link",
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
