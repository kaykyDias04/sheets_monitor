import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "@/lib/utils";
import type { HTMLAttributes } from "react";
const variants = cva(
  "inline-flex items-center gap-1.5 rounded-md border px-2 py-1 text-xs font-medium",
  {
    variants: {
      variant: {
        default: "border-transparent bg-primary text-primary-foreground",
        secondary: "border-transparent bg-secondary text-secondary-foreground",
        outline: "text-foreground",
        demo: "border-amber-200 bg-amber-50 text-amber-800",
      },
    },
    defaultVariants: { variant: "default" },
  },
);
export function Badge({
  className,
  variant,
  ...props
}: HTMLAttributes<HTMLDivElement> & VariantProps<typeof variants>) {
  return <div className={cn(variants({ variant }), className)} {...props} />;
}
