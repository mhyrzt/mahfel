import type { SessionSurface } from "./use-session-view";
import { directionForLocale, type Locale } from "@/lib/i18n/config";

import { SessionBreadcrumb } from "./session-breadcrumb";
import { SessionConnectionStatus } from "./session-connection-status";
import type { SessionConnection } from "./use-session-connection";

export function SessionHeader({
  roomId,
  roomName,
  title,
  locale,
  connection,
  activeSurface,
}: Readonly<{
  roomId?: string;
  roomName: string;
  title: string;
  locale: Locale;
  connection?: SessionConnection;
  activeSurface?: SessionSurface;
}>) {
  return (
    <div
      className="flex min-h-10 shrink-0 items-center justify-between gap-2 pe-2 md:pe-4"
      dir={directionForLocale(locale)}
    >
      <div className="min-w-0 flex-1">
        {roomId ? (
          <SessionBreadcrumb
            activeSurface={activeSurface}
            roomId={roomId}
            roomName={roomName}
            title={title}
            locale={locale}
          />
        ) : null}
      </div>
      {connection ? <SessionConnectionStatus connection={connection} locale={locale} /> : null}
    </div>
  );
}
