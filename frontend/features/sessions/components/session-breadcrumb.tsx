import type { SessionSurface } from "./use-session-view";
import { messagesFor } from "@/lib/i18n/messages";
import Link from "next/link";
import {
  Breadcrumb,
  BreadcrumbList,
  BreadcrumbItem,
  BreadcrumbLink,
  BreadcrumbPage,
  BreadcrumbSeparator,
} from "@/components/ui/breadcrumb";

import type { Locale } from "@/lib/i18n/config";

export function SessionBreadcrumb({
  roomId,
  roomName,
  title,
  locale,
  activeSurface,
}: Readonly<{
  roomId: string;
  roomName: string;
  title: string;
  locale: Locale;
  activeSurface?: SessionSurface;
}>) {
  const text = messagesFor(locale);
  const viewLabels = {
    participants: text.sessions.participants,
    activities: text.interactions.title,
    notes: text.collaboration.notes,
    whiteboard: text.collaboration.whiteboard,
    presentation: text.collaboration.presentation,
  };
  const viewLabel = activeSurface ? viewLabels[activeSurface] : null;
  return (
    <Breadcrumb
      aria-label={locale === "fa" ? "مسیر صفحه" : "Breadcrumb"}
      className="flex min-h-10 shrink-0 items-center gap-1.5 overflow-hidden px-3 text-xs text-muted-foreground md:px-4"
      dir={locale === "fa" ? "rtl" : "ltr"}
    >
      <BreadcrumbList className="min-w-0 flex-nowrap text-xs gap-1.5">
        <BreadcrumbItem className="shrink-0">
          <BreadcrumbLink asChild>
            <Link className="shrink-0 hover:text-foreground" href="/workspace">
              {locale === "fa" ? "خانه" : "Home"}
            </Link>
          </BreadcrumbLink>
        </BreadcrumbItem>
        <BreadcrumbSeparator className="shrink-0" />
        <BreadcrumbItem className="min-w-0">
          <BreadcrumbLink asChild>
            <Link className="max-w-40 truncate hover:text-foreground" href={`/room/${roomId}`}>
              {roomName}
            </Link>
          </BreadcrumbLink>
        </BreadcrumbItem>
        <BreadcrumbSeparator className="shrink-0" />
        <BreadcrumbItem className="min-w-0">
          {viewLabel ? (
            <span className="truncate">{title}</span>
          ) : (
            <BreadcrumbPage className="truncate font-medium">{title}</BreadcrumbPage>
          )}
        </BreadcrumbItem>
        {viewLabel ? (
          <>
            <BreadcrumbSeparator className="shrink-0" />
            <BreadcrumbItem className="min-w-0">
              <BreadcrumbPage className="truncate font-medium">{viewLabel}</BreadcrumbPage>
            </BreadcrumbItem>
          </>
        ) : null}
      </BreadcrumbList>
    </Breadcrumb>
  );
}
