export type SessionStatus = "scheduled" | "live" | "ended";

export type ParticipantAction = "mute" | "remove" | "block";

export interface Participant {
  id: string;
  name: string;
  initials: string;
  role: "manager" | "moderator" | "member";
  speaking?: boolean;
  muted?: boolean;
  videoEnabled?: boolean;
}

export interface AdmissionRequest {
  id: string;
  name: string;
  initials: string;
  requestedAt: string;
}
