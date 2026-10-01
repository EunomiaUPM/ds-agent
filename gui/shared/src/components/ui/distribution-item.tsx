import React from "react";
import Heading from "shared/src/components/ui/heading";
import { ArrowRight, Layers, Server } from "lucide-react";
import { Link } from "@tanstack/react-router";
import {
  Card,
  CardHeader,
  CardTitle,
  CardDescription,
  CardContent,
} from "shared/src/components/ui/card";

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
    <Card className="h-full flex flex-col justify-between hover:border-ink/20 transition-colors">
      <CardHeader className="pb-3 space-y-1.5">
        <div className="flex items-center gap-2">
          <Layers className="h-4 w-4 text-brand-sky flex-shrink-0" />
          <CardTitle className="text-base font-semibold">{title || "Distribution"}</CardTitle>
        </div>
        <CardDescription className="line-clamp-3">
          {description || "No description provided for this distribution."}
        </CardDescription>
      </CardHeader>

      {ownDataset && (
        <CardContent className="pt-2">
          <div className="rounded-lg border border-ink/10 bg-background-800/30 divide-y divide-ink/5 text-xs">
            <div className="flex items-center justify-between p-2.5">
              <span className="text-muted-foreground font-medium flex items-center gap-1.5">
                <Server className="h-3.5 w-3.5 text-brand-sky" />
                Connector Instance
              </span>
              <Link
                to={"/catalog/$prevRoute/distribution-connector/$distributionId"}
                params={{
                  prevRoute: prevRoute!,
                  distributionId: distribuionId!,
                }}
                className="inline-flex items-center gap-1 text-primary hover:underline font-mono text-xs group"
              >
                <span>View Details</span>
                <ArrowRight className="h-3 w-3 transition-transform group-hover:translate-x-0.5" />
              </Link>
            </div>

            <div className="flex items-center justify-between p-2.5">
              <span className="text-muted-foreground font-medium flex items-center gap-1.5">
                <Server className="h-3.5 w-3.5 text-emerald-700 dark:text-emerald-400" />
                Dataservice
              </span>
              <Link
                to={
                  ownDataset
                    ? "/catalog/$prevRoute/data-service/$dataserviceId"
                    : "/catalog/participant/$prevRoute/data-service/$dataserviceId"
                }
                params={{
                  prevRoute: prevRoute!,
                  dataserviceId: dataserviceId!,
                }}
                className="inline-flex items-center gap-1 text-emerald-700 dark:text-emerald-400 hover:underline font-mono text-xs group"
              >
                <span>Endpoint Details</span>
                <ArrowRight className="h-3 w-3 transition-transform group-hover:translate-x-0.5" />
              </Link>
            </div>
          </div>
        </CardContent>
      )}
    </Card>
  );
};

export default DistributionItem;
