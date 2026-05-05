import React, { useEffect, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { X } from "lucide-react";
import { Button } from "./ui/button";
import Heading from "./ui/heading"

type Props = {
    open: boolean;
    onClose: () => void;
    anchorRef?: React.RefObject<HTMLElement>;
    width?: number | string;
    align?: "center" | "left";
    children?: React.ReactNode;
    content: React.ReactNode;
    title: String
};

export default function WizardDialog({ open, onClose, anchorRef, width = 460, align = "center", content, title, children }: Props) {
    const ref = useRef<HTMLDivElement | null>(null);
    const [pos, setPos] = useState({ left: 0, top: 0, tailLeft: 24, w: typeof width === "number" ? width : 560 });
    const [measured, setMeasured] = useState(false);

    useEffect(() => {
        if (!open) return;

        function update() {
            if (!anchorRef?.current || !ref.current) return;
            const a = anchorRef.current.getBoundingClientRect();
            const vw = window.innerWidth;
            const boxW = Math.min(typeof width === "number" ? width : 460, vw - 32);
            let left;
            if (align === "left") {
                // align left edge of tooltip close to anchor left
                left = Math.max(8, a.left - 8);
                // ensure tooltip doesn't overflow right
                left = Math.min(left, vw - boxW - 8);
            } else {
                left = a.left + a.width / 2 - boxW / 2;
                left = Math.max(8, Math.min(left, vw - boxW - 8));
            }
            const tooltipHeight = ref.current.getBoundingClientRect().height || 80;
            const top = Math.max(8, a.top - tooltipHeight - 12);
            let tailLeft;
            if (align === "left") {
                // tail close to left side of tooltip
                tailLeft = 20;
            } else {
                tailLeft = Math.max(12, Math.min(a.left + a.width / 2 - left - 12, boxW - 24));
            }
            setPos({ left, top, tailLeft, w: boxW });
            setMeasured(true);
        }

        update();
        window.addEventListener("resize", update);
        window.addEventListener("scroll", update, true);
        return () => {
            window.removeEventListener("resize", update);
            window.removeEventListener("scroll", update, true);
        };
    }, [open, anchorRef, width]);

    if (!open) return null;

    const portalRoot = typeof window !== "undefined" ? document.body : null;

        // render portal always so the element mounts (needed for measurement),
        // but keep it visually hidden/offscreen until we've computed pos
        const portalContent = (
            <div
                ref={ref}
                className="fixed z-[9999] pointer-events-none"
                style={ measured ? { left: pos.left, top: pos.top, width: pos.w } : { left: -9999, top: -9999, width: pos.w, opacity: 0 }}
            >
                <div className={`relative mx-4 bg-background-300 border border-secondary-800 text-white p-3 rounded-md shadow-lg pointer-events-auto transition-opacity transition-transform duration-200 ${measured ? 'opacity-100 translate-y-0' : 'opacity-0 -translate-y-2'}`}>
                <div className="flex items-start gap-3">
                    <div className="flex-1 text-sm leading-snug">
                        <Heading level="h5">{title}</Heading>
                        <p className="text-sm">{content}</p>
                        {children}</div>
                    {/* <div className="shrink-0">
                        <button
                            type="button"
                            aria-label="Close"
                            onClick={onClose}
                            className="inline-flex items-center justify-center p-1 rounded hover:bg-white/10 transition-colors"
                        >
                            <X className="h-4 w-4" />
                        </button>
                    </div> */}
                </div>
                {/* tail */}
                {/* tail: outer purple border + inner yellow triangle to simulate a bordered tail */}
                <div style={{ position: "absolute", left: pos.tailLeft - 2, bottom: -14 }}>
                    {/* outer (border) triangle - slightly larger and purple */}
                    <div style={{ width: 0, height: 0, borderLeft: "14px solid transparent", borderRight: "14px solid transparent", borderTop: "14px solid #62388E" }} />
                </div>
                <div style={{ position: "absolute", left: pos.tailLeft, bottom: -12 }}>
                    {/* inner triangle matching tooltip background */}
                    <div style={{ width: 0, height: 0, borderLeft: "12px solid transparent", borderRight: "12px solid transparent", borderTop: "12px solid #2E3356" }} />
                </div>
            </div>
        </div>
    );

    if (portalRoot) return createPortal(portalContent, portalRoot);
    return portalContent;
}
