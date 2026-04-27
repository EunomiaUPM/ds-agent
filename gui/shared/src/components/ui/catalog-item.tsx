import React from 'react';
import Heading from 'shared/src/components/ui/heading';
import { Link } from "@tanstack/react-router";
import { FormatDate } from "shared/src/components/ui/format-date";

const CatalogItem = ({ date, datasetNumber, organizationName, id, title }) => {
  
  const conditionalCatalogClasses = id === null ? "opacity-65 grayscale cursor-not-allowed"
    : "cursor-pointer";

  return (
   
        <div className={`catalog-card bg-background-200/15  hover:bg-background-200/30 transition-all border rounded-md border-white/10 flex flex-col p-4 gap-3 max-w-lg ${conditionalCatalogClasses}}`}>
        <div className="catalog-dates-container flex gap-3 text-sm tracking-wide">
          <div className="catalog-dates-created flex gap-1">
            <p>Created at:</p>
            <p>
              <FormatDate date={date} />
            </p>
          </div>
          {/* <p> | </p>
           <div className="catalog-dates-updated flex gap-1">
            <p>Updated at:</p> 
            <p>02/08/2025</p>
          </div> */}
        </div>
        <div className="catalog-text-container">
          {id !== null ? (
            <Link to="/catalog/participant/$participantId" params={{ participantId: id}}>
          <Heading level="h3" className="capitalize mb-3 underline-offset-2 hover:underline"> {title ? title : `${organizationName}'s Catalog for Demo`} </Heading>
            </Link>
          ) : (
            <Heading level="h3" className="capitalize mb-3 underline-offset-2 hover:underline"> {title ? title : `${organizationName}'s Catalog for Demo`} </Heading>
          )}
          <p className="mb-2 ">This is the catalog of <span className='capitalize'>{organizationName}</span>, who is also part of this dataspace. 
            Click on the catalog name to see the datasets and dataservice they offer.
          </p>
        </div>
        <div className="catalog-participant-container flex gap-2 justify-start">
          <img className={` rounded-full h-6 aspect-square ${organizationName === "provider" ? "bg-violet-700" : "bg-orange-500"}`}></img>
          <Heading level="h4" className='capitalize'> {organizationName} </Heading>
        </div>
        <div className="catalog-items-container flex justify-end gap-2 text-sm italic">
          <p> 1 Dataservice </p>
          <p> {datasetNumber} Datasets </p>
        </div>
      </div>

  );
};

export default CatalogItem;