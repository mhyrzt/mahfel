"use client";

import { useState } from "react";
import { CalendarClock, PanelsTopLeft } from "lucide-react";

import { Button } from "@/components/ui/button";
import type { Locale } from "@/lib/i18n/config";
import { messagesFor } from "@/lib/i18n/messages";

import { WorkspaceBrowserMenu } from "./workspace-browser-menu";

export const sidebarIconButtonClassName =
  "flex size-12 items-center justify-center rounded-xl text-muted-foreground transition-colors hover:bg-accent hover:text-accent-foreground focus-visible:bg-accent focus-visible:text-accent-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring";

export function SidebarNavigation({ locale = "en" }: Readonly<{ locale?: Locale }>) {
  const text = messagesFor(locale).shell;
  const [openKind, setOpenKind] = useState<"rooms" | "sessions" | null>(null);
  return (
    <nav className="space-y-2 p-2" aria-label={text.workspaceNavigation}>
      <WorkspaceBrowserMenu
        kind="rooms"
        locale={locale}
        open={openKind === "rooms"}
        onOpenChange={(open) => setOpenKind(open ? "rooms" : null)}
      >
        <Button
          aria-label={text.rooms}
          className={sidebarIconButtonClassName}
          data-testid="browse-rooms"
          size="icon"
          variant="ghost"
        >
          <PanelsTopLeft className="size-5" strokeWidth={1.8} />
        </Button>
      </WorkspaceBrowserMenu>
      <WorkspaceBrowserMenu
        kind="sessions"
        locale={locale}
        open={openKind === "sessions"}
        onOpenChange={(open) => setOpenKind(open ? "sessions" : null)}
      >
        <Button
          aria-label={text.sessions}
          className={sidebarIconButtonClassName}
          data-testid="browse-sessions"
          size="icon"
          variant="ghost"
        >
          <CalendarClock className="size-5" strokeWidth={1.8} />
        </Button>
      </WorkspaceBrowserMenu>
    </nav>
  );
}
