import React from "react";
import { InfoList } from "@eunomia/ui";
import { Badge } from "@eunomia/ui";

type TransferProcessDataPlaneComponentProps = {
  dataPlane: DataplaneSession;
};

/**
 * Component for displaying Data Plane details in a Transfer Process.
 */
const TransferProcessDataPlaneComponent: React.FC<TransferProcessDataPlaneComponentProps> = ({
  dataPlane,
}) => {
  const scopedListItemKeyClasses = "basis-[33%]";

  return (
    <InfoList
      items={
        [
          { label: "Data plane Id", value: { type: "urn", value: dataPlane.id } },
          {
            label: "Process Direction",
            value: {
              type: "custom",
              content: <Badge className="uppercase">{dataPlane.process_direction}</Badge>,
            },
          },
          dataPlane.upstream_hop.url
            ? {
                label: "Upstream protocol",
                value: {
                  type: "custom",
                  content: <Badge className="uppercase">{dataPlane.upstream_hop.protocol}</Badge>,
                },
              }
            : undefined,
          dataPlane.downstream_hop.url
            ? {
                label: "Downstream protocol",
                value: {
                  type: "custom",
                  content: <Badge className="uppercase">{dataPlane.downstream_hop.protocol}</Badge>,
                },
              }
            : undefined,
          {
            label: "Process address",
            value: {
              type: "custom",
              content: (
                <Badge variant="info">
                  {dataPlane.process_address.url.slice(0, -45) + "[...]"}
                </Badge>
              ),
            },
          },
          { label: "Created At", value: { type: "date", value: dataPlane.created_at } },
          { label: "Updated At", value: { type: "date", value: dataPlane.updated_at } },
          { label: "State", value: { type: "status", value: dataPlane.state } },
        ].filter((item) => item !== undefined) as any
      }
    />
  );
};

export default TransferProcessDataPlaneComponent;
