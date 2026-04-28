import { createFileRoute } from "@tanstack/react-router";
import { useGetAllVCRequests } from "shared/src/data/orval/vc-request/vc-request";
import { PageLayout } from "shared/src/components/layout/PageLayout";
import { PageHeader } from "shared/src/components/layout/PageHeader";
import { PageSection } from "shared/src/components/layout/PageSection";
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  CardDescription,
} from "shared/src/components/ui/card";
import { Badge } from "shared/src/components/ui/badge";
import { FormatDate } from "shared/src/components/ui/format-date";
import { formatUrn } from "shared/src/lib/utils";
import { Shield, Clock, CheckCircle2, AlertCircle, Info, ExternalLink } from "lucide-react";
import { z } from "zod";

const searchSchema = z.object({
  requestId: z.string().optional(),
});

/**
 * Route for viewing details of a specific VC request.
 * Path: /authority/request-details
 */
// @ts-ignore
export const Route = createFileRoute("/authority/request-details")({
  validateSearch: (search) => searchSchema.parse(search),
  component: RequestDetailsPage,
});

function RequestDetailsPage() {
  const { requestId } = Route.useSearch();
  const { data: response, isLoading } = useGetAllVCRequests();

  const requests = response?.status === 200 ? response.data : [];
  const request = requests.find((r) => r.id === requestId);

  if (isLoading) {
    return (
      <PageLayout>
        <PageHeader title="Loading request..." />
        <PageSection>
          <div className="flex justify-center py-20">
            <Badge variant="infoLighter" className="animate-pulse">
              Loading details...
            </Badge>
          </div>
        </PageSection>
      </PageLayout>
    );
  }

  if (!request) {
    return (
      <PageLayout>
        <PageHeader title="Request Not Found" />
        <PageSection>
          <div className="flex flex-col items-center justify-center py-20 text-muted-foreground">
            <AlertCircle className="h-12 w-12 mb-4 opacity-20" />
            <p>We couldn't find a VC request with the ID: {requestId}</p>
          </div>
        </PageSection>
      </PageLayout>
    );
  }

  return (
    <PageLayout>
      <PageHeader title={`Request: ${request.authority_slug || "Authority"}`}>
        <div className="text-xs text-muted-foreground font-mono mt-1">ID: {request.id}</div>
      </PageHeader>
      <PageSection>
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
          <Card className="md:col-span-2">
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Info className="h-5 w-5 text-primary" />
                Connection Details
              </CardTitle>
              <CardDescription>
                Detailed information about the VC request and the authority.
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-6">
              <div className="grid grid-cols-1 sm:grid-cols-2 gap-y-6 gap-x-4">
                <DetailItem label="Status">
                  <Badge variant="status" state={request.status}>
                    {request.status}
                  </Badge>
                </DetailItem>
                <DetailItem label="VC Type">
                  <Badge variant="info">{request.vc_type}</Badge>
                </DetailItem>
                <DetailItem label="Authority DID">
                  <span className="font-mono text-xs break-all">
                    {formatUrn(request.authority_id || "")}
                  </span>
                </DetailItem>
                <DetailItem label="Authority Slug">{request.authority_slug}</DetailItem>
                <DetailItem label="Created At">
                  {request.created_at ? <FormatDate date={request.created_at} /> : "-"}
                </DetailItem>
                <DetailItem label="Ended At">
                  {request.ended_at ? <FormatDate date={request.ended_at} /> : "-"}
                </DetailItem>
                <DetailItem label="Assigned ID">
                  <span className="font-mono text-xs">{request.assigned_id || "-"}</span>
                </DetailItem>
                <DetailItem label="Grant Endpoint">
                  <div className="flex items-center gap-1 text-xs text-muted-foreground break-all">
                    {request.grant_endpoint || "-"}
                    {request.grant_endpoint && <ExternalLink className="h-3 w-3" />}
                  </div>
                </DetailItem>
              </div>

              {request.vc_uri && (
                <div className="pt-4 border-t">
                  <DetailItem label="VC URI (QR Link)">
                    <div className="mt-2 p-3 bg-muted rounded-md break-all font-mono text-[10px]">
                      {request.vc_uri}
                    </div>
                  </DetailItem>
                </div>
              )}
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle>Request Timeline</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="space-y-6">
                <TimelineItem
                  icon={<Clock className="h-4 w-4" />}
                  title="Request Created"
                  date={request.created_at}
                  active={true}
                />
                <TimelineItem
                  icon={<Shield className="h-4 w-4" />}
                  title="Processing"
                  description="Authority is verifying credentials"
                  active={request.status !== "Pending"}
                />
                <TimelineItem
                  icon={<CheckCircle2 className="h-4 w-4" />}
                  title="Approved"
                  description="VC is ready to be claimed"
                  active={request.status === "Approved" || request.status === "Finalized"}
                  date={request.status === "Approved" ? request.ended_at : undefined}
                />
              </div>
            </CardContent>
          </Card>
        </div>
      </PageSection>
    </PageLayout>
  );
}

function DetailItem({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div className="flex flex-col gap-1.5">
      <span className="text-xs font-semibold text-muted-foreground uppercase tracking-wider">
        {label}
      </span>
      <div className="text-sm font-medium">{children}</div>
    </div>
  );
}

function TimelineItem({
  icon,
  title,
  description,
  date,
  active,
}: {
  icon: React.ReactNode;
  title: string;
  description?: string;
  date?: string | null;
  active: boolean;
}) {
  return (
    <div className={`flex gap-3 ${active ? "opacity-100" : "opacity-40"}`}>
      <div className="flex flex-col items-center">
        <div
          className={`p-2 rounded-full ${active ? "bg-primary/20 text-primary" : "bg-muted text-muted-foreground"}`}
        >
          {icon}
        </div>
        <div className="w-[2px] flex-1 bg-muted my-1 last:hidden" />
      </div>
      <div className="pb-6">
        <p className="text-sm font-bold">{title}</p>
        {description && <p className="text-xs text-muted-foreground">{description}</p>}
        {date && (
          <p className="text-[10px] mt-1 text-muted-foreground/60 font-mono">
            {new Date(date).toLocaleString()}
          </p>
        )}
      </div>
    </div>
  );
}
