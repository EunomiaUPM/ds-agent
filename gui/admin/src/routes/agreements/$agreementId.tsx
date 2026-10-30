import { createFileRoute, Link, useNavigate } from "@tanstack/react-router";
import { ArrowUpRight } from "lucide-react";
import {
  getGetAgreementByIdSQueryOptions,
  useGetAgreementByIdS,
} from "shared/src/data/orval/negotiations/negotiations";
import { useGetTransferProcesses } from "shared/src/data/orval/transfers/transfers";
import { TransferProcessDto } from "shared/src/data/orval/model";
import { PageLayout } from "@eunomia/ui";
import { PageHeader } from "@eunomia/ui";
import { PageSection } from "@eunomia/ui";
import { FieldGrid } from "@eunomia/ui";
import { Badge } from "@eunomia/ui";
import { FormatDate } from "@eunomia/ui";
import { DataTable } from "@eunomia/ui";
import { ParticipantLabel } from "@eunomia/ui";
import { DatasetLabel } from "shared/src/components/identity/DatasetLabel";
import { Identifier } from "@eunomia/ui";
import { OdrlRulesView } from "shared/src/components/policy/OdrlRulesView";
import { JsonViewer } from "@eunomia/ui";
import { AgreementActions } from "shared/src/components/actions/AgreementActions";
import { useParticipantDirectory } from "shared/src/data/useParticipantDirectory";
import {
  Accordion,
  AccordionContent,
  AccordionItem,
  AccordionTrigger,
} from "@eunomia/ui";

export const Route = createFileRoute("/agreements/$agreementId")({
  component: RouteComponent,
  loader: ({ context: { queryClient }, params: { agreementId } }) =>
    queryClient.ensureQueryData(getGetAgreementByIdSQueryOptions(agreementId)),
});

function RouteComponent() {
  const { agreementId } = Route.useParams();
  const navigate = useNavigate();
  const { isSelf } = useParticipantDirectory();
  const { data: response } = useGetAgreementByIdS(agreementId);
  const agreement = response?.status === 200 ? response.data : undefined;
  const { data: transfersResponse } = useGetTransferProcesses(
    { agreement_id: agreementId, limit: 50, sort: "created_at_desc" },
    { query: { enabled: !!agreement } },
  );
  const transfers = transfersResponse?.status === 200 ? transfersResponse.data : undefined;

  if (!agreement) return null;
  const myRole = isSelf(agreement.providerParticipantId) ? "Provider" : "Consumer";

  return (
    <PageLayout>
      <PageHeader
        eyebrow="Agreement"
        title={<DatasetLabel id={agreement.target} compact />}
        badge={
          <>
            <Badge variant="status" state={agreement.state}>
              {agreement.state}
            </Badge>
            <Badge variant="role" dsrole={myRole}>
              {myRole}
            </Badge>
          </>
        }
        description={
          <>
            Agreed <FormatDate date={agreement.createdAt} />
          </>
        }
        actions={<AgreementActions agreement={agreement} />}
      />

      <PageSection title="Summary">
        <FieldGrid
          columns={3}
          fields={[
            { label: "Provider", value: <ParticipantLabel id={agreement.providerParticipantId} /> },
            { label: "Consumer", value: <ParticipantLabel id={agreement.consumerParticipantId} /> },
            { label: "Dataset", value: <DatasetLabel id={agreement.target} /> },
            { label: "Agreement", value: <Identifier value={agreement.id} tail={36} /> },
            {
              label: "Negotiation",
              value: (
                <Link
                  to="/contract-negotiation/$cnProcess"
                  params={{ cnProcess: agreement.negotiationAgentProcessId }}
                  className="inline-flex items-center gap-1 text-brand-sky hover:underline"
                >
                  <Identifier value={agreement.negotiationAgentProcessId} copyable={false} />
                  <ArrowUpRight className="h-3.5 w-3.5" />
                </Link>
              ),
            },
            {
              label: "Agreement message",
              value: <Identifier value={agreement.negotiationAgentMessageId} />,
            },
          ]}
        />
      </PageSection>

      <PageSection title="Policy">
        <OdrlRulesView policy={agreement.agreementContent} />
        <Accordion type="multiple" className="mt-3">
          <AccordionItem value="odrl" className="border-none">
            <AccordionTrigger className="py-2 text-sm text-ink/60 hover:no-underline">
              ODRL agreement document
            </AccordionTrigger>
            <AccordionContent>
              <JsonViewer value={agreement.agreementContent} />
            </AccordionContent>
          </AccordionItem>
        </Accordion>
      </PageSection>

      <PageSection title="Transfers under this agreement">
        <DataTable<TransferProcessDto>
          data={transfers}
          keyExtractor={(t) => t.id}
          hideToolbar
          onRowClick={(t) =>
            navigate({
              to: "/transfer-process/$transferProcessId",
              params: { transferProcessId: t.id },
            })
          }
          emptyMessage="No transfers have been requested under this agreement."
          columns={[
            { header: "Transfer", cell: (t) => <Identifier value={t.id} /> },
            {
              header: "State",
              cell: (t) => (
                <Badge variant="status" state={t.state} size="sm">
                  {t.state}
                </Badge>
              ),
            },
            {
              header: "Protocol",
              cell: (t) => (
                <span className="text-sm">{t.protocol === "dsp2025_1" ? "DSP 2025-1" : t.protocol}</span>
              ),
            },
            {
              header: "Created",
              accessorKey: "createdAt",
              sortKey: "created_at",
              cell: (t) => <FormatDate date={t.createdAt} />,
            },
          ]}
        />
      </PageSection>
    </PageLayout>
  );
}
