"use client";

import { useEffect, useState, type CSSProperties, type ReactNode } from "react";
import { createPortal } from "react-dom";
import { FileStack, Link2, ShieldCheck, UserRoundCheck, UsersRound } from "lucide-react";

import { Button } from "@/components/ui/button";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import {
  Sidebar,
  SidebarContent,
  SidebarInset,
  SidebarProvider,
  SidebarTrigger,
  useSidebar,
} from "@/components/ui/sidebar";
import { roomCopy } from "@/features/rooms/room-copy";
import type { Locale } from "@/lib/i18n/config";
import { messagesFor } from "@/lib/i18n/messages";

type Panel = "participants" | "admission" | "access" | "files" | "invites";

export function SessionSidebar({
  locale,
  showAdmission,
  participants,
  admission,
  access,
  files,
  invites,
  children,
}: Readonly<{
  locale: Locale;
  showAdmission: boolean;
  participants: ReactNode;
  admission: ReactNode;
  access: ReactNode;
  files?: ReactNode;
  invites?: ReactNode;
  children: ReactNode;
}>) {
  const [panel, setPanel] = useState<Panel>("participants");
  const [railTarget, setRailTarget] = useState<HTMLElement | null>(null);

  useEffect(() => {
    void Promise.resolve().then(() =>
      setRailTarget(document.getElementById("session-sidebar-actions")),
    );
  }, []);

  return (
    <SidebarProvider
      className="h-full min-h-0 overflow-hidden"
      data-session-sidebar
      defaultOpen={false}
      style={{ "--sidebar-width": "22rem" } as CSSProperties}
    >
      <Sidebar
        aria-label={messagesFor(locale).sessions.participants}
        className="session-context-panel md:top-10 md:h-[calc(100dvh-2.5rem)]"
        side={locale === "fa" ? "right" : "left"}
        dir={locale === "fa" ? "rtl" : "ltr"}
        collapsible="offcanvas"
      >
        <SidebarContent className="gap-0 bg-sidebar p-4 [&_.bg-card]:border-0 [&_.bg-card]:bg-sidebar">
          <div className="mb-3 flex items-center justify-center gap-2 md:hidden">
            <SessionPanelButtons
              locale={locale}
              panel={panel}
              setPanel={setPanel}
              showAdmission={showAdmission}
              showInvites={Boolean(invites)}
            />
          </div>
          <div hidden={panel !== "participants"} className="h-full">
            {participants}
          </div>
          {showAdmission ? (
            <>
              <div hidden={panel !== "admission"} className="h-full">
                {admission}
              </div>
              <div hidden={panel !== "access"} className="h-full">
                {access}
              </div>
            </>
          ) : null}
          <div hidden={panel !== "files"} className="space-y-4">
            {files}
          </div>
          {invites ? (
            <div hidden={panel !== "invites"} className="min-w-0">
              {invites}
            </div>
          ) : null}
        </SidebarContent>
      </Sidebar>
      {railTarget
        ? createPortal(
            <SessionPanelButtons
              locale={locale}
              panel={panel}
              setPanel={setPanel}
              showAdmission={showAdmission}
              showInvites={Boolean(invites)}
            />,
            railTarget,
          )
        : null}
      <SidebarInset className="h-full min-h-0 min-w-0 overflow-hidden bg-muted/40">
        <div className="h-full min-h-0 overflow-hidden px-2 pt-1 md:px-4 md:pt-0">
          <SidebarTrigger
            className="mb-3 md:hidden"
            aria-label={messagesFor(locale).sessions.participants}
          />
          {children}
        </div>
      </SidebarInset>
    </SidebarProvider>
  );
}

function SessionPanelButtons({
  locale,
  panel,
  setPanel,
  showAdmission,
  showInvites,
}: Readonly<{
  locale: Locale;
  panel: Panel;
  setPanel: (panel: Panel) => void;
  showAdmission: boolean;
  showInvites: boolean;
}>) {
  const { open, setOpen, openMobile, isMobile, setOpenMobile } = useSidebar();
  const isOpen = isMobile ? openMobile : open;
  const sessionText = messagesFor(locale).sessions;
  const collaborationText = messagesFor(locale).collaboration;
  const items: { key: Panel; label: string; icon: typeof UsersRound }[] = [
    { key: "participants", label: sessionText.participants, icon: UsersRound },
    ...(showAdmission
      ? [
          { key: "admission" as const, label: sessionText.admissionTab, icon: UserRoundCheck },
          { key: "access" as const, label: collaborationText.permissions, icon: ShieldCheck },
        ]
      : []),
    { key: "files", label: roomCopy(locale).files, icon: FileStack },
    ...(showInvites
      ? [{ key: "invites" as const, label: roomCopy(locale).invites, icon: Link2 }]
      : []),
  ];

  return (
    <>
      {items.map(({ key, label, icon: Icon }) => (
        <Tooltip key={key}>
          <TooltipTrigger asChild>
            <Button
              aria-label={label}
              aria-pressed={isOpen && panel === key}
              size="icon"
              variant={isOpen && panel === key ? "secondary" : "ghost"}
              className="size-11 shrink-0 rounded-xl"
              onClick={() => {
                if (isMobile) {
                  setPanel(key);
                  setOpenMobile(true);
                } else if (open && panel === key) {
                  setOpen(false);
                } else {
                  setPanel(key);
                  setOpen(true);
                }
              }}
            >
              <Icon className="size-5" aria-hidden="true" />
            </Button>
          </TooltipTrigger>
          <TooltipContent side={locale === "fa" ? "left" : "right"} sideOffset={12}>
            {label}
          </TooltipContent>
        </Tooltip>
      ))}
    </>
  );
}
