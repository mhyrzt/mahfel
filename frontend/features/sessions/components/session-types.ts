export type SessionStatus = "scheduled" | "live" | "ended";

export type ParticipantAction = "mute" | "remove" | "block";

export interface Participant {
  id: string;
  actorId?: string;
  name: string;
  initials: string;
  avatarUrl?: string;
  role: "manager" | "moderator" | "member";
  speaking?: boolean;
  muted?: boolean;
  videoEnabled?: boolean;
  screenShareEnabled?: boolean;
}

export interface ParticipantMediaStreams {
  camera?: MediaStream;
  screenShare?: MediaStream;
}

export interface AdmissionRequest {
  id: string;
  actorId?: string;
  name: string;
  initials: string;
  requestedAt: string;
}

export type SessionTiming =
  | { status: "available"; startedAt: string | null; endedAt: string | null; observedAt: string }
  | { status: "unavailable" };
