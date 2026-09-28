"use client";

import { signOut } from "next-auth/react";
import { LogOut, Settings2 } from "lucide-react";
import { useState } from "react";

import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogTitle } from "@/components/ui/dialog";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { AccountSettingsPanel } from "@/features/account/components/account-settings-panel";
import type { CalendarPreference } from "@/features/account/calendar-preference";
import type { Theme } from "@/features/appearance/theme";
import { getFederatedLogoutUrl } from "@/lib/auth/federated-logout-client";
import type { Locale } from "@/lib/i18n/config";
import { messagesFor } from "@/lib/i18n/messages";

interface AccountQuickSettingsProps {
  accountImage?: string | null;
  accountName?: string | null;
  calendar: CalendarPreference;
  email?: string;
  locale: Locale;
  theme: Theme;
}

export function AccountQuickSettings({
  accountImage,
  accountName,
  calendar,
  email = "",
  locale,
  theme,
}: Readonly<AccountQuickSettingsProps>) {
  const [logoutError, setLogoutError] = useState(false);
  const [loggingOut, setLoggingOut] = useState(false);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const messages = messagesFor(locale);
  const shell = messages.shell;
  const accountInitial = accountName?.trim().slice(0, 1).toLocaleUpperCase(locale) || "?";

  async function logout() {
    setLoggingOut(true);
    setLogoutError(false);
    try {
      const logoutUrl = await getFederatedLogoutUrl();
      await signOut({ redirect: false, callbackUrl: "/login?loggedOut=1" });
      window.location.assign(logoutUrl);
    } catch {
      setLoggingOut(false);
      setLogoutError(true);
    }
  }

  return (
    <>
      <DropdownMenu dir={locale === "fa" ? "rtl" : "ltr"}>
        <DropdownMenuTrigger asChild>
          <Button
            aria-label={shell.account}
            className="size-12 rounded-full"
            size="icon"
            variant="ghost"
          >
            <Avatar className="size-8">
              <AvatarImage alt="" src={accountImage ?? undefined} />
              <AvatarFallback className="bg-primary text-xs font-semibold text-primary-foreground">
                {accountInitial}
              </AvatarFallback>
            </Avatar>
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent
          align="end"
          className="w-64 rounded-2xl p-2 shadow-xl"
          side={locale === "fa" ? "left" : "right"}
          sideOffset={10}
        >
          <div className="flex min-w-0 items-center gap-3 px-2 py-2">
            <Avatar className="size-10 shrink-0">
              <AvatarImage alt="" src={accountImage ?? undefined} />
              <AvatarFallback className="bg-primary/10 font-semibold text-primary">
                {accountInitial}
              </AvatarFallback>
            </Avatar>
            <div className="min-w-0">
              <p className="truncate text-sm font-semibold">
                {accountName || messages.account.title}
              </p>
              {email ? <p className="truncate text-xs text-muted-foreground">{email}</p> : null}
            </div>
          </div>
          <DropdownMenuSeparator />
          <DropdownMenuItem
            className="h-10 gap-2 rounded-lg"
            onSelect={() => setSettingsOpen(true)}
          >
            <Settings2 aria-hidden="true" />
            {messages.account.openSettings}
          </DropdownMenuItem>
          <DropdownMenuItem
            className="h-10 gap-2 rounded-lg text-destructive focus:text-destructive"
            disabled={loggingOut}
            onSelect={(event) => {
              event.preventDefault();
              void logout();
            }}
          >
            <LogOut aria-hidden="true" />
            {shell.logout}
          </DropdownMenuItem>
          {logoutError ? (
            <p className="px-2 py-1 text-xs text-destructive" role="alert">
              {shell.logoutFailed}
            </p>
          ) : null}
        </DropdownMenuContent>
      </DropdownMenu>
      <Dialog onOpenChange={setSettingsOpen} open={settingsOpen}>
        <DialogContent className="max-h-[calc(100dvh-2rem)] max-w-3xl overflow-y-auto p-5 sm:p-7">
          <DialogTitle className="sr-only">{messages.account.title}</DialogTitle>
          <DialogDescription className="sr-only">{messages.account.description}</DialogDescription>
          <AccountSettingsPanel
            calendar={calendar}
            email={email}
            image={accountImage ?? null}
            locale={locale}
            name={accountName ?? ""}
            theme={theme}
          />
        </DialogContent>
      </Dialog>
    </>
  );
}
