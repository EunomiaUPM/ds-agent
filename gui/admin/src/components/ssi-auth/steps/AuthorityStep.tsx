
import { useContext } from "react";
import { SSIAuthContext } from "shared/src/context/SSIAuthContext";
import { AuthorityConnectorForm } from "../forms/AuthorityConnectorForm";
import { VCRequestForm } from "../forms/VCRequestForm";
import { VCAcceptForm } from "../forms/VCAcceptForm";
import { DataTable, Column } from "shared/src/components/DataTable";
import { Loader2, Circle } from "lucide-react";
import {AuthorityConnectorFormForDemo} from "@/components/ssi-auth/forms/AuthorityConnectorFormForDemo.tsx";

export function AuthorityStep() {
  const ssiAuthContext = useContext(SSIAuthContext);

  const vcRequestColumns: Column<any>[] = [
    {
      header: "ID",
      accessorKey: "id",
      cell: (item) => <span className="font-mono text-xs">{item.id}</span>,
    },
    { header: "Status", accessorKey: "status" },
    { header: "Type", accessorKey: "vc_type" },
    { header: "Created At", accessorKey: "created_at" },
  ];

  return (
    <div className="space-y-6 mt-4">
      <div className="space-y-4">
        <div>
          <h3 className="text-lg font-medium">2. Connect to Authority</h3>
          <p className="text-muted-foreground text-sm">
            Fetch the Authority DID and request a Dataspace Verifiable Credential.
          </p>
        </div>

        <div className="w-full border rounded-lg p-4 space-y-4">
          <AuthorityConnectorFormForDemo />
          <div>
            {ssiAuthContext.authDid.did ? (
              <div className="text-sm font-mono p-3 bg-muted rounded break-all border">
                <span className="font-semibold select-none mr-2">
                  Authority DID:
                </span>
                {ssiAuthContext.authDid.did}
              </div>
            ) : (
              <div className="text-sm text-muted-foreground italic">
                Fetch authority DID to proceed.
              </div>
            )}
          </div>
        </div>
      </div>

      <hr />

      <div className="space-y-4">
        <h3 className="text-lg font-medium">3. Request Credential</h3>
        <div className="flex gap-4 flex-col items-start w-full border rounded-lg p-4">
          <VCRequestForm />

          <div className="w-full mt-2">
            <div className="flex items-center gap-2 mb-2">
              <h4 className="text-sm font-medium">Request Status</h4>
              {ssiAuthContext.authRequestsPollInterval > 0 && (
                <Loader2 className="h-4 w-4 animate-spin text-orange-500" />
              )}
              {ssiAuthContext.authRequests.length > 0 &&
                ssiAuthContext.authRequests[0].status === "Approved" && (
                  <Circle className="h-4 w-4 text-green-500 fill-current" />
                )}
            </div>
            <div className="rounded-md border">
              <DataTable
                columns={vcRequestColumns}
                data={
                  Array.isArray(ssiAuthContext.authRequests)
                    ? ssiAuthContext.authRequests
                    : []
                }
                keyExtractor={(item) => item.id}
              />
            </div>
          </div>

          {ssiAuthContext.oidc4vciRequestUri && (
            <div className="w-full space-y-2">
              <div className="text-sm font-mono p-3 bg-muted rounded break-all border">
                <span className="font-semibold select-none block mb-1">
                  Credential Offer URI:
                </span>
                {ssiAuthContext.oidc4vciRequestUri}
              </div>

              <VCAcceptForm />
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
