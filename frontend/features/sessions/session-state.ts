import type { Participant } from "./components/session-types";

export type SessionPhase = "loading" | "waiting" | "live" | "ended" | "rejected" | "unavailable";

export interface SessionCapabilities {
  canPublishAudio: boolean;
  canPublishVideo: boolean;
  canShareScreen: boolean;
  canModerate: boolean;
}

export interface SessionState {
  phase: SessionPhase;
  sessionId: string | null;
  participantId: string | null;
  roomName: string | null;
  participants: Participant[];
  capabilities: SessionCapabilities;
  message: string | null;
}

export type SessionAction =
  | { type: "session-loading" }
  | {
      type: "session-connected";
      sessionId: string;
      participantId: string;
      roomName: string;
      participants: Participant[];
      capabilities: SessionCapabilities;
    }
  | { type: "admission-pending"; roomName: string }
  | { type: "admission-rejected"; message: string }
  | { type: "session-ended" }
  | { type: "connection-unavailable"; message: string }
  | { type: "roster-synchronized"; participants: Participant[] }
  | { type: "participant-upserted"; participant: Participant }
  | { type: "participant-removed"; participantId: string }
  | { type: "active-speakers-changed"; participantIds: string[] }
  | { type: "participant-muted"; actorId: string; muted: boolean };

const noCapabilities: SessionCapabilities = {
  canPublishAudio: false,
  canPublishVideo: false,
  canShareScreen: false,
  canModerate: false,
};

export const initialSessionState: SessionState = {
  phase: "loading",
  sessionId: null,
  participantId: null,
  roomName: null,
  participants: [],
  capabilities: noCapabilities,
  message: null,
};

export function sessionReducer(state: SessionState, action: SessionAction): SessionState {
  switch (action.type) {
    case "session-loading":
      return initialSessionState;
    case "session-connected":
      return {
        phase: "live",
        sessionId: action.sessionId,
        participantId: action.participantId,
        roomName: action.roomName,
        participants: synchronizeParticipants(action.participants),
        capabilities: action.capabilities,
        message: null,
      };
    case "admission-pending":
      return {
        ...initialSessionState,
        phase: "waiting",
        roomName: action.roomName,
      };
    case "admission-rejected":
      return {
        ...initialSessionState,
        phase: "rejected",
        message: action.message,
      };
    case "session-ended":
      return { ...state, phase: "ended", capabilities: noCapabilities };
    case "connection-unavailable":
      return {
        ...state,
        phase: "unavailable",
        capabilities: noCapabilities,
        message: action.message,
      };
    case "roster-synchronized":
      return state.phase === "live"
        ? { ...state, participants: synchronizeParticipants(action.participants) }
        : state;
    case "participant-upserted":
      return state.phase === "live"
        ? { ...state, participants: upsertParticipant(state.participants, action.participant) }
        : state;
    case "participant-removed":
      return state.phase === "live"
        ? {
            ...state,
            participants: state.participants.filter(
              (participant) => participant.id !== action.participantId,
            ),
          }
        : state;
    case "active-speakers-changed": {
      const active = new Set(action.participantIds);
      return state.phase === "live"
        ? {
            ...state,
            participants: state.participants.map((participant) => ({
              ...participant,
              speaking: active.has(participant.id),
            })),
          }
        : state;
    }
    case "participant-muted":
      return state.phase === "live"
        ? {
            ...state,
            participants: state.participants.map((participant) =>
              participant.actorId === action.actorId
                ? { ...participant, muted: action.muted }
                : participant,
            ),
          }
        : state;
    default:
      return state;
  }
}

export function canUseMediaControls(state: SessionState): boolean {
  return (
    state.phase === "live" &&
    (state.capabilities.canPublishAudio ||
      state.capabilities.canPublishVideo ||
      state.capabilities.canShareScreen)
  );
}

function synchronizeParticipants(participants: Participant[]): Participant[] {
  return Array.from(
    new Map(participants.map((participant) => [participant.id, participant])).values(),
  );
}

function upsertParticipant(participants: Participant[], participant: Participant): Participant[] {
  return synchronizeParticipants([...participants, participant]);
}
