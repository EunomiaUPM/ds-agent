import * as React from "react";
import { useRouterState } from "@tanstack/react-router";
import { Search } from "lucide-react";
import { cn } from "shared/src/lib/utils";

/**
 * Input component with specialized behavior for search placeholders based on route.
 */
const Input = React.forwardRef<HTMLInputElement, React.ComponentProps<"input">>(
  ({ className, type, ...props }, ref) => {
    const routerState = useRouterState();
    let pathsArray = routerState.location.pathname.split("/");

    pathsArray.map((path, index) => {
      path.includes("urn") ? pathsArray.splice(index) : "";

      return pathsArray;
    });
    pathsArray.splice(0, 1);
    let stringPathsArray = JSON.stringify(pathsArray);
    // console.log(pathsArray, "pathsArray2");
    let pathFormat = stringPathsArray
      .replace(/["[\]]/g, "")
      .split("-")
      .join(" ");
    let placeHolderText;

    if (type === "search") {
      placeHolderText = "Search for " + pathFormat;
    } else {
      placeHolderText = "Enter text";
    }

    return (
      <div
        className={cn(
          "flex h-7 items-center rounded-sm border-0 border-input bg-white/5 pl-2 text-xs focus-within:ring-1 focus-within:ring-ring/50",
          className,
        )}
      >
        {type === "search" && <Search className="h-4 w-4" />}
        <input
          {...props}
          type="search"
          ref={ref}
          placeholder={placeHolderText}
          className="bg-transparent w-full p-1 placeholder:text-muted-foreground/70 focus-visible:outline-none disabled:cursor-not-allowed disabled:opacity-50"
        />
      </div>
    );
  },
);
Input.displayName = "Input";

export { Input };
