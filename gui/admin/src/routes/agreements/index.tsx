import { createFileRoute, useNavigate } from "@tanstack/react-router";
import { keepPreviousData } from "@tanstack/react-query";
import { useGetAgreements } from "shared/src/data/orval/negotiations/negotiations";
import { AgreementDto } from "shared/src/data/orval/model";
import { DataTable } from "@eunomia/ui";
import { FormatDate } from "@eunomia/ui";
import { Badge } from "@eunomia/ui";
import { PageLayout } from "@eunomia/ui";
import { PageHeader } from "@eunomia/ui";
import { ParticipantLabel } from "@eunomia/ui";
import { DatasetLabel } from "shared/src/components/identity/DatasetLabel";
import { Identifier } from "@eunomia/ui";
import { useParticipantDirectory } from "shared/src/data/useParticipantDirectory";
import { useDatasetDirectory } from "shared/src/data/useDatasetDirectory";
import { useTableQueryParams } from "@eunomia/ui";

export const Route = createFileRoute("/agreements/")({
  component: RouteComponent,
});

function RouteComponent() {
  const navigate = useNavigate();
  const { self, isSelf } = useParticipantDirectory();
  const { params, apiParams, onQueryChange } = useTableQueryParams({
    defaultLimit: 10,
    defaultSort: "created_at_desc",
  });

  // The role filter maps to the participant column this agent fills.
  const { role, ...rest } = apiParams;
  const queryParams = {
    ...rest,
    ...(role === "Provider" && self && { providerId: self.participant_id }),
    ...(role === "Consumer" && self && { consumerId: self.participant_id }),
  };

  const { data: response, isFetching } = useGetAgreements(queryParams, {
    query: { placeholderData: keepPreviousData },
  });
  const page = response?.status === 200 ? response.data : undefined;
  const datasets = useDatasetDirectory(page?.items.map((a) => a.target) ?? []);

  return (
    <PageLayout>
      <PageHeader
        title="Agreements"
        description="Contract agreements reached through finalized negotiations (DSP 8.1)."
      />
      <DataTable<AgreementDto>
        data={page}
        className="table-fixed [&_td]:overflow-hidden"
        serverSide
        loading={isFetching}
        queryParams={params}
        onQueryChange={onQueryChange}
        keyExtractor={(a) => a.id}
        onRowClick={(a) =>
          navigate({ to: "/agreements/$agreementId", params: { agreementId: a.id } })
        }
        searchable={false}
        emptyMessage="No agreements yet. They appear when a negotiation is finalized."
        filters={[
          {
            id: "role",
            label: "My role",
            options: [
              { label: "Provider", value: "Provider" },
              { label: "Consumer", value: "Consumer" },
            ],
          },
        ]}
        columns={[
          {
            header: "Dataset",
            cell: (a) => <DatasetLabel id={a.target} directory={datasets} />,
          },
          {
            header: "Counterparty",
            cell: (a) => (
              <ParticipantLabel
                id={
                  isSelf(a.providerParticipantId)
                    ? a.consumerParticipantId
                    : a.providerParticipantId
                }
                compact
              />
            ),
          },
          {
            header: "My role",
            className: "w-28",
            cell: (a) => {
              const r = isSelf(a.providerParticipantId) ? "Provider" : "Consumer";
              return (
                <Badge variant="role" dsrole={r} size="sm">
                  {r}
                </Badge>
              );
            },
          },
          {
            header: "State",
            className: "w-32",
            cell: (a) => (
              <Badge variant="status" state={a.state} size="sm">
                {a.state}
              </Badge>
            ),
          },
          {
            header: "Agreement",
            cell: (a) => <Identifier value={a.id} />,
          },
          {
            header: "Agreed",
            className: "w-36 whitespace-nowrap",
            accessorKey: "createdAt",
            sortKey: "created_at",
            cell: (a) => <FormatDate date={a.createdAt} />,
          },
        ]}
      />
    </PageLayout>
  );
}
