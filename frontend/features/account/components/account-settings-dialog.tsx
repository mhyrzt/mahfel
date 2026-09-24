"use client";

import { useRouter } from "next/navigation";

import { Dialog, DialogContent, DialogDescription, DialogTitle } from "@/components/ui/dialog";
import { AccountSettingsPanel } from "@/features/account/components/account-settings-panel";
import type { CalendarPreference } from "@/features/account/calendar-preference";
import type { Theme } from "@/features/appearance/theme";
import type { Locale } from "@/lib/i18n/config";
import { messagesFor } from "@/lib/i18n/messages";

interface AccountSettingsDialogProps {
  calendar: CalendarPreference;
  email: string;
  image: string | null;
  locale: Locale;
  name: string;
  theme: Theme;
}

export function AccountSettingsDialog(props: Readonly<AccountSettingsDialogProps>) {
  const router = useRouter();
  const messages = messagesFor(props.locale).account;

  return (
    <Dialog open onOpenChange={(open) => !open && router.back()}>
      <DialogContent className="max-h-[calc(100dvh-2rem)] max-w-4xl overflow-y-auto p-5 sm:p-7">
        <DialogTitle className="sr-only">{messages.title}</DialogTitle>
        <DialogDescription className="sr-only">{messages.description}</DialogDescription>
        <AccountSettingsPanel {...props} />
      </DialogContent>
    </Dialog>
  );
}
