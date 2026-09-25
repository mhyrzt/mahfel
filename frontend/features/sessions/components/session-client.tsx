"use client";

import { useEffect, useReducer, useRef, useState } from "react";
import { useRouter } from "next/navigation";

import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { roomCopy } from "@/features/rooms/room-copy";
import { browserApiClient } from "@/lib/api/browser";
import type { Locale } from "@/lib/i18n/config";
import { messagesFor } from "@/lib/i18n/messages";
import type { ServerMessage } from "@/lib/websocket/generated/messages";
import { SignalingClient, type SignalingStatus } from "@/lib/websocket/client";

import { initialSessionState, sessionReducer, type SessionCapabilities } from "../session-state";
import type {
  AdmissionRequest,
  Participant,
  ParticipantAction,
  SessionTiming,
} from "./session-types";
import { SessionScreen } from "./session-screen";
import { CollaborationWorkspace } from "./collaboration-workspace";

interface SessionClientProps {
  actorId?: string;
  capabilities: SessionCapabilities;
  locale: Locale;
  sessionId: string;
  roomId: string;
  sessionTiming?: SessionTiming | null;
  title: string;
  canAdmit?: boolean;
}

interface RosterParticipant {
  id: string;
  actor_id: string;
  name: string;
  role: "manager" | "member";
  avatar_url: string | null;
}

export function SessionClient({
  actorId,
  capabilities,
  locale,
  sessionId,
  roomId,
  sessionTiming,
  title,
  canAdmit = false,
}: Readonly<SessionClientProps>) {
  const [state, dispatch] = useReducer(sessionReducer, initialSessionState);
  const router = useRouter();
  const clientRef = useRef<SignalingClient | null>(null);
  const participantsRef = useRef<Participant[]>([]);
  const pendingRosterRef = useRef<string[] | null>(null);
  const rosterVersionRef = useRef(-1);
  const [actionError, setActionError] = useState<string | null>(null);
  const [confirmation, setConfirmation] = useState<{
    action: "remove" | "block";
    participant: Participant;
  } | null>(null);
  const [admissionRequests, setAdmissionRequests] = useState<AdmissionRequest[]>([]);
  const actionText = roomCopy(locale);

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
        const previousById = new Map(participantsRef.current.map((item) => [item.id, item]));
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
            actorId: item.actor_id,
            name: item.name,
            initials: initials || "P",
            role: item.role,
            avatarUrl: item.avatar_url ?? undefined,
            speaking: previousById.get(id)?.speaking,
            muted: previousById.get(id)?.muted,
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
          dispatch({
            type: "participant-removed",
            participantId:
              participantsRef.current.find(
                (participant) => participant.actorId === message.participant_id,
              )?.id ?? message.participant_id,
          });
          return;
        case "active_speakers":
          dispatch({ type: "active-speakers-changed", participantIds: message.participant_ids });
          return;
        case "participant_muted":
          dispatch({ type: "participant-muted", actorId: message.actor_id, muted: true });
          return;
        case "participant_unmuted":
          dispatch({ type: "participant-muted", actorId: message.actor_id, muted: false });
          return;
        case "error":
          setActionError(message.message);
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

  useEffect(() => {
    if (!canAdmit) {
      return undefined;
    }
    let active = true;
    async function refreshLobby() {
      try {
        const { data } = await browserApiClient.get<{
          entries: { request_id: string; actor_id: string; requested_at: string }[];
        }>(`/v1/sessions/${encodeURIComponent(sessionId)}/lobby`);
        if (active) {
          setAdmissionRequests(
            data.entries.map((entry) => ({
              id: entry.request_id,
              actorId: entry.actor_id,
              name: `Participant ${entry.actor_id.slice(0, 4)}`,
              initials: "P",
              requestedAt: entry.requested_at,
            })),
          );
        }
      } catch {
        if (active) {
          setActionError(actionText.moderationFailed);
        }
      }
    }
    void Promise.resolve().then(refreshLobby);
    const interval = window.setInterval(() => void refreshLobby(), 3_000);
    return () => {
      active = false;
      window.clearInterval(interval);
    };
  }, [actionText.moderationFailed, canAdmit, sessionId]);

  async function decideAdmission(request: AdmissionRequest, decision: "admit" | "reject") {
    if (!request.actorId) {
      return;
    }
    try {
      await browserApiClient.post(
        `/v1/sessions/${encodeURIComponent(sessionId)}/lobby/${encodeURIComponent(request.actorId)}/${decision}`,
      );
      setAdmissionRequests((requests) => requests.filter((item) => item.id !== request.id));
      setActionError(null);
    } catch {
      setActionError(actionText.moderationFailed);
    }
  }

  function sendParticipantAction(action: ParticipantAction, participant: Participant) {
    if (!participant.actorId) {
      setActionError(actionText.moderationFailed);
      return;
    }
    if (action === "remove" || action === "block") {
      setConfirmation({ action, participant });
      return;
    }
    try {
      clientRef.current?.send({
        type: "mute_participant",
        session_id: sessionId,
        actor_id: participant.actorId,
        kind: "temporary",
      });
      setActionError(null);
    } catch {
      setActionError(actionText.moderationFailed);
    }
  }

  function confirmParticipantAction() {
    const target = confirmation;
    if (!target?.participant.actorId) {
      return;
    }
    try {
      clientRef.current?.send(
        target.action === "remove"
          ? {
              type: "remove_participant",
              session_id: sessionId,
              actor_id: target.participant.actorId,
            }
          : {
              type: "block_principal",
              session_id: sessionId,
              actor_id: target.participant.actorId,
            },
      );
      setActionError(null);
      setConfirmation(null);
    } catch {
      setActionError(actionText.moderationFailed);
    }
  }

  return (
    <div className="space-y-8">
      <SessionScreen
        title={title}
        state={state}
        sessionTiming={sessionTiming}
        locale={locale}
        onRetryJoin={() => clientRef.current?.send({ type: "join", session_id: sessionId })}
        onParticipantAction={sendParticipantAction}
        actionError={actionError}
        admissionRequests={admissionRequests}
        canAdmit={canAdmit}
        onAdmit={(request) => void decideAdmission(request, "admit")}
        onReject={(request) => void decideAdmission(request, "reject")}
        onLeave={() => {
          clientRef.current?.leave();
          clientRef.current?.close();
          router.push(`/room/${roomId}`);
        }}
      />
      {state.phase === "live" || state.phase === "ended" ? (
        <CollaborationWorkspace sessionId={sessionId} locale={locale} />
      ) : null}
      <Dialog
        onOpenChange={(open) => {
          if (!open) {
            setConfirmation(null);
          }
        }}
        open={confirmation !== null}
      >
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{actionText.confirmAction}</DialogTitle>
            <DialogDescription>
              {confirmation?.action === "block"
                ? actionText.confirmBlock
                : actionText.confirmRemove}
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button onClick={() => setConfirmation(null)} variant="outline">
              {actionText.keepSession}
            </Button>
            <Button onClick={confirmParticipantAction} variant="destructive">
              {actionText.confirmAction}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
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
