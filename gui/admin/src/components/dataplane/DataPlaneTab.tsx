import React from "react";
import { DataplaneInfoResponse, DataplaneTransferDto } from "shared/src/data/orval/model";
import { Badge } from "@eunomia/ui";
import { RelativeTime } from "@eunomia/ui";
import { FieldGrid } from "@eunomia/ui";
import { PageSection } from "@eunomia/ui";
import { JsonTable } from "@eunomia/ui";
import { Identifier } from "@eunomia/ui";
import { dataplaneStateVariant } from "./utils/dataplaneState";
import { CopyButton } from "./CopyButton";

/** The dataplane side of a transfer: where data enters and leaves, and how it flows. */
export function DataPlaneTab({
  dp,
  info,
}: {
  dp: DataplaneTransferDto | null;
  info: DataplaneInfoResponse | null;
}) {
  if (!dp) {
    return (
      <div className="rounded-lg border border-dashed border-ink/15 py-10 text-center text-sm text-ink/50">
        This transfer has no dataplane process yet.
      </div>
    );
  }
  const fields = Object.keys(dp.fields ?? {}).length > 0 ? dp.fields : undefined;

  return (
    <div className="space-y-6">
      <FieldGrid
        columns={3}
        fields={[
          {
            label: "State",
            value: (
              <Badge variant="status" state={dataplaneStateVariant(dp.state)} size="sm">
                {dp.state}
              </Badge>
            ),
          },
          { label: "Mode", value: dp.interactionMode },
          { label: "Role", value: dp.role },
          {
            label: "Endpoint",
            span: 3,
            value: info?.ingress_url ? (
              <span className="flex min-w-0 items-center gap-1.5">
                <span className="truncate font-mono text-xs text-brand-sky">
                  {info.ingress_url}
                </span>
                <CopyButton text={info.ingress_url} />
              </span>
            ) : undefined,
          },
          { label: "Connector instance", value: <Identifier value={dp.connectorInstanceId} /> },
          { label: "Dataplane process", value: <Identifier value={dp.id} /> },
          {
            label: "Last change",
            value: <RelativeTime date={dp.updatedAt ?? dp.createdAt} />,
          },
        ]}
      />

      <div className="grid grid-cols-1 gap-6 xl:grid-cols-2">
        <PageSection title="Ingress">
          <JsonTable value={dp.ingressConfig} emptyText="No ingress configuration." />
        </PageSection>
        <PageSection title="Egress">
          <JsonTable value={dp.egressConfig} emptyText="No egress configuration." />
        </PageSection>
        {!!dp.flowControl && (
          <PageSection title="Flow control">
            <JsonTable value={dp.flowControl} />
          </PageSection>
        )}
        {fields && (
          <PageSection title="Fields">
            <JsonTable value={fields} />
          </PageSection>
        )}
      </div>
    </div>
  );
}
