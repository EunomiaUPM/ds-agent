import React from 'react';
import Heading from 'shared/src/components/ui/heading';
import { Link } from "@tanstack/react-router";

interface CatalogItemProps {
  date?: string;
  datasetNumber?: number;
  organizationName?: string;
  id?: string;
}

const CatalogItem = ({ date = "N/A", datasetNumber = 0, organizationName = "Unknown", id = "" }: CatalogItemProps) => {
  return (
    <Link to="/catalog/participant/$participantId" params={{ participantId: id}}>
        <div className="catalog-card bg-background-200/15  hover:bg-background-200/50 transition-all border rounded-md border-white/20 flex flex-col p-4 gap-3 max-w-md">
        <div className="catalog-dates-container flex gap-3 text-sm tracking-wide">
          <div className="catalog-dates-created flex gap-1">
            <p>Created at:</p>
            <p>{date}</p>
          </div>
          {/* <p> | </p>
           <div className="catalog-dates-updated flex gap-1">
            <p>Updated at:</p> 
            <p>02/08/2025</p>
          </div> */}
        </div>
        <div className="catalog-text-container">
           
          <Heading level="h3" className="mb-3 underline-offset-2 hover:underline">Participant catalog </Heading>
          
          <p className="mb-2 ">This is the catalog of <span className='capitalize'>{organizationName}</span>, who is also part of this dataspace. 
            Click on the catalog name to see the datasets and dataservice they offer.
          </p>
        </div>
        <div className="catalog-participant-container flex gap-2 justify-start">
          <img className="rounded-full bg-violet-600 h-6 aspect-square"></img>
          <Heading level="h4" className='capitalize'> {organizationName} </Heading>
        </div>
        <div className="catalog-items-container flex justify-end gap-2 text-sm italic">
          <p> 1 Dataservice </p>
          <p> {datasetNumber} Datasets </p>
        </div>
      </div>
   </Link>
  );
};

export default CatalogItem;
