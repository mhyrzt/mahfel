import type { ComponentProps } from "react";

import { cn } from "@/lib/utils";

function Bubble({ className, ...props }: ComponentProps<"div">) {
  return (
    <div
      className={cn("rounded-xl border bg-card px-3.5 py-2.5 text-sm shadow-sm", className)}
      data-slot="bubble"
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
