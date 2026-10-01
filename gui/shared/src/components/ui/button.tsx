import * as React from "react";
import { Loader2 } from "lucide-react";
import { Slot } from "@radix-ui/react-slot";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "shared/src/lib/utils";

const buttonVariants = cva(
  "inline-flex items-center flex-nowrap min-w-fit tracking-wider !leading-none justify-center gap-2 whitespace-nowrap rounded-md text-xs font-semibold uppercase transition-all focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-brand-sky/40 focus-visible:ring-offset-1 focus-visible:ring-offset-background disabled:pointer-events-none disabled:opacity-40 disabled:cursor-not-allowed [&_svg]:pointer-events-none [&_svg]:size-3.5 [&_svg]:shrink-0",
  {
    variants: {
      variant: {
        default:
          "bg-primary-700 text-white hover:bg-primary-600 border border-primary-500/30 hover:border-primary-400/50 shadow-sm active:scale-[0.98]",
        destructive:
          "border border-danger-700/50 bg-danger-50 dark:bg-danger-950/40 text-danger-700 dark:text-danger-300 hover:bg-danger-100 dark:hover:bg-danger-900/60 hover:text-danger-800 dark:hover:text-danger-100 hover:border-danger-600/70 shadow-sm active:scale-[0.98]",
        outline:
          "border border-ink/20 text-ink/90 bg-transparent hover:bg-ink/10 hover:text-ink hover:border-ink/30 shadow-sm active:scale-[0.98]",
        outline_destructive:
          "border border-danger-700/60 text-danger-700 dark:text-danger-300 bg-transparent hover:bg-danger-50 dark:bg-danger-950/40 hover:text-danger-800 dark:hover:text-danger-200 hover:border-danger-600 active:scale-[0.98]",
        secondary:
          "bg-ink/10 text-brand-snow hover:bg-ink/15 border border-ink/15 hover:border-ink/25 shadow-sm active:scale-[0.98]",
        ghost: "text-ink/80 hover:text-ink hover:bg-ink/10 active:scale-[0.98]",
        success:
          "border border-success-700/50 bg-success-50 dark:bg-success-950/40 text-success-700 dark:text-success-300 hover:bg-success-100 dark:hover:bg-success-900/60 hover:text-success-800 dark:hover:text-success-100 hover:border-success-600/70 shadow-sm active:scale-[0.98]",
        icon_destructive:
          "flex text-danger-700 dark:text-danger-400 border border-danger-700/40 bg-danger-50 dark:bg-danger-950/20 hover:bg-danger-100 dark:hover:bg-danger-900/50 hover:text-danger-800 dark:hover:text-danger-200 active:scale-[0.98] p-1.5",
        link: "!px-0 flex-nowrap normal-case text-brand-sky underline-offset-4 hover:underline hover:text-ink active:scale-100 transition-colors",
      },
      size: {
        default: "h-8 px-3.5 py-1 text-xs",
        sm: "h-7 rounded-sm px-2.5 text-xs",
        xs: "h-6 rounded-sm px-2 text-xs",
        lg: "h-10 rounded-md px-6 text-sm font-bold",
        icon: "h-8 w-8 rounded-md p-0 flex items-center justify-center",
        icon_sm: "h-7 w-7 rounded-md p-0 flex items-center justify-center",
      },
      policy: {
        default: "",
        permission: "bg-success-600/20 border-success-700 text-success-800 dark:text-success-100 hover:bg-success-600/30",
        obligation: "bg-warn-700/20 border-warn-700 text-warn-800 dark:text-warn-100 hover:bg-warn-700/30",
        prohibition: "bg-danger-600/20 border-danger-700 text-danger-800 dark:text-danger-100 hover:bg-danger-600/30",
      },
    },
    defaultVariants: {
      variant: "default",
      size: "default",
      policy: "default",
    },
  },
);

export type ButtonSizes = VariantProps<typeof buttonVariants>["size"];

export interface ButtonProps
  extends React.ButtonHTMLAttributes<HTMLButtonElement>,
    VariantProps<typeof buttonVariants> {
  asChild?: boolean;
  isLoading?: boolean;
}

/**
 * Displays a button or a component that looks like a button.
 */
const Button = React.forwardRef<HTMLButtonElement, ButtonProps>(
  ({ className, variant, size, policy, asChild = false, isLoading, ...props }, ref) => {
    const Comp = asChild ? Slot : "button";
    return (
      <Comp
        className={cn(buttonVariants({ variant, size, policy, className }))}
        ref={ref}
        disabled={isLoading || props.disabled}
        {...props}
      >
        {isLoading && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
        {props.children}
      </Comp>
    );
  },
);
Button.displayName = "Button";

export { Button, buttonVariants };
