"use client";

import type { ReactNode } from "react";
import { Settings2 } from "lucide-react";

import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "@/components/ui/tooltip";
import type { Locale } from "@/lib/i18n/config";

import { roomCopy } from "./room-copy";

export function RoomSettingsDialog({
  locale,
  invites,
  requests,
  members,
}: Readonly<{
  locale: Locale;
  invites: ReactNode;
  requests: ReactNode;
  members: ReactNode;
}>) {
  const text = roomCopy(locale);

  return (
    <Dialog>
      <TooltipProvider>
        <Tooltip>
          <TooltipTrigger asChild>
            <DialogTrigger asChild>
              <Button
                aria-label={text.settings}
                data-testid="room-settings-trigger"
                size="icon"
                type="button"
                variant="outline"
              >
                <Settings2 aria-hidden="true" className="size-4" />
              </Button>
            </DialogTrigger>
          </TooltipTrigger>
          <TooltipContent>{text.settings}</TooltipContent>
        </Tooltip>
      </TooltipProvider>
      <DialogContent
        className="max-h-[min(85vh,48rem)] max-w-2xl overflow-y-auto"
        data-testid="room-settings-dialog"
      >
        <DialogHeader>
          <DialogTitle>{text.settings}</DialogTitle>
          <DialogDescription>{text.settingsDescription}</DialogDescription>
        </DialogHeader>
        <Tabs defaultValue="requests">
          <TabsList className="h-auto w-full flex-wrap justify-start gap-1">
            <TabsTrigger value="requests">{text.roomRequests}</TabsTrigger>
            <TabsTrigger value="members">{text.members}</TabsTrigger>
            <TabsTrigger value="invites">{text.invites}</TabsTrigger>
          </TabsList>
          <TabsContent className="pt-4" value="requests">
            {requests}
          </TabsContent>
          <TabsContent className="pt-4" value="members">
            {members}
          </TabsContent>
          <TabsContent className="pt-4" value="invites">
            {invites}
          </TabsContent>
        </Tabs>
      </DialogContent>
    </Dialog>
  );
}
