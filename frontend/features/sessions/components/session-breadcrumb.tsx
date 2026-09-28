import Link from "next/link";
import { ChevronRight } from "lucide-react";

import type { Locale } from "@/lib/i18n/config";

export function SessionBreadcrumb({
  roomId,
  roomName,
  title,
  locale,
}: Readonly<{ roomId: string; roomName: string; title: string; locale: Locale }>) {
  return (
    <nav
      aria-label={locale === "fa" ? "مسیر صفحه" : "Breadcrumb"}
      className="flex min-h-10 shrink-0 items-center gap-1.5 overflow-hidden px-3 text-xs text-muted-foreground md:px-4"
      dir={locale === "fa" ? "rtl" : "ltr"}
    >
      <Link className="shrink-0 hover:text-foreground" href="/workspace">
        {locale === "fa" ? "خانه" : "Home"}
      </Link>
      <ChevronRight aria-hidden="true" className="size-3.5 shrink-0 rtl:rotate-180" />
      <Link className="max-w-40 truncate hover:text-foreground" href={`/room/${roomId}`}>
        {roomName}
      </Link>
      <ChevronRight aria-hidden="true" className="size-3.5 shrink-0 rtl:rotate-180" />
      <span aria-current="page" className="truncate font-medium text-foreground">
        {title}
      </span>
    </nav>
  );
}
