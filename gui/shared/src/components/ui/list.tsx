import * as React from "react";
import { cn } from "shared/src/lib/utils";

/**
 * Wrapper for a list of items.
 */
const List = ({
  className,
  children,
  ...props
}: {
  className?: string;
  children: React.ReactNode;
  props?: any;
}) => {
  return (
    <ul
      className={cn(
        "mt-0 px-2 min-w-fit w-full text-sm flex flex-col justify-center relative bg-brand-sky/5 overflow-auto border border-ink/10 rounded-md",
        className,
      )}
      {...props}
    >
      {children}
    </ul>
  );
};

const ListItem = ({
  className,
  children,
  ...props
}: {
  className?: string;
  children: React.ReactNode;
  props?: any;
}) => {
  return (
    <li
      className={cn(
        "h-fit min-h-9 w-full py-1.5 flex flex-row tracking-wide flex-wrap justify-start items-center gap-2 border-b border-ink/20 last:border-0",
        className,
      )}
      {...props}
    >
      {children}
    </li>
  );
};

const ListItemKey = ({
  className,
  children,
  ...props
}: {
  className?: string;
  children: React.ReactNode;
  props?: any;
}) => {
  return (
    <p className={cn("font-bold basis-[45%] min-w-fit", className)} {...props}>
      {children}
    </p>
  );
};

const ListItemDate = ({
  className,
  children,
  ...props
}: {
  className?: string;
  children: React.ReactNode;
  props?: any;
}) => {
  return (
    <p className={cn("text-foreground-400", className)} {...props}>
      {children}
    </p>
  );
};

export { List, ListItem, ListItemKey, ListItemDate };
