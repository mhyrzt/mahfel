import Link from "next/link";
import { CalendarPlus, Plus, SquarePlus } from "lucide-react";

import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { sidebarIconButtonClassName } from "./sidebar-navigation";
import type { Locale } from "@/lib/i18n/config";
import { messagesFor } from "@/lib/i18n/messages";

export function SidebarActions({ locale = "en" }: Readonly<{ locale?: Locale }>) {
  const text = messagesFor(locale).shell;

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
        <DropdownMenuContent align="start" side="right" sideOffset={8}>
          <DropdownMenuLabel>{text.create}</DropdownMenuLabel>
          <DropdownMenuSeparator />
          <DropdownMenuItem asChild>
            <Link href="/rooms/new">
              <SquarePlus className="size-4" />
              {text.createRoom}
            </Link>
          </DropdownMenuItem>
          <DropdownMenuItem asChild>
            <Link href="/sessions/new">
              <CalendarPlus className="size-4" />
              {text.scheduleSession}
            </Link>
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>
    </div>
  );
}
