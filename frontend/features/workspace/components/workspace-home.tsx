import { SchedulePanel } from "./schedule-panel";
import { RoomList } from "./room-list";
import type { Room, ScheduledSession } from "./workspace-types";
import type { Locale } from "@/lib/i18n/config";
import type { CalendarPreference } from "@/features/account/calendar-preference";
import { WorkspaceGreeting } from "./workspace-greeting";

interface WorkspaceHomeProps {
  rooms: Room[];
  sessions: ScheduledSession[];
  locale?: Locale;
  calendar?: CalendarPreference;
  accountName?: string | null;
  referenceDate: string;
  scheduleUnavailable?: boolean;
}

export function WorkspaceHome({
  rooms,
  sessions,
  locale = "en",
  calendar = "gregorian",
  accountName,
  referenceDate,
  scheduleUnavailable = false,
}: Readonly<WorkspaceHomeProps>) {
  return (
    <div className="mx-auto flex min-h-[calc(100dvh-8rem)] w-full max-w-7xl flex-col justify-center gap-8 lg:h-[calc(100dvh-8rem)] lg:overflow-hidden">
      <h1 className="sr-only">Workspace overview</h1>
      <WorkspaceGreeting accountName={accountName} locale={locale} referenceDate={referenceDate} />
      <div className="grid min-h-0 gap-6 lg:max-h-[min(44rem,calc(100dvh-13rem))] lg:grid-cols-[19rem_minmax(0,1fr)]">
        <aside className="lg:min-h-0">
          <SchedulePanel
            calendar={calendar}
            locale={locale}
            referenceDate={referenceDate}
            scheduleUnavailable={scheduleUnavailable}
            sessions={sessions}
          />
        </aside>
        <RoomList locale={locale} rooms={rooms} />
      </div>
    </div>
  );
}
