import React, { useState } from "react";
import { dspTitle } from "../../data/catalog";
import { DcatVocabularies } from "../../data/dcat";
import { Heading } from "@eunomia/ui";
import { Badge } from "@eunomia/ui";
import { FormatDate } from "@eunomia/ui";
import { Link } from "@tanstack/react-router";
import { useGetDistributionsByDatasetId } from "shared/src/data/orval/distributions/distributions";
import { useGetPoliciesByEntityId } from "../../data/orval/odrl-policies/odrl-policies";
import { Card, CardHeader, CardContent, CardFooter } from "@eunomia/ui";
import {
  ArrowUpRight,
  ShieldCheck,
  Layers,
  Calendar,
  ChevronDown,
  ChevronUp,
  Pencil,
  Trash2,
} from "lucide-react";
import { Button } from "@eunomia/ui";
import { PublicationBadge } from "../catalog/Publication";

interface DatasetItemProps {
  date: string;
  title: string;
  description?: string;
  prevRoute: string;
  datasetId: string;
  ownDataset: boolean;
  dataset: any;
  /** Shown as card actions when given: own datasets only. */
  onEdit?: () => void;
  onDelete?: () => void;
}

const DatasetItem: React.FC<DatasetItemProps> = ({
  date,
  title,
  description,
  prevRoute,
  datasetId,
  ownDataset,
  dataset,
  onEdit,
  onDelete,
}) => {
  const { data: distributionsData } = useGetDistributionsByDatasetId(datasetId);
  const { data: policiesData } = useGetPoliciesByEntityId(datasetId);

  const distributions: any[] = dataset?.distribution
    ? dataset.distribution
    : distributionsData?.status === 200
      ? distributionsData.data
      : [];

  const policies: any[] = dataset?.hasPolicy
    ? dataset.hasPolicy
    : policiesData?.status === 200
      ? policiesData.data
      : [];

  const [showMore, setShowMore] = useState(false);

  const datasetUrl = ownDataset
    ? `/catalog/${prevRoute}/dataset/${datasetId}`
    : `/catalog/participant/${prevRoute}/dataset/${datasetId}`;

  return (
    <Card
      variant="interactive"
      className="h-full flex flex-col justify-between group transition-all duration-200"
    >
      <CardHeader className="pb-3 space-y-2">
        <Link to={datasetUrl}>
          <div className="flex items-start justify-between gap-2">
            <Heading
              level="h5"
              className="!mb-0 font-semibold text-foreground group-hover:text-brand-sky underline-offset-2 hover:underline line-clamp-1"
            >
              {title || "Untitled Dataset"}
            </Heading>
            <span className="flex flex-shrink-0 items-center gap-1.5">
              {ownDataset && dataset?.draft !== undefined && (
                <PublicationBadge draft={dataset.draft} />
              )}
              <ArrowUpRight className="h-4 w-4 text-muted-foreground group-hover:text-brand-sky transition-colors flex-shrink-0 mt-0.5" />
            </span>
          </div>
        </Link>
        <p className="text-xs text-muted-foreground line-clamp-2 leading-relaxed">
          {description || "No description provided for this dataset."}
        </p>
      </CardHeader>

      <CardContent className="space-y-3">
        {(dataset?.keywords?.length > 0 || dataset?.themes?.length > 0) && (
          <div className="flex flex-wrap gap-1">
            {(dataset.themes ?? []).map((t: string) => (
              <span
                key={t}
                title={t}
                className="rounded bg-brand-sky/10 px-1.5 py-0.5 text-xs text-brand-sky"
              >
                {DcatVocabularies.label(DcatVocabularies.themes, t)}
              </span>
            ))}
            {(dataset.keywords ?? []).slice(0, 5).map((k: string) => (
              <span key={k} className="rounded bg-ink/[0.07] px-1.5 py-0.5 text-xs text-ink/75">
                {k}
              </span>
            ))}
          </div>
        )}
        <div className="flex flex-wrap items-center gap-2">
          <Badge
            variant="outline"
            className="flex items-center gap-1 text-xs font-mono py-0.5 px-2 bg-purple-500/10 text-purple-700 dark:text-purple-300 border-purple-500/20"
          >
            <ShieldCheck className="h-3 w-3" />
            {policies.length} {policies.length === 1 ? "Policy" : "Policies"}
          </Badge>

          <Badge
            variant="outline"
            className="flex items-center gap-1 text-xs font-mono py-0.5 px-2 bg-sky-500/10 text-sky-700 dark:text-sky-300 border-sky-500/20"
          >
            <Layers className="h-3 w-3" />
            {distributions.length} {distributions.length === 1 ? "Distribution" : "Distributions"}
          </Badge>

          {(policies.length > 0 || distributions.length > 0) && (
            <button
              type="button"
              onClick={() => setShowMore((prev) => !prev)}
              className="inline-flex items-center gap-0.5 text-xs text-muted-foreground hover:text-foreground ml-auto transition-colors font-medium"
            >
              <span>{showMore ? "Less" : "Details"}</span>
              {showMore ? <ChevronUp className="h-3 w-3" /> : <ChevronDown className="h-3 w-3" />}
            </button>
          )}
        </div>

        {showMore && (
          <div className="pt-2 border-t border-ink/5 flex flex-col gap-2">
            {policies.length > 0 && (
              <div className="flex flex-col gap-1">
                <span className="text-xs font-mono uppercase text-muted-foreground">Policies</span>
                <div className="flex flex-wrap gap-1">
                  {policies.map((p: any, idx: number) => {
                    const policyTitle =
                      p.description
                        ?.split(" ")
                        .slice(0, 3)
                        .map((w: string) => w.charAt(0).toUpperCase() + w.slice(1))
                        .join(" ") || `Policy #${idx + 1}`;
                    return (
                      <Badge key={idx} variant="detail" className="text-xs py-0.5 px-1.5">
                        {policyTitle}
                      </Badge>
                    );
                  })}
                </div>
              </div>
            )}

            {distributions.length > 0 && (
              <div className="flex flex-col gap-1">
                <span className="text-xs font-mono uppercase text-muted-foreground">
                  Distributions
                </span>
                <div className="flex flex-wrap gap-1">
                  {distributions.map((d: any, idx: number) => (
                    <Badge key={idx} variant="detail" className="text-xs py-0.5 px-1.5">
                      {dspTitle(d) || `Distribution #${idx + 1}`}
                    </Badge>
                  ))}
                </div>
              </div>
            )}
          </div>
        )}
      </CardContent>

      <CardFooter className="pt-2.5 flex items-center justify-between text-xs border-t border-ink/5 bg-background-800/10 rounded-b-xl">
        <span className="font-mono text-muted-foreground/80 truncate max-w-[140px]">
          {datasetId ? `#${datasetId.slice(-8)}` : ""}
        </span>
        <div className="flex items-center gap-1 text-muted-foreground">
          <Calendar className="h-3 w-3 opacity-60" />
          <FormatDate date={date} />
          {onEdit && (
            <Button
              size="icon_sm"
              variant="ghost"
              title="Edit dataset"
              className="ml-1 h-7 w-7"
              onClick={onEdit}
            >
              <Pencil />
            </Button>
          )}
          {onDelete && (
            <Button
              size="icon_sm"
              variant="ghost"
              title="Delete dataset"
              className="h-7 w-7 text-danger-700 dark:text-danger-300"
              onClick={onDelete}
            >
              <Trash2 />
            </Button>
          )}
        </div>
      </CardFooter>
    </Card>
  );
};

export default DatasetItem;
