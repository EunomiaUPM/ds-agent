import React from "react";
import Heading from "shared/src/components/ui/heading";
import { ExternalLink } from "lucide-react";
import { Link } from "@tanstack/react-router";

interface DistributionItemProps {
  title?: string;
  ownDataset?: boolean;
  description?: string;
  date?: string;
  prevRoute?: string;
  distribuionId?: string;
  dataserviceId?: string;
}

const DistributionItem: React.FC<DistributionItemProps> = ({
  title,
  ownDataset,
  description,
  prevRoute,
  distribuionId,
  dataserviceId,
}) => {
  return (
    <div className="distribution-container  max-w-[600px] h-full dataset-item-container bg-brand-sky/5 border rounded-md border-white/15 flex flex-col p-4 gap-3">
      <div className="distribution-text">
        <Heading level="h4" className="mb-3">
          {title ? title : "Distribution Title"}
        </Heading>
        <p className="text-sm">{description ? description : "Distribution Description"}</p>
      </div>
      {ownDataset ? (
        <div className="distribution-table text-sm">
          <div className="grid grid-cols-2 border-y border-white/10 py-2 gap-x-5 ">
            <span className="font-bold">Associated Connector:</span>
            <Link
              target="_blank"
              to={"/catalog/$prevRoute/distribution-connector/$distributionId"}
              params={{
                prevRoute: prevRoute!,
                distributionId: distribuionId!,
              }}
            >
              <span className="underline-offset-2 hover:underline flex gap-2">
                Connector Distribution <ExternalLink className="h-4 w-4" />
              </span>
            </Link>
          </div>
          <div className="grid grid-cols-2 border-b border-white/10 py-2 gap-x-5">
            <span className="font-bold">Associated Dataservice:</span>
            <Link
              target="_blank"
              to={
                ownDataset
                  ? "/catalog/$prevRoute/data-service/$dataserviceId"
                  : "/catalog/participant/$prevRoute/data-service/$dataserviceId"
              }
              params={{
                prevRoute: prevRoute!,
                dataserviceId: dataserviceId!,
              }}
            >
              <span className="underline-offset-2 hover:underline flex gap-2">
                Dataservice <ExternalLink className="h-4 w-4" />
              </span>
            </Link>
          </div>
        </div>
      ) : null}
    </div>
  );
};

export default DistributionItem;
