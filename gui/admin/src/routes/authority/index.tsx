import { ArrowDown, ArrowRight, ArrowUp, ArrowUpDown, Plus } from "lucide-react";
import { useMemo, useState } from "react";
import { DataTable } from "shared/src/components/DataTable";
import { PageHeader } from "shared/src/components/layout/PageHeader";
import { PageLayout } from "shared/src/components/layout/PageLayout";
import { PageSection } from "shared/src/components/layout/PageSection";
import { Badge } from "shared/src/components/ui/badge";
import { Button } from "shared/src/components/ui/button";
import { FormatDate } from "shared/src/components/ui/format-date";
import { useGetAllVCRequests } from "shared/src/data/orval/vc-request/vc-request";
import { formatIdentifier, getFriendlyVCType } from "shared/src/lib/utils";

import { createFileRoute, Link } from "@tanstack/react-router";
import { useEffect } from "react";
import { useGetAllParticipants } from "shared/src/data/orval/participants/participants";
import WizardEndDialog from "shared/src/components/WizardEndDialog";

/**
 * Route for listing all VC requests to an authority.
 */
export const Route = createFileRoute("/authority/")({
  component: AuthorityRequestsPage,
});

function AuthorityRequestsPage() {
  const { data: response } = useGetAllVCRequests();
  const { data: participantsResponse } = useGetAllParticipants();
  const participants = participantsResponse?.status === 200 ? participantsResponse.data : [];

  const [showCongrats, setShowCongrats] = useState(false);

  useEffect(() => {
    try {
      const justJoined = sessionStorage.getItem("justJoinedDataspace");
      const authorities = participants.filter((p: any) => p.participant_type === "Authority");
      if (justJoined === "true" && authorities.length === 1) {
        setShowCongrats(true);
        sessionStorage.removeItem("justJoinedDataspace");
      }
    } catch (e) {
      // ignore storage errors
    }
  }, [participantsResponse]);
  const rawRequests = response?.status === 200 ? response.data : [];

  const [sortConfig, setSortConfig] = useState<{ key: string; direction: "asc" | "desc" } | null>(
    null,
  );

  const requests = useMemo(() => {
    let sortableRequests = [...(rawRequests || [])];
    if (sortConfig !== null) {
      sortableRequests.sort((a: any, b: any) => {
        const aVal = a[sortConfig.key];
        const bVal = b[sortConfig.key];

        if (aVal === bVal) return 0;

        if (aVal === null || aVal === undefined) return sortConfig.direction === "asc" ? -1 : 1;
        if (bVal === null || bVal === undefined) return sortConfig.direction === "asc" ? 1 : -1;

        const aString = String(aVal).toLowerCase();
        const bString = String(bVal).toLowerCase();

        if (aString < bString) {
          return sortConfig.direction === "asc" ? -1 : 1;
        }
        if (aString > bString) {
          return sortConfig.direction === "asc" ? 1 : -1;
        }
        return 0;
      });
    }
    return sortableRequests;
  }, [rawRequests, sortConfig]);



  const handleSort = (key: string) => {
    let direction: "asc" | "desc" = "asc";
    if (sortConfig && sortConfig.key === key && sortConfig.direction === "asc") {
      direction = "desc";
    }
    setSortConfig({ key, direction });
  };

  const getSortIcon = (key: string) => {
    if (!sortConfig || sortConfig.key !== key)
      return <ArrowUpDown className="ml-2 h-4 w-4 opacity-50" />;
    return sortConfig.direction === "asc" ? (
      <ArrowUp className="ml-2 h-4 w-4" />
    ) : (
      <ArrowDown className="ml-2 h-4 w-4" />
    );
  };

  return (
    <PageLayout>
      <WizardEndDialog
        open={showCongrats}
        sectionTitle="Dataspace Sign Up Tutorial"
        onClose={() => setShowCongrats(false)}
        title={"Congratulations"}
        content={<>
        Congratulations, you are part now of the Dataspace of Heimdall
        <br/>
        You can now browse the catalogs in the dataspace.
          </>}
        actionHref={"/catalog/"}
        actionLabel={"See dataspace"}
      />
      <PageHeader title="Credential Requests">
        <div className="flex justify-end mb-4">
          <Link to="/authority/new">
            <Button>
              <Plus className="mr-2 h-4 w-4" />
              Request New Credential
            </Button>
          </Link>
        </div>
      </PageHeader>

      <PageSection>
        <DataTable
          className="text-sm"
          data={requests ?? []}
          keyExtractor={(a) => a.id}
          columns={[
            {
              header: "Authority Name",
              // (
              //   <Button
              //     variant="ghost"
              //     onClick={() => handleSort("authority_slug")}
              //     className="p-0 h-auto font-semibold"
              //   >
              //     Authority Name {getSortIcon("authority_slug")}
              //   </Button>
              // ),
              cell: (a: any) => a.authority_slug || "-",
            },
            {
              header: "Request ID",
              // (
              //   <Button
              //     variant="ghost"
              //     onClick={() => handleSort("id")}
              //     className="p-0 h-auto font-semibold"
              //   >
              //     Request ID {getSortIcon("id")}
              //   </Button>
              // ),
              cell: (a: any) => <Badge variant={"info"}>{formatIdentifier(a.id)}</Badge>,
            },
            {
              header: "Credential Type",
              // (
              //   <Button
              //     variant="ghost"
              //     onClick={() => handleSort("vc_type")}
              //     className="p-0 h-auto font-semibold"
              //   >
              //     Credential Type {getSortIcon("vc_type")}
              //   </Button>
              // ),
              cell: (a: any) => <Badge variant="role">{getFriendlyVCType(a.vc_type)}</Badge>,
            },
            {
              header: "Status",
              // (
              //   <Button
              //     variant="ghost"
              //     onClick={() => handleSort("status")}
              //     className="p-0 h-auto font-semibold"
              //   >
              //     Status {getSortIcon("status")}
              //   </Button>
              // ),
              cell: (a: any) => (
                <Badge variant={"status"} state={a.status}>
                  {a.status || "-"}
                </Badge>
              ),
            },
            {
              header: "Created at",
              // (
              //   <Button
              //     variant="ghost"
              //     onClick={() => handleSort("created_at")}
              //     className="p-0 h-auto font-semibold"
              //   >
              //     Created at {getSortIcon("created_at")}
              //   </Button>
              // ),
              cell: (a: any) => (a.created_at ? <FormatDate date={a.created_at} /> : "-"),
            },
            {
              header: "Details",
              cell: (a) => (
                // @ts-ignore
                <Link to="/authority/request-details" search={{ requestId: a.id }}>
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
}
