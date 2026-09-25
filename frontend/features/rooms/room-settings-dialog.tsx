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
import type { Locale } from "@/lib/i18n/config";

import { roomCopy } from "./room-copy";

export function RoomSettingsDialog({
  locale,
  invites,
  lobby,
  admissionSettings,
}: Readonly<{
  locale: Locale;
  invites: ReactNode;
  lobby: ReactNode;
  admissionSettings: ReactNode;
}>) {
  const text = roomCopy(locale);

  return (
    <Dialog>
      <DialogTrigger asChild>
        <Button data-testid="room-settings-trigger" type="button" variant="outline">
          <Settings2 aria-hidden="true" className="size-4" />
          {text.settings}
        </Button>
      </DialogTrigger>
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
            <TabsTrigger value="admission">{text.admissionSettings}</TabsTrigger>
            <TabsTrigger value="invites">{text.invites}</TabsTrigger>
          </TabsList>
          <TabsContent className="pt-4" value="requests">
            {lobby}
          </TabsContent>
          <TabsContent className="pt-4" value="admission">
            {admissionSettings}
          </TabsContent>
          <TabsContent className="pt-4" value="invites">
            {invites}
          </TabsContent>
        </Tabs>
      </DialogContent>
    </Dialog>
  );
}
