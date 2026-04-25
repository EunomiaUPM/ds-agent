import { createFileRoute, Link } from "@tanstack/react-router";
import { useQuery } from "@tanstack/react-query";
import { customInstance } from "shared/src/data/orval-mutator";
import { PageLayout } from "shared/src/components/layout/PageLayout";
import { PageHeader } from "shared/src/components/layout/PageHeader";
import { PageSection } from "shared/src/components/layout/PageSection";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "shared/src/components/ui/card";
import { Badge } from "shared/src/components/ui/badge";
import { Button } from "shared/src/components/ui/button";
import { ArrowLeft, ExternalLink, Calendar, Shield, Hash, Key, Eye, EyeOff } from "lucide-react";
import { useState } from "react";
import { FormatDate } from "shared/src/components/ui/format-date";
import * as z from "zod";
import { OnboardRequest } from "./index";

const searchSchema = z.object({
  requestId: z.string(),
});

// @ts-ignore
export const Route = createFileRoute("/providers/request-details")({
  validateSearch: (search) => searchSchema.parse(search),
  component: ProviderRequestDetails,
});

function ProviderRequestDetails() {
  const { requestId } = Route.useSearch();

  const { data: response, isLoading } = useQuery({
    queryKey: ["onboard-request", requestId],
    queryFn: () => 
      customInstance<{ status: number; data: OnboardRequest }>(`/onboard/request/${requestId}`, { method: "GET" }),
    enabled: !!requestId,
  });

  const request = response?.data;
  const [showSecrets, setShowSecrets] = useState(false);

  const getStatusColor = (status: string) => {
    switch (status.toLowerCase()) {
      case 'processing': return 'bg-blue-500/10 text-blue-500 border-blue-500/20';
      case 'pending': return 'bg-amber-500/10 text-amber-500 border-amber-500/20';
      case 'approved': return 'bg-green-500/10 text-green-500 border-green-500/20';
      case 'finalized': return 'bg-purple-500/10 text-purple-500 border-purple-500/20';
      default: return 'bg-gray-500/10 text-gray-500 border-gray-500/20';
    }
  };

  if (isLoading) {
    return (
      <PageLayout>
        <PageHeader title="Loading request..." />
        <PageSection>
          <div className="flex items-center justify-center h-64">
            <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
          </div>
        </PageSection>
      </PageLayout>
    );
  }

  if (!request) {
    return (
      <PageLayout>
        <PageHeader title="Request not found" />
        <PageSection>
          <div className="text-center py-12">
            <p className="text-muted-foreground mb-4">The onboarding request with ID {requestId} could not be found.</p>
            <Link to="/providers">
              <Button variant="outline">
                <ArrowLeft className="mr-2 h-4 w-4" />
                Back to Providers
              </Button>
            </Link>
          </div>
        </PageSection>
      </PageLayout>
    );
  }

  return (
    <PageLayout>
      <PageHeader title={`Request: ${request.provider_slug || request.id}`}>
        <div className="flex gap-2 mb-4">
          <Link to="/providers">
            <Button variant="ghost" size="sm">
              <ArrowLeft className="h-4 w-4 mr-2" />
              Back
            </Button>
          </Link>
          <Badge className={`border ${getStatusColor(request.status)}`}>
            {request.status}
          </Badge>
        </div>
      </PageHeader>

      <PageSection>
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
          <div className="md:col-span-2 space-y-6">
            <Card>
              <CardHeader>
                <div className="flex justify-between items-start">
                  <div>
                    <CardTitle>Provider Information</CardTitle>
                    <CardDescription>Onboarding details and endpoint configuration.</CardDescription>
                  </div>
                  <Shield className="h-5 w-5 text-muted-foreground" />
                </div>
              </CardHeader>
              <CardContent className="space-y-6">
                <div className="grid grid-cols-1 sm:grid-cols-2 gap-6">
                  <div className="space-y-1">
                    <p className="text-xs font-medium text-muted-foreground uppercase flex items-center gap-2">
                      <Hash className="h-3 w-3" /> Request ID
                    </p>
                    <p className="text-sm font-mono bg-muted/50 p-2 rounded truncate">{request.id}</p>
                  </div>
                  <div className="space-y-1">
                    <p className="text-xs font-medium text-muted-foreground uppercase flex items-center gap-2">
                      <Hash className="h-3 w-3" /> Provider DID
                    </p>
                    <p className="text-sm font-mono bg-muted/50 p-2 rounded truncate">{request.provider_id}</p>
                  </div>
                </div>

                <div className="space-y-1">
                  <p className="text-xs font-medium text-muted-foreground uppercase flex items-center gap-2">
                    <ExternalLink className="h-3 w-3" /> Grant Endpoint
                  </p>
                  <p className="text-sm font-mono break-all">{request.grant_endpoint}</p>
                </div>

                <div className="grid grid-cols-1 sm:grid-cols-2 gap-6">
                  <div className="space-y-1">
                    <p className="text-xs font-medium text-muted-foreground uppercase">Auto Authentication</p>
                    <Badge variant={request.auto ? "info" : "infoLighter"}>
                      {request.auto ? "Enabled" : "Disabled"}
                    </Badge>
                  </div>
                  <div className="space-y-1">
                    <p className="text-xs font-medium text-muted-foreground uppercase">Friendly Name</p>
                    <p className="text-sm">{request.provider_slug || "-"}</p>
                  </div>
                </div>
              </CardContent>
            </Card>

            {(request.assigned_id || request.token) && (
              <Card className="border-stroke bg-background shadow-sm">
                <CardHeader>
                  <div className="flex justify-between items-center">
                    <div>
                      <CardTitle className="flex items-center gap-2">
                        <Key className="h-5 w-5 text-brand-sky" />
                        Response Data
                      </CardTitle>
                      <CardDescription>Credentials received from the provider.</CardDescription>
                    </div>
                    <Button variant="outline" size="sm" onClick={() => setShowSecrets(!showSecrets)}>
                      {showSecrets ? <><EyeOff className="h-4 w-4 mr-2"/> Hide</> : <><Eye className="h-4 w-4 mr-2"/> Show</>}
                    </Button>
                  </div>
                </CardHeader>
                <CardContent className="space-y-6">
                  {request.assigned_id && (
                    <div className="space-y-1">
                      <p className="text-xs font-medium text-muted-foreground uppercase">Assigned ID</p>
                      <p className="text-sm font-mono bg-muted/50 p-2 rounded border border-stroke">
                        {showSecrets ? request.assigned_id : "••••••••••••••••••••••••••••••••"}
                      </p>
                    </div>
                  )}
                  {request.token && (
                    <div className="space-y-1">
                      <p className="text-xs font-medium text-muted-foreground uppercase">Access Token</p>
                      <div className="relative">
                        <p className={`text-sm font-mono bg-muted/50 p-3 rounded border border-stroke break-all ${showSecrets ? 'select-all' : 'select-none'}`}>
                          {showSecrets ? request.token : "••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••"}
                        </p>
                      </div>
                    </div>
                  )}
                </CardContent>
              </Card>
            )}
          </div>

          <div className="space-y-6">
            <Card>
              <CardHeader>
                <CardTitle>Timeline</CardTitle>
              </CardHeader>
              <CardContent className="space-y-6">
                <div className="relative pl-6 border-l-2 border-muted space-y-8">
                  <div className="relative">
                    <span className="absolute -left-[31px] top-1 h-4 w-4 rounded-full bg-primary border-4 border-background" />
                    <div className="space-y-1">
                      <p className="text-sm font-medium">Request Created</p>
                      <p className="text-xs text-muted-foreground flex items-center gap-1">
                        <Calendar className="h-3 w-3" />
                        <FormatDate date={request.created_at} />
                      </p>
                    </div>
                  </div>

                  <div className="relative">
                    <span className={`absolute -left-[31px] top-1 h-4 w-4 rounded-full border-4 border-background ${request.ended_at ? 'bg-primary' : 'bg-muted'}`} />
                    <div className="space-y-1">
                      <p className="text-sm font-medium">Request Processed</p>
                      <p className="text-xs text-muted-foreground flex items-center gap-1">
                        <Calendar className="h-3 w-3" />
                        {request.ended_at ? <FormatDate date={request.ended_at} /> : "In progress..."}
                      </p>
                    </div>
                  </div>
                </div>

                <div className="pt-4 border-t space-y-4">
                  <div className="flex justify-between items-center text-sm">
                    <span className="text-muted-foreground">Current Status:</span>
                    <Badge className={`border ${getStatusColor(request.status)}`}>{request.status}</Badge>
                  </div>
                </div>
              </CardContent>
            </Card>
          </div>
        </div>
      </PageSection>
    </PageLayout>
  );
}
