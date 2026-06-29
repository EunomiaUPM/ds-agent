import { createFileRoute } from "@tanstack/react-router";
import { useQuery } from "@tanstack/react-query";
import { customInstance } from "shared/src/data/orval-mutator";
import { PageSection } from "shared/src/components/layout/PageSection";
import { Skeleton } from "shared/src/components/ui/skeleton";
import { Badge } from "shared/src/components/ui/badge";
import { InfoList } from "shared/src/components/ui/info-list";
import { DataTable } from "shared/src/components/DataTable";

interface KeyRef {
  internal: string;
  fragment: string;
}

interface DidModel {
  id: string;
  did: string;
  alias: string;
  default: boolean;
  type: string;
  keys: KeyRef[];
  default_key: KeyRef;
  did_document: any;
  service?: any[] | null;
}

interface WalletInfo {
  id: string;
  name: string;
  createdOn: string;
  addedOn: string;
  permission: string;
  dids: DidModel[];
}

interface WalletInfoResponse {
  status: number;
  data: WalletInfo;
}

const WalletInfoPage = () => {
  const {
    data: response,
    isLoading,
    error,
  } = useQuery({
    queryKey: ["wallet-info-custom"],
    queryFn: () => customInstance<WalletInfoResponse>("/wallet/info", { method: "GET" }),
  });

  const info = response?.status === 200 ? response.data : null;

  if (isLoading) {
    return (
      <div className="space-y-8">
        <Skeleton className="h-40 w-full" />
        <Skeleton className="h-64 w-full" />
      </div>
    );
  }

  if (error || !info) {
    return <div className="text-destructive font-mono text-xs">Error loading wallet info</div>;
  }

  return (
    <div className="space-y-12 pb-20">
      <PageSection title="General Information">
        <div className="bg-white/5 border border-white/10 rounded-xl p-8 backdrop-blur-sm shadow-xl">
          <InfoList
            items={[
              { label: "Wallet ID", value: info.id },
              { label: "Name", value: info.name },
              { label: "Created On", value: info.createdOn || "—" },
              { label: "Permission Level", value: info.permission },
            ]}
          />
        </div>
      </PageSection>

      <PageSection title="Associated DIDs">
        <p className="text-xs text-muted-foreground mb-4">
          Quick overview of stored DIDs. Manage keys and defaults from the{" "}
          <span className="font-semibold text-primary">DID</span> tab.
        </p>
        <DataTable
          data={info.dids}
          keyExtractor={(d) => d.did}
          columns={[
            {
              header: "Alias",
              accessorKey: "alias",
              cell: (d) => <span className="font-semibold text-primary/80">{d.alias}</span>,
            },
            {
              header: "DID",
              accessorKey: "did",
              cell: (d) => (
                <span className="font-mono text-[10px] text-muted-foreground break-all">
                  {d.did}
                </span>
              ),
            },
            {
              header: "Type",
              accessorKey: "type",
              cell: (d) => (
                <Badge variant="info" className="font-mono">
                  {d.type}
                </Badge>
              ),
            },
            {
              header: "Default",
              accessorKey: "default",
              cell: (d) => (
                <Badge variant={d.default ? "default" : "info"}>
                  {d.default ? "PRIMARY" : "SECONDARY"}
                </Badge>
              ),
            },
            {
              header: "# Keys",
              cell: (d) => (
                <span className="text-xs text-muted-foreground">{d.keys.length}</span>
              ),
            },
          ]}
        />
      </PageSection>
    </div>
  );
};

export const Route = createFileRoute("/wallet/info")({
  component: WalletInfoPage,
});
