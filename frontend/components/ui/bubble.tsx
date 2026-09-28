import type { ComponentProps } from "react";

import { cn } from "@/lib/utils";

function Bubble({
  className,
  variant = "secondary",
  align = "start",
  ...props
}: ComponentProps<"div"> & {
  variant?: "default" | "secondary" | "muted" | "tinted" | "outline" | "ghost";
  align?: "start" | "end";
}) {
  return (
    <div
      className={cn(
        "max-w-[min(80%,38rem)] rounded-2xl px-3.5 py-2.5 text-sm leading-relaxed shadow-sm",
        align === "end" && "ms-auto",
        variant === "default" && "bg-primary text-primary-foreground",
        variant === "secondary" && "bg-muted text-foreground",
        variant === "muted" && "bg-muted/60 text-foreground",
        variant === "tinted" && "bg-primary/10 text-foreground",
        variant === "outline" && "border bg-card text-foreground",
        variant === "ghost" && "max-w-none bg-transparent shadow-none",
        className,
      )}
      data-align={align}
      data-slot="bubble"
      data-variant={variant}
      {...props}
    />
  );
}

function BubbleContent({ className, ...props }: ComponentProps<"div">) {
  return (
    <div
      className={cn("whitespace-pre-wrap break-words", className)}
      data-slot="bubble-content"
      {...props}
    />
  );
}

export { Bubble, BubbleContent };
