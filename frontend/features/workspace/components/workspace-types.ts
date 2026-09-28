export type RoomStatus = "live" | "scheduled" | "idle";

export interface ScheduledSession {
  id: string;
  title: string;
  roomName: string;
  scheduledAt: string;
  durationMinutes: number | null;
  participantCount?: number;
}

export interface Room {
  id: string;
  name: string;
  description?: string;
  status: RoomStatus;
  sessionLabel?: string;
  participantCount?: number;
}
