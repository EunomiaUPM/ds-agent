import { createFileRoute, Link } from "@tanstack/react-router";
import { useGetAllVCRequests } from "shared/src/data/orval/vc-request/vc-request";
import { DataTable } from "shared/src/components/DataTable";
import { FormatDate } from "shared/src/components/ui/format-date";
import { Button } from "shared/src/components/ui/button";
import { Badge } from "shared/src/components/ui/badge";
import { ArrowRight, Plus, ArrowUpDown, ArrowUp, ArrowDown } from "lucide-react";
import { useState, useMemo } from "react";
import { PageLayout } from "shared/src/components/layout/PageLayout";
import { PageHeader } from "shared/src/components/layout/PageHeader";
import { PageSection } from "shared/src/components/layout/PageSection";
import { formatUrn } from "shared/src/lib/utils";

/**
 * Route for listing all VC requests to an authority.
 */
export const Route = createFileRoute("/authority/")({
  component: AuthorityRequestsPage,
});

function AuthorityRequestsPage() {
  const { data: response } = useGetAllVCRequests();
  const rawRequests = response?.status === 200 ? response.data : [];

  const [sortConfig, setSortConfig] = useState<{ key: string; direction: 'asc' | 'desc' } | null>(null);

  const requests = useMemo(() => {
    let sortableRequests = [...(rawRequests || [])];
    if (sortConfig !== null) {
      sortableRequests.sort((a: any, b: any) => {
        const aVal = a[sortConfig.key];
        const bVal = b[sortConfig.key];
        
        if (aVal === bVal) return 0;
        
        if (aVal === null || aVal === undefined) return sortConfig.direction === 'asc' ? -1 : 1;
        if (bVal === null || bVal === undefined) return sortConfig.direction === 'asc' ? 1 : -1;

        const aString = String(aVal).toLowerCase();
        const bString = String(bVal).toLowerCase();

        if (aString < bString) {
          return sortConfig.direction === 'asc' ? -1 : 1;
        }
        if (aString > bString) {
          return sortConfig.direction === 'asc' ? 1 : -1;
        }
        return 0;
      });
    }
    return sortableRequests;
  }, [rawRequests, sortConfig]);

  const handleSort = (key: string) => {
    let direction: 'asc' | 'desc' = 'asc';
    if (sortConfig && sortConfig.key === key && sortConfig.direction === 'asc') {
      direction = 'desc';
    }
    setSortConfig({ key, direction });
  };

  const getSortIcon = (key: string) => {
    if (!sortConfig || sortConfig.key !== key) return <ArrowUpDown className="ml-2 h-4 w-4 opacity-50" />;
    return sortConfig.direction === 'asc' ? <ArrowUp className="ml-2 h-4 w-4" /> : <ArrowDown className="ml-2 h-4 w-4" />;
  };

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
  return (
    <PageLayout>
      <PageHeader title="Request Credential">
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
              header: (
                <Button variant="ghost" onClick={() => handleSort('id')} className="p-0 h-auto font-semibold">
                  Request ID {getSortIcon('id')}
                </Button>
              ),
              cell: (a: any) => <Badge variant={"info"}>{formatUrn(a.id)}</Badge>,
            },
            {
              header: (
                <Button variant="ghost" onClick={() => handleSort('authority_id')} className="p-0 h-auto font-semibold">
                  Authority ID {getSortIcon('authority_id')}
                </Button>
              ),
              cell: (a: any) => (
                <div className="flex flex-col gap-1">
                  <Badge variant={"info"}>{a.authority_id ? formatUrn(a.authority_id) : "-"}</Badge>
                </div>
              ),
            },
            {
              header: (
                <Button variant="ghost" onClick={() => handleSort('authority_slug')} className="p-0 h-auto font-semibold">
                  Authority Name {getSortIcon('authority_slug')}
                </Button>
              ),
              cell: (a: any) => a.authority_slug || "-",
            },
            {
              header: (
                <Button variant="ghost" onClick={() => handleSort('vc_type')} className="p-0 h-auto font-semibold">
                  VC Type {getSortIcon('vc_type')}
                </Button>
              ),
              cell: (a: any) => a.vc_type,
            },
            {
              header: (
                <Button variant="ghost" onClick={() => handleSort('status')} className="p-0 h-auto font-semibold">
                  Status {getSortIcon('status')}
                </Button>
              ),
              cell: (a: any) => (
                <Badge className={`border ${getStatusColor(a.status)}`}>
                  {a.status || "-"}
                </Badge>
              ),
            },
            {
              header: (
                <Button variant="ghost" onClick={() => handleSort('created_at')} className="p-0 h-auto font-semibold">
                  Created at {getSortIcon('created_at')}
                </Button>
              ),
              cell: (a: any) => (a.created_at ? <FormatDate date={a.created_at} /> : "-"),
            },
            {
              header: "Link",
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
