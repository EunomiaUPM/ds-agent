import {
    AlertCircle, Check, CheckCircle2, Clock, Copy, ExternalLink, Info, Key, Loader2, Shield
} from 'lucide-react';
import { useState } from 'react';
import QRCode from 'react-qr-code';
import { PageHeader } from 'shared/src/components/layout/PageHeader';
import { PageLayout } from 'shared/src/components/layout/PageLayout';
import { PageSection } from 'shared/src/components/layout/PageSection';
import { Badge } from 'shared/src/components/ui/badge';
import { Button } from 'shared/src/components/ui/button';
import {
    Card, CardContent, CardDescription, CardHeader, CardTitle
} from 'shared/src/components/ui/card';
import { FormatDate } from 'shared/src/components/ui/format-date';
import { customInstance } from 'shared/src/data/orval-mutator';
import {
    getGetAllVCRequestsQueryKey, useGetAllVCRequests
} from 'shared/src/data/orval/vc-request/vc-request';
import { formatUrn } from 'shared/src/lib/utils';
import { z } from 'zod';

import { useQueryClient } from '@tanstack/react-query';
import { createFileRoute } from '@tanstack/react-router';

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
  const [isProcessing, setIsProcessing] = useState(false);
  const queryClient = useQueryClient();
  
  const requests = response?.status === 200 ? response.data : [];
  const request = requests.find((r) => r.id === requestId);

  const getStatusColor = (status: string) => {
    switch (status.toLowerCase()) {
      case 'processing': return 'bg-blue-500/10 text-blue-500 border-blue-500/20';
      case 'pending': return 'bg-amber-500/10 text-amber-500 border-amber-500/20';
      case 'approved': return 'bg-green-500/10 text-green-500 border-green-500/20';
      case 'finalized': return 'bg-purple-500/10 text-purple-500 border-purple-500/20';
      case 'rejected': return 'bg-red-500/10 text-red-500 border-red-500/20';
      default: return 'bg-gray-500/10 text-gray-500 border-gray-500/20';
    }
  };

  const getTimelineData = (req: any) => {
    const status = req.status?.toLowerCase() || '';
    const pastEvents: { id: string; title: string; date?: string | null }[] = [
      { id: 'created', title: 'Request Created', date: req.created_at }
    ];

    if (status === 'pending') {
      pastEvents.push({ id: 'processing', title: 'Processing' });
    } else if (status === 'rejected') {
      pastEvents.push({ id: 'processing', title: 'Processing' });
      pastEvents.push({ id: 'pending', title: 'Pending' });
    } else if (status === 'approved') {
      pastEvents.push({ id: 'processing', title: 'Processing' });
      pastEvents.push({ id: 'pending', title: 'Pending' });
    } else if (status === 'finalized') {
      pastEvents.push({ id: 'processing', title: 'Processing' });
      pastEvents.push({ id: 'pending', title: 'Pending' });
      pastEvents.push({ id: 'approved', title: 'Approved' });
    }

    let instruction = "";
    switch (status) {
      case 'processing':
        instruction = "The Authorization Server (AS) has not yet evaluated the request.";
        break;
      case 'pending':
        if (req.verification_uri) {
          instruction = "Waiting for your authentication. Please scan the QR code to authenticate with the authority.";
        } else {
          instruction = "The Authorization Server (AS) is currently evaluating the request.";
        }
        break;
      case 'rejected':
        instruction = "Your request has been rejected. No further action can be taken.";
        break;
      case 'approved':
        instruction = "The request has been approved. You can now claim your Verifiable Credential.";
        break;
      case 'finalized':
        instruction = "You have successfully claimed the Verifiable Credential. The process is complete.";
        break;
      default:
        instruction = "Unknown state.";
        break;
    }

    return { pastEvents, instruction };
  };

  const timelineData = request ? getTimelineData(request) : null;

  const handleAction = async (endpoint: string, uri: string) => {
    if (!request) return;
    
    setIsProcessing(true);
    try {
      const data: any = {
        id: request.id,
        uri: uri,
      };

      if (endpoint.includes('oidc4vp')) {
        data.entity = "authority";
      }

      await customInstance(endpoint, {
        method: "POST",
        data,
      });
      // Invalidate query to refetch data instead of full page reload
      queryClient.invalidateQueries({ queryKey: getGetAllVCRequestsQueryKey() });
      
    } catch (err) {
      console.error(err);
    } finally {
      setIsProcessing(false);
    }
  };

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
                  <Badge className={`border ${getStatusColor(request.status || '')}`}>
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

              {(request.vc_uri || request.verification_uri) && request.status?.toLowerCase() !== 'finalized' && (
                <div className="pt-4 border-t space-y-6">
                  {request.vc_uri ? (
                    <div className="space-y-3">
                      <DetailItem label="VC URI (Claiming)" labelClassName="text-green-500">
                        <div className="mt-2 flex flex-col sm:flex-row gap-6 items-start">
                          <div className="p-3 bg-white rounded-lg shadow-sm border border-stroke flex-shrink-0">
                            <QRCode value={request.vc_uri} size={120} />
                          </div>
                          <div className="flex-1 w-full space-y-3">
                             <p className="text-xs text-muted-foreground italic">Scan this QR to claim your Verifiable Credential directly in your wallet.</p>
                             <UriDisplay uri={request.vc_uri} />
                             <Button 
                               className="w-full sm:w-auto bg-green-600 hover:bg-green-700 text-white" 
                               size="sm"
                               onClick={() => handleAction('/vc-request/oidc4vci', request.vc_uri!)}
                               disabled={isProcessing}
                             >
                               {isProcessing ? <Loader2 className="mr-2 h-4 w-4 animate-spin" /> : <Shield className="mr-2 h-4 w-4" />}
                               Claim in Agent
                             </Button>
                          </div>
                        </div>
                      </DetailItem>
                    </div>
                  ) : request.verification_uri ? (
                    <div className="space-y-3">
                      <DetailItem label="Verification URI (Authentication)" labelClassName="text-amber-500">
                        <div className="mt-2 flex flex-col sm:flex-row gap-6 items-start">
                          <div className="p-3 bg-white rounded-lg shadow-sm border border-stroke flex-shrink-0">
                            <QRCode value={request.verification_uri} size={120} />
                          </div>
                          <div className="flex-1 w-full space-y-3">
                             <p className="text-xs text-muted-foreground italic">Use this QR if you need to authenticate with the authority before receiving the VC.</p>
                             <UriDisplay uri={request.verification_uri} />
                             <Button 
                               className="w-full sm:w-auto bg-amber-600 hover:bg-amber-700 text-white" 
                               size="sm"
                               onClick={() => handleAction('/vc-request/oidc4vp', request.verification_uri!)}
                               disabled={isProcessing}
                             >
                               {isProcessing ? <Loader2 className="mr-2 h-4 w-4 animate-spin" /> : <Key className="mr-2 h-4 w-4" />}
                               Present in Agent
                             </Button>
                          </div>
                        </div>
                      </DetailItem>
                    </div>
                  ) : null}
                </div>
              )}
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle>Request Timeline</CardTitle>
            </CardHeader>
            <CardContent className="space-y-6">
              {timelineData && timelineData.pastEvents.length > 0 && (
                <div className="relative pl-8 border-l-[3px] border-primary/10 space-y-8 ml-2">
                  {timelineData.pastEvents.map((event) => (
                    <div key={event.id} className="relative">
                      <span className="absolute -left-[43.5px] top-1 h-5 w-5 rounded-full bg-primary border-[5px] border-background shadow-sm" />
                      <div className="space-y-1">
                        <p className="text-sm font-medium text-muted-foreground">{event.title}</p>
                        {event.date && (
                          <p className="text-[10px] text-muted-foreground/60 flex items-center gap-1 font-mono">
                            <Clock className="h-3 w-3" />
                            <FormatDate date={event.date} />
                          </p>
                        )}
                      </div>
                    </div>
                  ))}
                </div>
              )}

              {timelineData && (
                <div className="pt-4 border-t border-stroke space-y-4">
                  <div className="flex justify-between items-center text-sm">
                    <span className="text-muted-foreground font-semibold uppercase tracking-wider text-xs">Current State:</span>
                    <Badge className={`border ${getStatusColor(request.status || '')}`}>
                      {request.status}
                    </Badge>
                  </div>
                  <div className="p-4 rounded-lg bg-muted/30 border border-stroke text-sm text-foreground/90 leading-relaxed">
                    {timelineData.instruction}
                  </div>
                  {request.ended_at && (request.status?.toLowerCase() === 'finalized' || request.status?.toLowerCase() === 'rejected' || request.status?.toLowerCase() === 'approved') && (
                    <p className="text-[10px] text-muted-foreground/60 flex items-center gap-1 font-mono justify-end mt-2">
                      <Clock className="h-3 w-3" />
                      Updated on: <FormatDate date={request.ended_at} />
                    </p>
                  )}
                </div>
              )}
            </CardContent>
          </Card>
        </div>
      </PageSection>
    </PageLayout>
  );
}

function UriDisplay({ uri }: { uri: string }) {
  const [copied, setCopied] = useState(false);

  const handleCopy = () => {
    navigator.clipboard.writeText(uri);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const truncatedUri = uri.length > 50 ? `${uri.substring(0, 25)}...${uri.substring(uri.length - 20)}` : uri;

  return (
    <div className="flex items-center gap-2 p-2 bg-muted/50 rounded border border-stroke overflow-hidden">
      <span className="font-mono text-[10px] truncate flex-1">{truncatedUri}</span>
      <Button variant="ghost" size="icon" className="h-7 w-7" onClick={handleCopy}>
        {copied ? <Check className="h-3 w-3 text-green-500" /> : <Copy className="h-3 w-3" />}
      </Button>
    </div>
  );
}

function DetailItem({ label, children, labelClassName }: { label: string; children: React.ReactNode; labelClassName?: string }) {
  return (
    <div className="flex flex-col gap-1.5">
      <span className={`text-xs font-semibold uppercase tracking-wider ${labelClassName || 'text-muted-foreground'}`}>{label}</span>
      <div className="text-sm font-medium">{children}</div>
    </div>
  );
}
