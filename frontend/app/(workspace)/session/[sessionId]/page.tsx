import { notFound, redirect } from "next/navigation";

import { SessionActivitiesHistory } from "@/features/interactions/activities-panel";
import { SessionClient } from "@/features/sessions/components/session-client";
import type { SessionCapabilities } from "@/features/sessions/session-state";
import { getLocale } from "@/lib/i18n/server";
import {
  createServerApiClient,
  ServerApiAuthenticationError,
  ServerApiRequestError,
} from "@/lib/api/server";
import type { components } from "@/lib/api/types";
import { getServerSessionUser } from "@/lib/auth/server";
import { CollaborationWorkspace } from "@/features/sessions/components/collaboration-workspace";
import { SessionLifecyclePanel } from "@/features/sessions/components/session-lifecycle-panel";
import { InviteManager } from "@/features/invites/components/invite-manager";
import type { SessionTiming } from "@/features/sessions/components/session-types";
import { ContextFileLibrary } from "@/features/files/components/context-file-library";
import { getProfile } from "@/features/account/profile";
import { ParticipationSummary } from "@/features/sessions/components/participation-summary";
import { SessionDetailView } from "@/features/sessions/components/session-detail-view";
import { SessionBreadcrumb } from "@/features/sessions/components/session-breadcrumb";

interface SessionPageProps {
  params: Promise<{ sessionId: string }>;
}

interface SessionCapabilitiesResponse {
  capabilities: string[];
}

export default async function SessionPage({ params }: Readonly<SessionPageProps>) {
  const [{ sessionId }, locale, user, profile] = await Promise.all([
    params,
    getLocale(),
    getServerSessionUser(),
    getProfile(),
  ]);
  let capabilities: SessionCapabilities;
  let sessionTiming: SessionTiming | null = null;
  let session: components["schemas"]["SessionResponse"];
  let roomName: string;
  let canManage = false;
  let participation: components["schemas"]["SessionParticipationResponse"] | null = null;
  try {
    const api = await createServerApiClient();
    const [sessionResponse, capabilityResponse] = await Promise.all([
      api.get<components["schemas"]["SessionResponse"]>(
        `/v1/sessions/${encodeURIComponent(sessionId)}`,
      ),
      api.get<SessionCapabilitiesResponse>(
        `/v1/sessions/${encodeURIComponent(sessionId)}/capabilities`,
      ),
    ]);
    session = sessionResponse.data;
    roomName = (
      await api.get<components["schemas"]["RoomResponse"]>(
        `/v1/rooms/${encodeURIComponent(session.room_id)}`,
      )
    ).data.title;
    const granted = new Set(capabilityResponse.data.capabilities);
    canManage = granted.has("manage_session");
    if (canManage) {
      sessionTiming = {
        status: "available",
        startedAt: session.started_at ?? null,
        endedAt: session.ended_at ?? null,
        observedAt: new Date().toISOString(),
      };
      if (session.status === "ended") {
        try {
          const response = await api.get<components["schemas"]["SessionParticipationResponse"]>(
            `/v1/sessions/${encodeURIComponent(sessionId)}/participation`,
          );
          participation = response.data;
        } catch {
          participation = null;
        }
      }
    }
    capabilities = {
      canPublishAudio: granted.has("publish_media"),
      canPublishVideo: granted.has("publish_media"),
      canShareScreen: granted.has("share_screen"),
      canModerate: ["mute_others", "remove_participant", "admit_participant"].some((item) =>
        granted.has(item),
      ),
      canMuteOthers: granted.has("mute_others"),
      canRemoveParticipant: granted.has("remove_participant"),
    };
  } catch (error) {
    if (error instanceof ServerApiAuthenticationError) {
      redirect(`/login?callbackUrl=${encodeURIComponent(`/session/${sessionId}`)}`);
    }
    if (error instanceof ServerApiRequestError && error.status === 404) {
      notFound();
    }
    throw error;
  }

  if (session.status !== "live") {
    return (
      <SessionDetailView
        breadcrumb={
          <SessionBreadcrumb
            roomId={session.room_id}
            roomName={roomName}
            title={session.title}
            locale={locale}
          />
        }
        lifecycle={
          <SessionLifecyclePanel session={session} canManage={canManage} locale={locale} />
        }
        collaboration={
          session.status === "ended" ? (
            <CollaborationWorkspace sessionId={sessionId} locale={locale} />
          ) : null
        }
        activities={
          session.status === "ended" ? (
            <SessionActivitiesHistory sessionId={sessionId} locale={locale} />
          ) : null
        }
        participation={
          session.status === "ended" && canManage ? (
            <ParticipationSummary data={participation} locale={locale} />
          ) : null
        }
        files={
          <ContextFileLibrary
            context={{ kind: "session", id: sessionId }}
            locale={locale}
            canDelete={canManage}
            actorId={user?.userId}
          />
        }
        invites={
          canManage && session.status === "scheduled" ? (
            <InviteManager target="session" targetId={sessionId} locale={locale} />
          ) : null
        }
      />
    );
  }

  return (
    <SessionDetailView
      stage={
        <SessionClient
          actorId={user?.userId}
          actorName={profile?.display_name ?? user?.name}
          sessionId={sessionId}
          roomId={session.room_id}
          roomName={roomName}
          locale={locale}
          title={session.title}
          capabilities={capabilities}
          canAdmit={canManage}
          canDeleteFiles={canManage}
          sessionTiming={sessionTiming}
          endAction={
            canManage ? (
              <SessionLifecyclePanel session={session} canManage locale={locale} compact />
            ) : null
          }
          invites={
            canManage ? (
              <InviteManager target="session" targetId={sessionId} locale={locale} />
            ) : null
          }
        />
      }
    />
  );
}
