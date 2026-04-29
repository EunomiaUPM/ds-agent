import React from "react";
import { useState } from "react";
import Heading from "shared/src/components/ui/heading";
import { Badge } from "./badge";
import { FormatDate } from "./format-date";
import { Link } from "@tanstack/react-router";
import { useGetDistributionsByDatasetId } from "shared/src/data/orval/distributions/distributions";
import { useGetPoliciesByEntityId } from "../../data/orval/odrl-policies/odrl-policies";

const DatasetItem = ({ date, title, description, prevRoute, datasetId, ownDataset, dataset }) => {
  //for own catalog
  const { data: distributionsData } = useGetDistributionsByDatasetId(datasetId);
  const { data: policiesData, refetch: refetchPolicies } = useGetPoliciesByEntityId(datasetId);

  console.log(dataset.distributions, "dataset in dataset item component");
  // if dataset is from participant, it will have attribute "distributions". If it's from
  // own catalog, we need to fetch distributions with useGetDistributionsByDatasetId
  const distributions = dataset?.distribution
    ? dataset.distribution
    : distributionsData?.status === 200
      ? distributionsData.data
      : [];

  // if dataset is from participant, it will have attribute "haspolicy". If it's from
  // own catalog, we need to fetch policies with useGetPoliciesByEntityId
  const policies = dataset?.hasPolicy
    ? dataset.hasPolicy
    : policiesData?.status === 200
      ? policiesData.data
      : [];

  //for participant catalog the "dataset" object contains policies and distributions

  const [showMore, setShowMore] = useState(false);
  const toggleDdatasetDetails = () => {
    setShowMore((prevState) => !prevState); // Alterna entre true y false,
    // cogiendo de base el estado. Si es false lo convierte a true, y viceversa
  };
  return (
    <div className="  h-full dataset-item-container bg-background-200/15 hover:bg-background-200/40 border rounded-md border-white/10 flex flex-col p-3 gap-1">
      <Link
        to={
          ownDataset
            ? "/catalog/$prevRoute/dataset/$datasetId"
            : "/catalog/participant/$prevRoute/dataset/$datasetId"
        }
        params={{
          prevRoute: prevRoute!,
          datasetId: datasetId!,
        }}
      >
        <div className="dataset-header flex justify-between items-center">
          <Heading level="h5" className="!mb-0 font-bold underline-offset-2 hover:underline">
            {" "}
            {title} Dataset{" "}
          </Heading>
        </div>
      </Link>
      <p className="dataset-item-description text-sm">
        {description || "Description of the dataset."}
      </p>
      <div className="h-2"></div>
      <div className="dataset-item-policies flex flex-col gap-2">
        <div className="dataset-item-details-summary flex gap-6 items-start ">
          <div className="policies-summary-container flex flex-col gap-2 text-xs uppercase">
            <span>{policies.length} policies </span>
            {policies.length > 0 &&
              policies.map((p) => (
                <Badge variant="detail" className={"text-2xs " + (showMore ? `flex` : `hidden`)}>
                  {p.description ? p.description : "Policy description"}
                </Badge>
              ))}
          </div>
          <div className="distributions-summary-container flex flex-col gap-2 text-xs uppercase">
            <span>{distributions.length} distributions </span>
            {/* Depending whether the dataset is from own catalog or participant catalog, 
                        distributions have attribute "dctTitle (own catalog) or "title" (participant catalog), so we check both and show the one that exists. */}
            {distributions.length > 0 &&
              distributions.map((d) => (
                <Badge variant="detail" className={"text-2xs " + (showMore ? `flex` : `hidden`)}>
                  {d.dctTitle ? d.dctTitle : d.title ? d.title : "Distribution title"}
                </Badge>
              ))}
          </div>
          <button
            className="font-bold text-2xs underline underline-offset-2"
            onClick={toggleDdatasetDetails}
          >
            {showMore ? "Show less" : "Show more"}{" "}
          </button>
        </div>
      </div>
      <div className="catalog-dates-updated flex gap-1 text-xs h-2 text-right justify-end mt-1 mb-1 italic text-muted-foreground">
        <p>Issued at:</p>
        <p>
          {" "}
          <FormatDate date={date} />{" "}
        </p>
      </div>
    </div>
  );
};

export default DatasetItem;
