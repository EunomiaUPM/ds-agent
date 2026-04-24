import React from 'react';
import { useState } from "react";
import Heading from 'shared/src/components/ui/heading';
import { Badge } from './badge';
import { FormatDate } from './format-date';
import { Link } from '@tanstack/react-router';

const DatasetItem = ({ date, title, description, prevRoute, datasetId, ownDataset }) => {
    const [showMore, setShowMore] = useState(false);
    const toggleDdatasetDetails = () => {
        setShowMore((prevState) => !prevState); // Alterna entre true y false,
        // cogiendo de base el estado. Si es false lo convierte a true, y viceversa
    };
    return (

        <div className="  h-full dataset-item-container bg-background-200/15 hover:bg-background-200/40 border rounded-md border-white/10 flex flex-col p-3 gap-1">
            <Link
                to={ownDataset ? "/catalog/$prevRoute/dataset/$datasetId"
                    : "/catalog/participant/$prevRoute/dataset/$datasetId"}
                params={{
                    prevRoute: prevRoute!,
                    datasetId: datasetId!,
                }}
            >
                <div className="dataset-header flex justify-between items-center">
                    <Heading level="h5" className="!mb-0 font-bold underline-offset-2 hover:underline"> {title} Dataset </Heading>

                </div>
            </Link>
            <p className="dataset-item-description text-sm">{description || "Description of the dataset."}</p>
            <div className="h-2"></div>
            <div className="dataset-item-policies flex flex-col gap-2">
                <div className="dataset-item-details-summary flex gap-6 items-start ">
                    <div className="policies-summary-container flex flex-col gap-2 text-xs uppercase">
                        <span>3 policies </span>
                        <Badge variant="detail" className={"text-2xs " + (showMore ? `flex` : `hidden`)} > Policy title</Badge>
                        <Badge variant="detail" className={"text-2xs " + (showMore ? `flex` : `hidden`)} > Research Juner Trial Access (large title)</Badge>
                    </div>
                    <div className="distributions-summary-container flex flex-col gap-2 text-xs uppercase">
                        <span>2 distributions </span>
                        <Badge variant="detail" className={"text-2xs " + (showMore ? `flex` : `hidden`)} > Distribution title</Badge>
                    </div>
                    <button className="font-bold text-2xs underline underline-offset-2" onClick={toggleDdatasetDetails}>{(showMore ? "Show less" : "Show more")} </button>
                </div>
            </div>
            <div className="catalog-dates-updated flex gap-1 text-xs h-2 text-right justify-end mt-1 mb-1 italic text-muted-foreground">
                <p>Issued at:</p>
                <p> <FormatDate date={date} />  </p>
            </div>
        </div>

    );
};

export default DatasetItem;