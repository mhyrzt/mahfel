import { SchedulePanel } from "./schedule-panel";
import { RoomList } from "./room-list";
import type { Room, ScheduledSession } from "./workspace-types";

interface WorkspaceHomeProps {
  rooms: Room[];
  sessions: ScheduledSession[];
}

export function WorkspaceHome({ rooms, sessions }: Readonly<WorkspaceHomeProps>) {
  return (
    <div className="min-h-0 lg:h-[calc(100dvh-8rem)] lg:overflow-hidden">
      <h1 className="sr-only">Workspace overview</h1>
      <div className="grid min-h-0 h-full gap-6 lg:grid-cols-[19rem_minmax(0,1fr)]">
        <aside className="lg:min-h-0">
          <SchedulePanel sessions={sessions} />
        </aside>
        <RoomList rooms={rooms} />
      </div>
    </div>
  );
}
