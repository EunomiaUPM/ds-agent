import React from "react";
import { useState } from "react";
import Heading from "shared/src/components/ui/heading";
import { Badge } from "./badge";

const DatasetItem = () => {
  const [showMore, setShowMore] = useState(false);
  const toggleDdatasetDetails = () => {
    setShowMore((prevState) => !prevState); // Alterna entre true y false,
    // cogiendo de base el estado. Si es false lo convierte a true, y viceversa
  };
  return (
    <div className=" w-[500px] h-full dataset-item-container bg-brand-sky/5 border rounded-md border-white/20 flex flex-col p-4 gap-1">
      <div className="dataset-header flex justify-between items-center">
        <Heading level="h4" className="!mb-0">
          Dataset Title{" "}
        </Heading>
        <div className="catalog-dates-updated flex gap-1 text-sm">
          <p>Updated at:</p>
          <p>02/08/2025</p>
        </div>
      </div>
      <p className="dataset-item-description">
        Description of the dataset telling what data it has, which format, details about data.
      </p>
      <div className="h-2"></div>
      <div className="dataset-item-policies flex flex-col gap-2">
        <div className="dataset-item-details-summary flex gap-6 items-start ">
          <div className="policies-summary-container flex flex-col gap-2">
            <span>3 policies </span>
            <Badge variant="detail" className={showMore ? `flex` : `hidden`}>
              {" "}
              Policy title
            </Badge>
            <Badge variant="detail" className={showMore ? `flex` : `hidden`}>
              {" "}
              Research Juner Trial Access (large title)
            </Badge>
          </div>
          <div className="distributions-summary-container flex flex-col gap-2">
            <span>2 distributions </span>
            <Badge variant="detail" className={showMore ? `flex` : `hidden`}>
              {" "}
              Distribution title
            </Badge>
          </div>
          <button
            className="font-bold text-xs underline underline-offset-2 mt-1"
            onClick={toggleDdatasetDetails}
          >
            {showMore ? "Show less" : "Show more"}{" "}
          </button>
        </div>
      </div>
    </div>
  );
};

export default DatasetItem;
