import type { ComponentProps } from "react";

import { cn } from "@/lib/utils";

function Message({
  align = "start",
  className,
  ...props
}: ComponentProps<"div"> & { align?: "start" | "end" }) {
  return (
    <div
      className={cn("flex items-end gap-2.5", align === "end" && "flex-row-reverse", className)}
      data-align={align}
      data-slot="message"
      {...props}
    />
  );
}

function MessageAvatar({ className, ...props }: ComponentProps<"div">) {
  return (
    <div className={cn("shrink-0 self-end", className)} data-slot="message-avatar" {...props} />
  );
}

function MessageContent({ className, ...props }: ComponentProps<"div">) {
  return (
    <div
      className={cn("flex max-w-[min(100%,38rem)] min-w-0 flex-col gap-1", className)}
      data-slot="message-content"
      {...props}
    />
  );
}

function MessageHeader({ className, ...props }: ComponentProps<"div">) {
  return (
    <div
      className={cn("flex items-center gap-2 text-xs text-muted-foreground", className)}
      data-slot="message-header"
      {...props}
    />
  );
}

function MessageFooter({ className, ...props }: ComponentProps<"div">) {
  return (
    <div
      className={cn("flex flex-wrap items-center gap-1", className)}
      data-slot="message-footer"
      {...props}
    />
  );
}

export { Message, MessageAvatar, MessageContent, MessageHeader, MessageFooter };
