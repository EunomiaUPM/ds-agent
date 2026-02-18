import * as React from "react";
import { cva } from "class-variance-authority";
import { cn } from "shared/src/lib/utils";
import dayjs from "dayjs";
import Heading from "../heading";
import { RoleType, mapRoleToVariant } from "./message-log";

export const roleVariants = cva("", {
  variants: {
    variant: {
      default: "bg-brand-snow/10 border border-brand-snow",
      Provider: "bg-roles-provider/10 border border-roles-provider",
      Consumer: "bg-roles-consumer/10 border border-roles-consumer",
    },
  },
  defaultVariants: {
    variant: "default",
  },
});

export const RoleHeader = React.forwardRef<
  HTMLDivElement,
  React.HTMLAttributes<HTMLDivElement> & { from: RoleType }
>(({ from, className, ...props }, ref) => {
  const styleVariant = mapRoleToVariant(from);
  const isLeft = styleVariant === "Provider";

  return (
    <div
      ref={ref}
      className={cn("flex w-full", isLeft ? "justify-start" : "justify-end", className)}
      {...props}
    >
      <div
        className={cn(
          roleVariants({ variant: styleVariant }),
          "w-fit max-w-[640px] uppercase text-18 px-4 py-1 font-medium rounded-t-sm border-none",
          isLeft ? "ml-2 text-roles-provider" : "mr-2 text-roles-consumer",
        )}
      >
        {from}
      </div>
    </div>
  );
});
RoleHeader.displayName = "RoleHeader";

export const MessageBody = React.forwardRef<
  HTMLDivElement,
  React.HTMLAttributes<HTMLDivElement> & { variant: RoleType }
>(({ className, children, variant, ...props }, ref) => {
  const styleVariant = mapRoleToVariant(variant!);

  return (
    <div
      ref={ref}
      className={cn(
        roleVariants({ variant: styleVariant }),
        "w-full max-w-[640px] px-4 py-3 rounded-md rounded-b-xl border flex flex-col gap-2",
        className,
      )}
      {...props}
    >
      {children}
    </div>
  );
});
MessageBody.displayName = "MessageBody";

export const MessageTitle = React.forwardRef<
  HTMLHeadingElement,
  React.HTMLAttributes<HTMLHeadingElement>
>(({ className, children, ...props }, ref) => (
  <Heading
    level="h4"
    ref={ref}
    className={cn("mb-0 !text-[20px] !text-foreground-400", className)}
    {...props}
  >
    {children}
  </Heading>
));
MessageTitle.displayName = "MessageTitle";

export const MessageTimestamp = React.forwardRef<
  HTMLParagraphElement,
  { created_at: string } & React.HTMLAttributes<HTMLParagraphElement>
>(({ created_at, className, ...props }, ref) => (
  <p ref={ref} className={cn("text-foreground mb-3", className)} {...props}>
    <i>{dayjs(created_at).format("DD/MM/YYYY - HH:mm")}</i>
  </p>
));
MessageTimestamp.displayName = "MessageTimestamp";
