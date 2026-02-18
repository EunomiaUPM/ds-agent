import { createFileRoute } from "@tanstack/react-router";
import {
  useGetDataServiceById,
  getGetDataServiceByIdQueryOptions,
} from "shared/src/data/orval/data-services/data-services";
import { FormatDate } from "shared/src/components/ui/format-date";
import Heading from "shared/src/components/ui/heading";
import { Badge } from "shared/src/components/ui/badge";
import { InfoList } from "shared/src/components/ui/info-list";
import { formatUrn } from "shared/src/lib/utils.ts";
import { PageLayout } from "shared/src/components/layout/PageLayout";
import { PageHeader } from "shared/src/components/layout/PageHeader";
import { InfoGrid } from "shared/src/components/layout/InfoGrid";
import { PageSection } from "shared/src/components/layout/PageSection";

function RouteComponent() {
  const { dataServiceId } = Route.useParams();
  const { data: dataServiceData } = useGetDataServiceById(dataServiceId);
  const dataService = dataServiceData?.status === 200 ? dataServiceData.data : undefined;

  if (!dataService) return null;
  return (
    <PageLayout>
      <PageHeader
        title="Data service info with id"
        badge={
          <Badge variant="info" size="lg">
            {formatUrn(dataService.id!)}
          </Badge>
        }
      />
      <InfoGrid>
        <PageSection>
          <InfoList
            items={[
              { label: "Data service title", value: dataService.dctTitle },
              {
                label: "Data service creation date",
                value: { type: "custom", content: <FormatDate date={dataService.dctIssued!} /> },
              },
              { label: "Data service endpoint URL", value: dataService.dcatEndpointUrl },
              {
                label: "Data service description",
                value: dataService.dcatEndpointDescription,
              },
            ]}
          />
        </PageSection>
      </InfoGrid>
    </PageLayout>
  );
}

/**
 * Route for displaying data service details.
 */
export const Route = createFileRoute("/catalog/$catalogId/data-service/$dataServiceId")({
  component: RouteComponent,
  pendingComponent: () => <div>Loading...</div>,
  loader: ({ context: { queryClient }, params: { dataServiceId } }) => {
    return queryClient.ensureQueryData(getGetDataServiceByIdQueryOptions(dataServiceId));
  },
});
