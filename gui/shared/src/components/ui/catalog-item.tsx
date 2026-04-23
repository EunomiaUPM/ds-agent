import React from "react";
import Heading from "shared/src/components/ui/heading";

const CatalogItem = () => {
  return (
    <div>
      <div className="catalog-card bg-brand-sky/5 border rounded-md border-white/20 flex flex-col p-4 gap-3">
        <div className="catalog-dates-container flex gap-3 text-sm tracking-wide">
          <div className="catalog-dates-created flex gap-1">
            <p>Created at:</p>
            <p>02/08/2023</p>
          </div>
          <p> | </p>
          <div className="catalog-dates-updated flex gap-1">
            <p>Updated at:</p>
            <p>02/08/2025</p>
          </div>
        </div>
        <div className="catalog-text-container">
          <Heading level="h3" className="mb-1">
            Catalog title
          </Heading>
          <p>
            Descripción del catálogo pues sería una frase corta que dice de qué va el contenido del
            catálogo.
          </p>
        </div>
        <div className="catalog-participant-container flex gap-2 justify-start">
          <img className="rounded-full bg-violet-600 h-6 aspect-square"></img>
          <Heading level="h4"> Organization name</Heading>
        </div>
        <div className="catalog-items-container flex justify-end gap-2 text-sm italic">
          <p> 2 Dataservices </p>
          <p> 4 Datasets </p>
        </div>
      </div>
    </div>
  );
};

export default CatalogItem;
