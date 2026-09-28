"use client";

import { useEffect, useState, type ReactNode } from "react";
import { createPortal } from "react-dom";
import { MessageCircle } from "lucide-react";

import { Button } from "@/components/ui/button";
import { Sheet, SheetContent, SheetHeader, SheetTitle, SheetTrigger } from "@/components/ui/sheet";
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "@/components/ui/tooltip";
import { roomCopy } from "@/features/rooms/room-copy";
import type { Locale } from "@/lib/i18n/config";
import { useIsMobile } from "@/hooks/use-mobile";

export function ConversationSidebar({
  conversation,
  locale,
  open: controlledOpen,
  onOpenChange,
}: Readonly<{
  conversation: ReactNode;
  locale: Locale;
  open?: boolean;
  onOpenChange?: (open: boolean) => void;
}>) {
  const [localOpen, setLocalOpen] = useState(false);
  const [rail, setRail] = useState<HTMLElement | null>(null);
  const isMobile = useIsMobile();
  const open = controlledOpen ?? localOpen;
  const setOpen = onOpenChange ?? setLocalOpen;
  const label = roomCopy(locale).chat;

  useEffect(() => {
    void Promise.resolve().then(() => setRail(document.getElementById("session-sidebar-actions")));
  }, []);

  const trigger = (
    <TooltipProvider>
      <Tooltip>
        <TooltipTrigger asChild>
          <SheetTrigger asChild>
            <Button
              aria-label={label}
              aria-expanded={open}
              className="size-11 shrink-0 rounded-xl"
              size="icon"
              variant={open ? "secondary" : "ghost"}
            >
              <MessageCircle aria-hidden="true" className="size-5" />
            </Button>
          </SheetTrigger>
        </TooltipTrigger>
        <TooltipContent side={locale === "fa" ? "left" : "right"}>{label}</TooltipContent>
      </Tooltip>
    </TooltipProvider>
  );

  let mountedTrigger = <div className="flex justify-end">{trigger}</div>;
  if (isMobile) {
    mountedTrigger = (
      <div className="fixed end-4 top-4 z-40 rounded-xl border bg-sidebar md:hidden">{trigger}</div>
    );
  } else if (rail) {
    mountedTrigger = createPortal(trigger, rail);
  }

  return (
    <Sheet modal={false} open={open} onOpenChange={setOpen}>
      {mountedTrigger}
      <SheetContent
        aria-describedby={undefined}
        className={`flex w-[min(100vw,26rem)] flex-col gap-0 bg-sidebar p-0 sm:max-w-none ${locale === "fa" ? "md:right-16" : "md:left-16"}`}
        dir={locale === "fa" ? "rtl" : "ltr"}
        side={locale === "fa" ? "right" : "left"}
      >
        <SheetHeader className="shrink-0 border-b px-5 py-5 text-start">
          <SheetTitle className="flex items-center gap-2 text-base">
            <MessageCircle aria-hidden="true" className="size-4" />
            {label}
          </SheetTitle>
        </SheetHeader>
        <div className="min-h-0 flex-1 overflow-hidden">{conversation}</div>
      </SheetContent>
    </Sheet>
  );
}
