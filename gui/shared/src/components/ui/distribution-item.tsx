import React from 'react';
import { useState } from "react";
import Heading from 'shared/src/components/ui/heading';
import { Badge } from './badge';

const DistributionItem = () => {
    return (
        <div className='distribution-container min-w-[500px] max-w-[600px] h-full dataset-item-container bg-brand-sky/5 border rounded-md border-white/20 flex flex-col p-4 gap-3'>
            <div className="distribution-text">
                <Heading level="h3" className="mb-1">Distribution title</Heading>
                <p>Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.</p>
            </div>
            <div className="distribution-table ">
                <div className="grid grid-cols-2 border-y border-white/20 py-2 gap-x-5">
                    <span className="font-bold">Associated Connector:</span>
                    <span>Connector title</span>
                </div>
                <div className="grid grid-cols-2 border-b border-white/20 py-2 gap-x-5">
                    <span className="font-bold">Associated Policy:</span>
                    <span>Connector title</span>
                </div>
            </div>
        </div>
    );
};

export default DistributionItem;