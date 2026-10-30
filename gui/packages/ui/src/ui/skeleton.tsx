import React from "react";
import { cn } from "../lib/utils";

/**
 * Component for displaying a placeholder skeleton while content is loading.
 */
function Skeleton({ className, ...props }: React.HTMLAttributes<HTMLDivElement>) {
  return <div className={cn("animate-pulse rounded-md bg-primary/10", className)} {...props} />;
}

export { Skeleton };
