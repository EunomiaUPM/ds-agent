import React from "react";
import { useQuery } from "@tanstack/react-query";
import { useLink } from "../link";
import { customInstance } from "../lib/http";
import { Identifier } from "../identity/Identifier";
import { PageSection } from "../layout/PageSection";
import { FieldGrid } from "../layout/FieldGrid";
import { DataTable } from "../table/DataTable";
import { Skeleton } from "../ui/skeleton";
import { Badge } from "../ui/badge";
import { FormatDate } from "../ui/format-date";

export interface WalletKeyRef {
  internal: string;
  fragment: string;
}

export interface WalletDidModel {
  id: string;
  did: string;
  alias: string;
  default: boolean;
  type: string;
  keys: WalletKeyRef[];
  default_key: WalletKeyRef;
  did_document: unknown;
  service?: unknown[] | null;
}

export interface WalletInfoData {
  id: string;
  name: string;
  createdOn: string;
  addedOn: string;
  permission: string;
  dids: WalletDidModel[];
}

export interface WalletInfoResponse {
  status: number;
  data: WalletInfoData;
}

export function WalletInfoView({ didTabHref = "/wallet/did" }: { didTabHref?: string }) {
  const Link = useLink();
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
      <PageSection title="General information">
        <FieldGrid
          columns={4}
          fields={[
            { label: "Wallet ID", value: <span className="font-mono text-xs">{info.id}</span> },
            { label: "Name", value: info.name },
            { label: "Created", value: <FormatDate date={info.createdOn || null} /> },
            {
              label: "Permission level",
              value: <Badge variant="outline">{info.permission}</Badge>,
            },
          ]}
        />
      </PageSection>

      <PageSection title="Associated DIDs">
        <p className="text-xs text-muted-foreground mb-4">
          Keys and the default DID are managed in the{" "}
          <Link to={didTabHref} className="font-medium text-brand-sky hover:underline">
            DID
          </Link>{" "}
          tab.
        </p>
        <DataTable
          searchable={false}
          data={info.dids}
          keyExtractor={(d) => d.did}
          columns={[
            {
              header: "Alias",
              accessorKey: "alias",
              cell: (d) => <span className="font-medium">{d.alias}</span>,
            },
            {
              header: "DID",
              accessorKey: "did",
              cell: (d) => <Identifier value={d.did} tail={12} />,
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
              cell: (d) => <span className="text-xs text-muted-foreground">{d.keys.length}</span>,
            },
          ]}
        />
      </PageSection>
    </div>
  );
}
