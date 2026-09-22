import { SchedulePanel } from "./schedule-panel";
import { RoomList } from "./room-list";
import type { Room, ScheduledSession } from "./workspace-types";

interface WorkspaceHomeProps {
  rooms: Room[];
  sessions: ScheduledSession[];
}

export function WorkspaceHome({ rooms, sessions }: Readonly<WorkspaceHomeProps>) {
  return (
    <div className="flex min-h-0 flex-col gap-6 lg:h-[calc(100dvh-8rem)] lg:overflow-hidden">
      <header className="flex flex-wrap items-end justify-between gap-3">
        <div>
          <p className="text-sm font-medium text-muted-foreground">Tuesday, September 22</p>
          <h1 className="text-3xl font-semibold tracking-tight text-balance">Your workspace</h1>
        </div>
        <p className="text-sm text-muted-foreground">2 sessions scheduled today</p>
      </header>
      <div className="grid min-h-0 flex-1 gap-6 lg:grid-cols-[19rem_minmax(0,1fr)]">
        <aside className="lg:min-h-0">
          <SchedulePanel sessions={sessions} />
        </aside>
        <RoomList rooms={rooms} />
      </div>
    </div>
  );
}
