import * as React from "react";
import { cva } from "class-variance-authority";
import { cn } from "shared/src/lib/utils";

export type RoleType = "Provider" | "Consumer" | "Business" | "Customer";

export const mapRoleToVariant = (role: RoleType) => {
  if (role === "Provider" || role === "Business") return "Provider";
  if (role === "Consumer" || role === "Customer") return "Consumer";
  return "default";
};

const layoutVariants = cva("my-4 text-sm overflow-hidden", {
  variants: {
    variant: {
      Provider: "pr-[7%]",
      Consumer: "pl-[7%]",
      default: "pr-[7%]",
    },
  },
});

export const MessageLog = React.forwardRef<
  HTMLDivElement,
  React.HTMLAttributes<HTMLDivElement> & { variant: RoleType }
>(({ className, variant, children, ...props }, ref) => {
  const styleVariant = mapRoleToVariant(variant);
  const isLeft = styleVariant === "Provider";

  return (
    <div
      ref={ref}
      className={cn(
        layoutVariants({ variant: styleVariant }),
        "w-full min-h-fit flex flex-col",
        isLeft ? "justify-start items-start" : "justify-end items-end",
        className,
      )}
      {...props}
    >
      {children}
    </div>
  );
});
MessageLog.displayName = "MessageLog";
