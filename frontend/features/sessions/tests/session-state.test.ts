import { expect, test } from "vitest";

import type { Participant } from "../components/session-types";
import {
  canUseMediaControls,
  initialSessionState,
  sessionReducer,
  type SessionCapabilities,
} from "../session-state";

const capabilities: SessionCapabilities = {
  canPublishAudio: true,
  canPublishVideo: true,
  canShareScreen: false,
  canModerate: false,
};

const alex: Participant = {
  id: "participant-alex",
  name: "Alex",
  initials: "AL",
  role: "member",
};

const taylor: Participant = {
  id: "participant-taylor",
  name: "Taylor",
  initials: "TA",
  role: "moderator",
};

function connectedState() {
  return sessionReducer(initialSessionState, {
    type: "session-connected",
    sessionId: "session-1",
    participantId: alex.id,
    roomName: "Biology 101",
    participants: [alex, taylor],
    capabilities,
  });
}

test("reconnect replaces the roster without duplicate stable participants", () => {
  const reconnected = sessionReducer(connectedState(), {
    type: "session-connected",
    sessionId: "session-1",
    participantId: alex.id,
    roomName: "Biology 101",
    participants: [alex, { ...taylor, muted: true }, alex],
    capabilities,
  });

  expect(reconnected.participantId).toBe(alex.id);
  expect(reconnected.participants).toEqual([alex, { ...taylor, muted: true }]);
});

test("roster events upsert and remove participants", () => {
  const withUpdatedAlex = sessionReducer(connectedState(), {
    type: "participant-upserted",
    participant: { ...alex, speaking: true },
  });
  const afterRemoval = sessionReducer(withUpdatedAlex, {
    type: "participant-removed",
    participantId: taylor.id,
  });

  expect(afterRemoval.participants).toEqual([{ ...alex, speaking: true }]);
});

test("waiting and unavailable states do not enable media controls", () => {
  const waiting = sessionReducer(initialSessionState, {
    type: "admission-pending",
    roomName: "Biology 101",
  });
  const unavailable = sessionReducer(connectedState(), {
    type: "connection-unavailable",
    message: "Connection lost.",
  });

  expect(canUseMediaControls(waiting)).toBe(false);
  expect(canUseMediaControls(unavailable)).toBe(false);
});
