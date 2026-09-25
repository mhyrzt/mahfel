"use client";

import { useState } from "react";
import Link from "next/link";

import { Button } from "@/components/ui/button";
import { roomCopy } from "@/features/rooms/room-copy";
import type { Locale } from "@/lib/i18n/config";

import { CreateWorkspaceDialog } from "./create-workspace-dialog";

export function WorkspaceEntry({ locale }: Readonly<{ locale: Locale }>) {
  const [open, setOpen] = useState(false);
  const text = roomCopy(locale);
  return (
    <section
      className="mx-auto flex min-h-[60dvh] max-w-lg flex-col justify-center gap-6"
      data-testid="workspace-entry"
    >
      <div className="space-y-2">
        <h1 className="text-2xl font-semibold tracking-tight">{text.empty}</h1>
        <p className="text-sm leading-6 text-muted-foreground">{text.emptyHelp}</p>
      </div>
      <div className="flex flex-wrap gap-3">
        <Button onClick={() => setOpen(true)}>{text.create}</Button>
        <Button asChild variant="outline">
          <Link href="/join">{text.joinWithCode}</Link>
        </Button>
      </div>
      <CreateWorkspaceDialog kind="room" locale={locale} onOpenChange={setOpen} open={open} />
    </section>
  );
}
