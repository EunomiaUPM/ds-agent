import { createFileRoute } from "@tanstack/react-router";
import { formatUrn } from "shared/src/lib/utils.ts";
import { PageLayout } from "shared/src/components/layout/PageLayout";
import { PageHeader } from "shared/src/components/layout/PageHeader";
import { PageSection } from "shared/src/components/layout/PageSection";

// Components
import Heading from "../../../../../shared/src/components/ui/heading.tsx";
import { Badge, BadgeRole } from "shared/src/components/ui/badge.tsx";
import { InfoList } from "shared/src/components/ui/info-list";
import { useGetParticipantById } from "shared/data/orval/participants/participants.ts";
import { GeneralErrorComponent } from "@/components/GeneralErrorComponent.tsx";
import { useGetAgreementsByParticipantId } from "shared/data/orval/negotiations/negotiations.ts";
import { Skeleton } from "shared/src/components/ui/skeleton";
import dayjs from "dayjs";
import { Card, CardContent, CardHeader, CardTitle } from "shared/src/components/ui/card";
import { Separator } from "shared/src/components/ui/separator";
import { History, Shield, Globe, Cpu, Key, Calendar, Activity } from "lucide-react";

import { Button } from "shared/src/components/ui/button.tsx";
import { ParticipantDto } from "shared/data/orval/model/participantDto";

interface Participant extends ParticipantDto {
  last_interaction?: string;
  saved_at?: string;
  extra_fields?: any;
}

/**
 * Route for displaying individual participant details.
 */
export const Route = createFileRoute("/participants/$participantId/")({
  component: RouteComponent,
});

function RouteComponent() {
  const { participantId } = Route.useParams();
  const {
    data: participant,
    isLoading: isParticipantLoading,
    isError: isParticipantError,
    error: participantError,
  } = useGetParticipantById(participantId);
  const {
    data: agreements,
    isLoading: isAgreementsLoading,
    isError: isAgreementsError,
    error: agreementsError,
  } = useGetAgreementsByParticipantId(participantId);

  if (isParticipantLoading || isAgreementsLoading) {
    return (
      <PageLayout>
        <PageHeader title="Participant Details" />
        <div className="space-y-6">
          <Skeleton className="h-48 w-full rounded-xl" />
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <Skeleton className="h-64 w-full rounded-xl" />
            <Skeleton className="h-64 w-full rounded-xl" />
          </div>
        </div>
      </PageLayout>
    );
  }

  if (isParticipantError || !participant || participant.status !== 200) {
    const error =
      participantError instanceof Error ? participantError : new Error("Participant not found");
    return <GeneralErrorComponent error={error} reset={() => {}} />;
  }

  const p = participant.data as Participant;
  const agreementList = agreements?.status === 200 ? agreements.data : [];

  return (
    <PageLayout>
      <PageHeader
        title={p.participant_slug || "Participant Details"}
        badge={
          <div className="flex gap-2">
            <Badge variant="role" dsrole={p.participant_type as BadgeRole}>
              {p.participant_type}
            </Badge>
            {p.is_me && (
              <Badge variant="info">Local Agent</Badge>
            )}
          </div>
        }
      />

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        {/* Left Column: Essential Info */}
        <div className="lg:col-span-2 space-y-6">
          <Card>
            <CardHeader className="flex flex-row items-center justify-between pb-2">
              <CardTitle className="text-lg font-semibold flex items-center gap-2">
                <Shield className="h-5 w-5 text-brand-sky" />
                Identity Information
              </CardTitle>
            </CardHeader>
            <CardContent>
              <InfoList
                items={[
                  { 
                    label: "Identifier (DID)", 
                    value: {
                      type: "custom",
                      content: (
                        <div className="font-mono text-xs break-all bg-background-200 p-2 rounded border border-white/10">
                          {p.participant_id}
                        </div>
                      )
                    } 
                  },
                  { 
                    label: "Identity Token", 
                    value: {
                      type: "custom",
                      content: (
                        <div className="font-mono text-[10px] opacity-60 truncate max-w-[300px]">
                          {p.token || "No token assigned"}
                        </div>
                      )
                    } 
                  },
                  { 
                    label: "Base URL", 
                    value: {
                      type: "custom",
                      content: (
                        <a href={p.base_url} target="_blank" rel="noreferrer" className="flex items-center gap-1 text-brand-sky hover:underline">
                          <Globe className="h-3 w-3" />
                          {p.base_url}
                        </a>
                      )
                    } 
                  },
                ]}
              />
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle className="text-lg font-semibold flex items-center gap-2">
                <History className="h-5 w-5 text-brand-sky" />
                Agreements & Contracts
              </CardTitle>
            </CardHeader>
            <CardContent>
              {Array.isArray(agreementList) && agreementList.length > 0 ? (
                 <div className="space-y-4">
                    {agreementList.map((agreement: any, idx: number) => (
                      <div key={idx} className="p-3 border rounded-md bg-background-200/30 flex justify-between items-center">
                        <div className="flex gap-3 items-center">
                           <div className="p-2 bg-brand-sky/20 rounded-full">
                              <Shield className="h-4 w-4 text-brand-sky" />
                           </div>
                           <div>
                              <p className="text-sm font-medium">Agreement {agreement.id || idx}</p>
                              <p className="text-xs text-muted-foreground">Status: {agreement.status || 'Active'}</p>
                           </div>
                        </div>
                        <Button variant="ghost" size="sm">Details</Button>
                      </div>
                    ))}
                 </div>
              ) : (
                <div className="text-center py-10 opacity-50 border-2 border-dashed rounded-lg">
                  <p className="text-sm italic">No active agreements with this participant</p>
                </div>
              )}
            </CardContent>
          </Card>
        </div>

        {/* Right Column: Metadata & Activity */}
        <div className="space-y-6">
          <Card>
            <CardHeader>
              <CardTitle className="text-base font-semibold flex items-center gap-2">
                <Calendar className="h-4 w-4 text-brand-sky" />
                Timestamps
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="flex justify-between items-center text-sm">
                <span className="text-muted-foreground flex items-center gap-2">
                   <Key className="h-3 w-3" /> First Registered
                </span>
                <span className="font-medium">
                  {p.saved_at ? dayjs(p.saved_at).format("MMM D, YYYY") : "N/A"}
                </span>
              </div>
              <Separator />
              <div className="flex justify-between items-center text-sm">
                <span className="text-muted-foreground flex items-center gap-2">
                   <Activity className="h-3 w-3" /> Last Interaction
                </span>
                <span className="font-medium">
                  {p.last_interaction ? dayjs(p.last_interaction).format("MMM d, HH:mm") : "None"}
                </span>
              </div>
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle className="text-base font-semibold flex items-center gap-2">
                <Cpu className="h-4 w-4 text-brand-sky" />
                Extended Metadata
              </CardTitle>
            </CardHeader>
            <CardContent>
              {p.extra_fields && Object.keys(p.extra_fields).length > 0 ? (
                <pre className="text-[10px] bg-background-300 p-3 rounded-lg overflow-x-auto max-h-[300px]">
                  {JSON.stringify(p.extra_fields, null, 2)}
                </pre>
              ) : (
                <p className="text-xs italic text-muted-foreground text-center py-4">
                  No extra fields available
                </p>
              )}
            </CardContent>
          </Card>
        </div>
      </div>
    </PageLayout>
  );
}

const DataTypeSkeleton = () => (
  <div className="space-y-4">
    <Skeleton className="h-8 w-48" />
    <Skeleton className="h-24 w-full" />
  </div>
);
