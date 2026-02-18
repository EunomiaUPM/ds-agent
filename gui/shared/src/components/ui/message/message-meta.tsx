import * as React from "react";
import { cn } from "shared/src/lib/utils";
import { Badge } from "../badge";

export const MessageMetaContainer = React.forwardRef<
  HTMLUListElement,
  React.HTMLAttributes<HTMLUListElement>
>(({ className, children, ...props }, ref) => (
  <ul ref={ref} className={cn("gap-0", className)} {...props}>
    {children}
  </ul>
));
MessageMetaContainer.displayName = "MessageMetaContainer";

export interface MessageMetaProps extends React.HTMLAttributes<HTMLLIElement> {
  label: string;
  value: string;
}

export const MessageMeta = React.forwardRef<HTMLLIElement, MessageMetaProps>(
  ({ label, value, className, ...props }, ref) => (
    <li
      ref={ref}
      className={cn(
        "min-h-8 flex flex-row flex-wrap gap-1 mb-1 text-white/70 border-none p-0 m-0",
        className,
      )}
      {...props}
    >
      <span className="font-bold max-w-40">{label}</span>
      <Badge className="max-w-full overflow-hidden" variant="info">
        {value}
      </Badge>
    </li>
  ),
);
MessageMeta.displayName = "MessageMeta";
