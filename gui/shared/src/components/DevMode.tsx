import React, {useContext, useEffect, useMemo, useState} from "react"
import {GlobalInfoContext, GlobalInfoContextType} from "shared/src/context/GlobalInfoContext";

const randomThemes = [
    { border: "border-red-500", bg: "bg-red-500" },
    { border: "border-orange-500", bg: "bg-orange-500" },
    { border: "border-amber-500", bg: "bg-amber-500" },
    { border: "border-yellow-400", bg: "bg-yellow-400" },
    { border: "border-lime-400", bg: "bg-lime-400" },
    { border: "border-green-500", bg: "bg-green-500" },
    { border: "border-emerald-500", bg: "bg-emerald-500" },
    { border: "border-teal-400", bg: "bg-teal-400" },
    { border: "border-cyan-400", bg: "bg-cyan-400" },
    { border: "border-sky-400", bg: "bg-sky-400" },
    { border: "border-blue-400", bg: "bg-blue-400" },
    { border: "border-indigo-400", bg: "bg-indigo-400" },
    { border: "border-violet-400", bg: "bg-violet-400" },
    { border: "border-purple-400", bg: "bg-purple-400" },
    { border: "border-fuchsia-400", bg: "bg-fuchsia-400" },
    { border: "border-pink-400", bg: "bg-pink-400" },
    { border: "border-rose-400", bg: "bg-rose-400" }
];

const STORAGE_KEY = "dev_mode_gateway_colors";

type ThemeType = { border: string; bg: string };

export const DevMode = () => {
    const {api_gateway} = useContext<GlobalInfoContextType>(GlobalInfoContext);
    const [theme, setTheme] = useState<ThemeType | null>(null);


    useEffect(() => {
        if (!api_gateway) return;

        try {
            const storedData = localStorage.getItem(STORAGE_KEY);
            const colorMap: Record<string, ThemeType> = storedData ? JSON.parse(storedData) : {};
            if (colorMap[api_gateway]) {
                setTheme(colorMap[api_gateway]);
            } else {
                const randomIndex = Math.floor(Math.random() * randomThemes.length);
                const newTheme = randomThemes[randomIndex];
                colorMap[api_gateway] = newTheme;
                localStorage.setItem(STORAGE_KEY, JSON.stringify(colorMap));

                setTheme(newTheme);
            }
        } catch (error) {
            console.warn("Not able to load localstorage", error);
            setTheme(randomThemes[2]); // default
        }
    }, [api_gateway]);

    if (!theme) return null;

    return (
        <div className={`fixed w-full h-full border-b-[4px] ${theme.border} top-0 left-0 z-[10000] pointer-events-none`}>
            <div className={`absolute bottom-[0px] right-[0px] ${theme.bg} text-black text-xs p-2`}>
                GUI Dev mode ON. <br />
                API gateway: {api_gateway}
            </div>
        </div>
    );
}