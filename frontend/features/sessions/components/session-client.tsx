"use client";

import { useEffect, useReducer, useRef } from "react";

import { browserApiClient } from "@/lib/api/browser";
import type { Locale } from "@/lib/i18n/config";
import { messagesFor } from "@/lib/i18n/messages";
import type { ServerMessage } from "@/lib/websocket/generated/messages";
import { SignalingClient, type SignalingStatus } from "@/lib/websocket/client";

import { initialSessionState, sessionReducer, type SessionCapabilities } from "../session-state";
import type { Participant } from "./session-types";
import { SessionScreen } from "./session-screen";
import { CollaborationWorkspace } from "./collaboration-workspace";

interface SessionClientProps {
  actorId?: string;
  capabilities: SessionCapabilities;
  locale: Locale;
  sessionId: string;
  title: string;
}

interface RosterParticipant {
  id: string;
  name: string;
  role: "manager" | "member";
  avatar_url: string | null;
}

export function SessionClient({
  actorId,
  capabilities,
  locale,
  sessionId,
  title,
}: Readonly<SessionClientProps>) {
  const [state, dispatch] = useReducer(sessionReducer, initialSessionState);
  const clientRef = useRef<SignalingClient | null>(null);
  const participantsRef = useRef<Participant[]>([]);
  const pendingRosterRef = useRef<string[] | null>(null);
  const rosterVersionRef = useRef(-1);

  useEffect(() => {
    participantsRef.current = state.participants;
  }, [state.participants]);

  useEffect(() => {
    const text = messagesFor(locale).sessions;
    let active = true;
    let rosterRequest = 0;
    const client = new SignalingClient(sessionId, handleMessage, handleStatus);
    clientRef.current = client;
    client.connect();

    return () => {
      active = false;
      client.leave();
      client.close();
      clientRef.current = null;
    };

    async function refreshRoster(ids: string[]) {
      const request = ++rosterRequest;
      try {
        const { data } = await browserApiClient.get<RosterParticipant[]>(
          `/v1/sessions/${sessionId}/roster`,
        );
        if (!active || request !== rosterRequest) {
          return;
        }
        const byId = new Map(data.map((item) => [item.id, item]));
        const participants = ids.map((id) => {
          const item = byId.get(id);
          if (!item) {
            return participantFor(id);
          }
          const initials = item.name
            .split(/\s+/)
            .filter(Boolean)
            .slice(0, 2)
            .map((part) => part[0]?.toUpperCase())
            .join("");
          return {
            id,
            name: item.name,
            initials: initials || "P",
            role: item.role,
            avatarUrl: item.avatar_url ?? undefined,
          } satisfies Participant;
        });
        dispatch({ type: "roster-synchronized", participants });
      } catch {
        if (active && request === rosterRequest) {
          dispatch({ type: "roster-synchronized", participants: participantList(ids) });
        }
      }
    }

    function handleStatus(status: SignalingStatus) {
      if (status === "unauthorized") {
        dispatch({
          type: "connection-unavailable",
          message: text.signInAgainMessage,
        });
      } else if (status === "unavailable") {
        dispatch({
          type: "connection-unavailable",
          message: text.connectionUnavailableMessage,
        });
      } else if (status === "reconnecting") {
        dispatch({ type: "connection-unavailable", message: text.reconnectingMessage });
      }
    }

    function handleMessage(message: ServerMessage) {
      if ("session_id" in message && message.session_id !== sessionId) {
        return;
      }

      switch (message.type) {
        case "joined": {
          const ids = pendingRosterRef.current ?? participantsRef.current.map(({ id }) => id);
          const participants = participantList(ids);
          dispatch({
            type: "session-connected",
            sessionId,
            participantId: message.participant_id,
            roomName: title,
            participants,
            capabilities,
          });
          pendingRosterRef.current = null;
          void refreshRoster(ids);
          return;
        }
        case "join_waiting":
          dispatch({ type: "admission-pending", roomName: title });
          return;
        case "join_rejected":
          dispatch({ type: "admission-rejected", message: message.message });
          return;
        case "session_roster":
          if (message.version < rosterVersionRef.current) {
            return;
          }
          rosterVersionRef.current = message.version;
          pendingRosterRef.current = message.participant_ids;
          dispatch({
            type: "roster-synchronized",
            participants: participantList(message.participant_ids),
          });
          void refreshRoster(message.participant_ids);
          return;
        case "participant_joined":
          dispatch({
            type: "participant-upserted",
            participant: participantFor(message.participant_id),
          });
          void refreshRoster([
            ...new Set([...participantsRef.current.map(({ id }) => id), message.participant_id]),
          ]);
          return;
        case "participant_left":
        case "participant_removed":
          rosterRequest += 1;
          dispatch({ type: "participant-removed", participantId: message.participant_id });
          return;
        case "principal_blocked":
          if (message.actor_id === actorId) {
            dispatch({
              type: "admission-rejected",
              message: text.accessBlockedMessage,
            });
            client.close();
          }
          return;
        case "left":
        case "bye":
          dispatch({ type: "session-ended" });
        default:
      }
    }
  }, [actorId, capabilities, locale, sessionId, title]);

  return (
    <div className="space-y-8">
      <SessionScreen
        title={title}
        state={state}
        locale={locale}
        onRetryJoin={() => clientRef.current?.send({ type: "join", session_id: sessionId })}
      />
      {state.phase === "live" || state.phase === "ended" ? (
        <CollaborationWorkspace sessionId={sessionId} locale={locale} />
      ) : null}
    </div>
  );
}

function participantList(ids: string[]): Participant[] {
  return ids.map(participantFor);
}

function participantFor(id: string): Participant {
  const name = `Participant ${id.slice(0, 4)}`;
  return { id, name, initials: "P", role: "member" };
}
