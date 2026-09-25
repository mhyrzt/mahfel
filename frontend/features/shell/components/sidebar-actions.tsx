"use client";

import { useState } from "react";
import { CalendarPlus, Plus, SquarePlus } from "lucide-react";

import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import type { Locale } from "@/lib/i18n/config";
import { messagesFor } from "@/lib/i18n/messages";

import { CreateWorkspaceDialog } from "./create-workspace-dialog";
import { sidebarIconButtonClassName } from "./sidebar-navigation";

export function SidebarActions({ locale = "en" }: Readonly<{ locale?: Locale }>) {
  const text = messagesFor(locale).shell;
  const [openKind, setOpenKind] = useState<"room" | "session" | null>(null);
  return (
    <div className="px-2 pb-2">
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <Button
            aria-label={text.create}
            className={sidebarIconButtonClassName}
            size="icon"
            variant="ghost"
          >
            <Plus className="size-5" strokeWidth={1.8} />
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="start" side="right">
          <DropdownMenuItem onSelect={() => setOpenKind("room")}>
            <SquarePlus aria-hidden="true" />
            {text.createRoom}
          </DropdownMenuItem>
          <DropdownMenuItem onSelect={() => setOpenKind("session")}>
            <CalendarPlus aria-hidden="true" />
            {text.scheduleSession}
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>
      <CreateWorkspaceDialog
        kind="room"
        locale={locale}
        onOpenChange={(open) => !open && setOpenKind(null)}
        open={openKind === "room"}
      />
      <CreateWorkspaceDialog
        kind="session"
        locale={locale}
        onOpenChange={(open) => !open && setOpenKind(null)}
        open={openKind === "session"}
      />
    </div>
  );
}
