import * as React from "react";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "shared/src/lib/utils";

/**
 * Visual variants for Card container component.
 */
const cardVariants = cva("rounded-xl text-card-foreground transition-all duration-200", {
  variants: {
    variant: {
      default: "border border-ink/10 bg-card/60 backdrop-blur-sm shadow-sm",
      interactive:
        "border border-ink/10 bg-card/60 backdrop-blur-sm shadow-sm hover:border-ink/20 hover:bg-card/90 hover:shadow-md cursor-pointer",
      glass: "border border-ink/15 bg-background-600/40 backdrop-blur-md shadow-lg",
      metric:
        "border border-ink/10 bg-gradient-to-br from-card/80 to-background-800/80 shadow-sm relative overflow-hidden",
      outline: "border border-ink/15 bg-transparent",
      subtle: "border border-ink/5 bg-background-200/10",
      ghost: "border-transparent bg-transparent",
    },
  },
  defaultVariants: {
    variant: "default",
  },
});

export interface CardProps
  extends React.HTMLAttributes<HTMLDivElement>,
    VariantProps<typeof cardVariants> {}

/**
 * Consolidated enterprise Card container component.
 */
const Card = React.forwardRef<HTMLDivElement, CardProps>(
  ({ className, variant, ...props }, ref) => (
    <div
      ref={ref}
      data-slot="card"
      className={cn(cardVariants({ variant }), className)}
      {...props}
    />
  ),
);
Card.displayName = "Card";

const CardHeader = React.forwardRef<HTMLDivElement, React.HTMLAttributes<HTMLDivElement>>(
  ({ className, ...props }, ref) => (
    <div
      ref={ref}
      data-slot="card-header"
      className={cn("flex flex-col space-y-1.5 p-5", className)}
      {...props}
    />
  ),
);
CardHeader.displayName = "CardHeader";

const CardTitle = React.forwardRef<HTMLHeadingElement, React.HTMLAttributes<HTMLHeadingElement>>(
  ({ className, ...props }, ref) => (
    <h3
      ref={ref}
      data-slot="card-title"
      className={cn(
        "font-semibold text-foreground leading-tight tracking-tight text-base sm:text-lg",
        className,
      )}
      {...props}
    />
  ),
);
CardTitle.displayName = "CardTitle";

const CardDescription = React.forwardRef<
  HTMLParagraphElement,
  React.HTMLAttributes<HTMLParagraphElement>
>(({ className, ...props }, ref) => (
  <p
    ref={ref}
    data-slot="card-description"
    className={cn("text-xs sm:text-sm text-muted-foreground leading-relaxed", className)}
    {...props}
  />
));
CardDescription.displayName = "CardDescription";

const CardContent = React.forwardRef<HTMLDivElement, React.HTMLAttributes<HTMLDivElement>>(
  ({ className, ...props }, ref) => (
    <div ref={ref} data-slot="card-content" className={cn("p-5 pt-0", className)} {...props} />
  ),
);
CardContent.displayName = "CardContent";

const CardFooter = React.forwardRef<HTMLDivElement, React.HTMLAttributes<HTMLDivElement>>(
  ({ className, ...props }, ref) => (
    <div
      ref={ref}
      data-slot="card-footer"
      className={cn("flex items-center p-5 pt-0 border-t border-ink/5 mt-auto", className)}
      {...props}
    />
  ),
);
CardFooter.displayName = "CardFooter";

export { Card, CardHeader, CardFooter, CardTitle, CardDescription, CardContent, cardVariants };
