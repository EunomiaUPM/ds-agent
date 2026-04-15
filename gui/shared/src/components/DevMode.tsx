import React, {useContext} from "react"
import {GlobalInfoContext, GlobalInfoContextType} from "shared/src/context/GlobalInfoContext";

export const DevMode = () => {
    const {api_gateway} = useContext<GlobalInfoContextType>(GlobalInfoContext);
    return (<div className="absolute w-full h-full border-b-[4px] border-amber-600 top-0 left-0 z-[10000] pointer-events-none">
        <div className="absolute bottom-[0px] right-[0px] bg-amber-600 text-black text-xs p-2">
            GUI Dev mode ON. <br />
            API gateway: {api_gateway}
        </div>
    </div>)
}