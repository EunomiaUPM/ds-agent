/*
 * Copyright (C) 2026 - Universidad Politécnica de Madrid - UPM
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program. If not, see <https://www.gnu.org/licenses/>.
 */

import * as React from "react";
import {
  motion,
  useMotionValue,
  useSpring,
  type SpringOptions,
} from "motion/react";
import { cn } from "../../lib/utils";

export type BubbleColors = {
  first: string;
  second: string;
  third: string;
  fourth: string;
  fifth: string;
  sixth: string;
};

export type BubbleBackgroundProps = React.ComponentProps<"div"> & {
  interactive?: boolean;
  transition?: SpringOptions;
  colors?: BubbleColors;
};

// Soft and balanced Eunomia color palette for ethereal ambient bubbling.
const DEFAULT_EUNOMIA_COLORS: BubbleColors = {
  first: "70, 95, 195",   // Soft Eunomia primary indigo
  second: "115, 65, 185", // Gentle Eunomia secondary violet
  third: "65, 170, 220",  // Soft cyan / dataspace sky
  fourth: "45, 55, 140",  // Deep midnight blue
  fifth: "130, 115, 210", // Ethereal lavender
  sixth: "85, 180, 235",  // Interactive glow cyan
};

export const BubbleBackground = React.forwardRef<
  HTMLDivElement,
  BubbleBackgroundProps
>(function BubbleBackground(
  {
    className,
    children,
    interactive = true,
    transition = { stiffness: 100, damping: 20 },
    colors = DEFAULT_EUNOMIA_COLORS,
    ...props
  },
  ref
) {
  const containerRef = React.useRef<HTMLDivElement>(null);
  React.useImperativeHandle(ref, () => containerRef.current as HTMLDivElement);

  const filterId = React.useId().replace(/[^a-zA-Z0-9_-]/g, "_");

  const mouseX = useMotionValue(0);
  const mouseY = useMotionValue(0);
  const springX = useSpring(mouseX, transition);
  const springY = useSpring(mouseY, transition);

  const rectRef = React.useRef<DOMRect | null>(null);
  const rafIdRef = React.useRef<number | null>(null);

  React.useLayoutEffect(() => {
    const updateRect = () => {
      if (containerRef.current) {
        rectRef.current = containerRef.current.getBoundingClientRect();
      }
    };

    updateRect();

    const el = containerRef.current;
    const ro = new ResizeObserver(updateRect);
    if (el) ro.observe(el);

    window.addEventListener("resize", updateRect);
    window.addEventListener("scroll", updateRect, { passive: true });

    return () => {
      ro.disconnect();
      window.removeEventListener("resize", updateRect);
      window.removeEventListener("scroll", updateRect);
    };
  }, []);

  React.useEffect(() => {
    if (!interactive) return;

    const el = containerRef.current;
    if (!el) return;

    const handleMouseMove = (e: MouseEvent) => {
      const rect = rectRef.current;
      if (!rect) return;
      const centerX = rect.left + rect.width / 2;
      const centerY = rect.top + rect.height / 2;

      if (rafIdRef.current != null) cancelAnimationFrame(rafIdRef.current);
      rafIdRef.current = requestAnimationFrame(() => {
        mouseX.set(e.clientX - centerX);
        mouseY.set(e.clientY - centerY);
      });
    };

    el.addEventListener("mousemove", handleMouseMove as EventListener, {
      passive: true,
    });
    return () => {
      el.removeEventListener("mousemove", handleMouseMove as EventListener);
      if (rafIdRef.current != null) cancelAnimationFrame(rafIdRef.current);
    };
  }, [interactive, mouseX, mouseY]);

  return (
    <div
      ref={containerRef}
      data-slot="bubble-background"
      className={cn(
        "relative size-full overflow-hidden bg-gradient-to-br from-[#0b0b1c] via-[#100f28] to-[#080816]",
        className
      )}
      {...props}
    >
      <style>
        {`
          :root {
            --bubble-first: ${colors.first};
            --bubble-second: ${colors.second};
            --bubble-third: ${colors.third};
            --bubble-fourth: ${colors.fourth};
            --bubble-fifth: ${colors.fifth};
            --bubble-sixth: ${colors.sixth};
          }
        `}
      </style>

      <svg
        xmlns="http://www.w3.org/2000/svg"
        className="absolute top-0 left-0 w-0 h-0 pointer-events-none"
        aria-hidden="true"
      >
        <defs>
          <filter id={`goo-${filterId}`}>
            <feGaussianBlur in="SourceGraphic" stdDeviation="16" result="blur" />
            <feColorMatrix
              in="blur"
              mode="matrix"
              values="1 0 0 0 0  0 1 0 0 0  0 0 1 0 0  0 0 0 18 -8"
              result="goo"
            />
            <feBlend in="SourceGraphic" in2="goo" />
          </filter>
        </defs>
      </svg>

      <div
        className="absolute inset-0 pointer-events-none opacity-80"
        style={{ filter: `url(#goo-${filterId}) blur(40px)` }}
      >
        <motion.div
          className="absolute rounded-full size-[80%] top-[10%] left-[10%] mix-blend-hard-light bg-[radial-gradient(circle_at_center,rgba(var(--bubble-first),0.65)_0%,rgba(var(--bubble-first),0)_55%)]"
          animate={{ y: [-40, 40, -40] }}
          transition={{ duration: 25, ease: "easeInOut", repeat: Infinity }}
          style={{ transform: "translateZ(0)", willChange: "transform" }}
        />

        <motion.div
          className="absolute inset-0 flex justify-center items-center origin-[calc(50%-400px)]"
          animate={{ rotate: 360 }}
          transition={{
            duration: 22,
            ease: "linear",
            repeat: Infinity,
            repeatType: "loop",
          }}
          style={{ transform: "translateZ(0)", willChange: "transform" }}
        >
          <div className="rounded-full size-[80%] top-[10%] left-[10%] mix-blend-hard-light bg-[radial-gradient(circle_at_center,rgba(var(--bubble-second),0.65)_0%,rgba(var(--bubble-second),0)_55%)]" />
        </motion.div>

        <motion.div
          className="absolute inset-0 flex justify-center items-center origin-[calc(50%+400px)]"
          animate={{ rotate: 360 }}
          transition={{ duration: 36, ease: "linear", repeat: Infinity }}
          style={{ transform: "translateZ(0)", willChange: "transform" }}
        >
          <div className="absolute rounded-full size-[80%] bg-[radial-gradient(circle_at_center,rgba(var(--bubble-third),0.65)_0%,rgba(var(--bubble-third),0)_55%)] mix-blend-hard-light top-[calc(50%+200px)] left-[calc(50%-500px)]" />
        </motion.div>

        <motion.div
          className="absolute rounded-full size-[80%] top-[10%] left-[10%] mix-blend-hard-light bg-[radial-gradient(circle_at_center,rgba(var(--bubble-fourth),0.65)_0%,rgba(var(--bubble-fourth),0)_55%)] opacity-60"
          animate={{ x: [-40, 40, -40] }}
          transition={{ duration: 32, ease: "easeInOut", repeat: Infinity }}
          style={{ transform: "translateZ(0)", willChange: "transform" }}
        />

        <motion.div
          className="absolute inset-0 flex justify-center items-center origin-[calc(50%_-_800px)_calc(50%_+_200px)]"
          animate={{ rotate: 360 }}
          transition={{ duration: 24, ease: "linear", repeat: Infinity }}
          style={{ transform: "translateZ(0)", willChange: "transform" }}
        >
          <div className="absolute rounded-full size-[160%] mix-blend-hard-light bg-[radial-gradient(circle_at_center,rgba(var(--bubble-fifth),0.6)_0%,rgba(var(--bubble-fifth),0)_55%)] top-[calc(50%-80%)] left-[calc(50%-80%)]" />
        </motion.div>

        {interactive && (
          <motion.div
            className="absolute rounded-full size-full mix-blend-hard-light bg-[radial-gradient(circle_at_center,rgba(var(--bubble-sixth),0.65)_0%,rgba(var(--bubble-sixth),0)_50%)] opacity-60"
            style={{
              x: springX,
              y: springY,
              transform: "translateZ(0)",
              willChange: "transform",
            }}
          />
        )}
      </div>

      {children}
    </div>
  );
});
