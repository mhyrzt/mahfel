import type { ReactNode } from "react";
import { Bell } from "lucide-react";

import { Avatar, AvatarFallback } from "@/components/ui/avatar";
import { Button } from "@/components/ui/button";
import { Separator } from "@/components/ui/separator";
import { SidebarActions } from "./sidebar-actions";
import { SidebarNavigation } from "./sidebar-navigation";

export function AppShell({ children }: Readonly<{ children: ReactNode }>) {
  return (
    <div className="min-h-screen bg-muted/40">
      <aside className="fixed inset-y-0 hidden w-16 flex-col bg-background lg:flex">
        <div className="flex h-16 items-center justify-center text-lg font-semibold tracking-tight">
          M
        </div>
        <Separator />
        <SidebarNavigation />
        <SidebarActions />
      </aside>
      <div className="lg:pl-16">
        <header className="sticky top-0 z-10 flex h-16 items-center justify-between border-b bg-background/95 px-4 backdrop-blur lg:px-8">
          <p className="text-lg font-semibold tracking-tight">Mahfel</p>
          <div className="flex items-center gap-2">
            <Button aria-label="Notifications" size="icon" variant="ghost">
              <Bell className="size-4" />
            </Button>
            <Avatar className="size-8">
              <AvatarFallback className="bg-primary text-xs font-semibold text-primary-foreground">
                R
              </AvatarFallback>
            </Avatar>
          </div>
        </header>
        <main className="mx-auto w-full max-w-7xl p-4 lg:p-8">{children}</main>
      </div>
    </div>
  );
}
