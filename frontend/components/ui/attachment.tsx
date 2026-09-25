import { Slot } from "@radix-ui/react-slot";
import type { ComponentProps } from "react";

import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";

type AttachmentState = "idle" | "uploading" | "processing" | "error" | "done";

function Attachment({
  className,
  state = "done",
  size = "default",
  children,
  ...props
}: ComponentProps<"div"> & {
  state?: AttachmentState;
  size?: "default" | "sm" | "xs";
}) {
  const sizeClass = {
    default: "min-h-16 px-3 py-2.5",
    sm: "min-h-14 px-3 py-2",
    xs: "min-h-11 px-2 py-1.5",
  }[size];
  return (
    <div
      className={cn(
        "group/attachment relative flex min-w-0 items-center gap-3 rounded-lg border bg-card text-card-foreground transition-colors",
        sizeClass,
        state === "error" && "border-destructive/40",
        state === "idle" && "border-dashed",
        (state === "uploading" || state === "processing") && "border-primary/30 bg-primary/5",
        className,
      )}
      data-slot="attachment"
      data-state={state}
      data-size={size}
      {...props}
    >
      {children}
    </div>
  );
}

function AttachmentMedia({ className, ...props }: ComponentProps<"div">) {
  return (
    <div
      className={cn(
        "flex size-9 shrink-0 items-center justify-center rounded-md bg-muted text-muted-foreground [&_svg]:size-4",
        className,
      )}
      data-slot="attachment-media"
      {...props}
    />
  );
}

function AttachmentContent({ className, ...props }: ComponentProps<"div">) {
  return (
    <div className={cn("min-w-0 flex-1", className)} data-slot="attachment-content" {...props} />
  );
}

function AttachmentTitle({ className, ...props }: ComponentProps<"span">) {
  return (
    <span
      className={cn(
        "block truncate text-sm font-medium group-data-[state=uploading]/attachment:animate-pulse group-data-[state=processing]/attachment:animate-pulse",
        className,
      )}
      data-slot="attachment-title"
      {...props}
    />
  );
}

function AttachmentDescription({ className, ...props }: ComponentProps<"span">) {
  return (
    <span
      className={cn("block truncate text-xs text-muted-foreground", className)}
      data-slot="attachment-description"
      {...props}
    />
  );
}

function AttachmentActions({ className, ...props }: ComponentProps<"div">) {
  return (
    <div
      className={cn("relative z-20 flex shrink-0 items-center gap-1", className)}
      data-slot="attachment-actions"
      {...props}
    />
  );
}

function AttachmentAction({ className, ...props }: ComponentProps<typeof Button>) {
  return (
    <Button
      className={cn("size-8 p-0", className)}
      data-slot="attachment-action"
      size="icon"
      type="button"
      variant="ghost"
      {...props}
    />
  );
}

function AttachmentTrigger({
  asChild = false,
  className,
  type,
  ...props
}: ComponentProps<"button"> & { asChild?: boolean }) {
  const Comp = asChild ? Slot : "button";
  return (
    <Comp
      className={cn(
        "absolute inset-0 z-10 rounded-lg outline-none focus-visible:ring-2 focus-visible:ring-ring",
        className,
      )}
      data-slot="attachment-trigger"
      type={asChild ? undefined : (type ?? "button")}
      {...props}
    />
  );
}

export {
  Attachment,
  AttachmentMedia,
  AttachmentContent,
  AttachmentTitle,
  AttachmentDescription,
  AttachmentActions,
  AttachmentAction,
  AttachmentTrigger,
};
