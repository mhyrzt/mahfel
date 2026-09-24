import { SessionClient } from "@/features/sessions/components/session-client";
import { initialSessionState, type SessionCapabilities } from "@/features/sessions/session-state";
import { SessionScreen } from "@/features/sessions/components/session-screen";
import { getLocale } from "@/lib/i18n/server";
import { createServerApiClient } from "@/lib/api/server";
import { messagesFor } from "@/lib/i18n/messages";
import { getServerSessionUser } from "@/lib/auth/server";
import { CollaborationWorkspace } from "@/features/sessions/components/collaboration-workspace";

interface SessionPageProps {
  params: Promise<{ sessionId: string }>;
}

interface SessionCapabilitiesResponse {
  capabilities: string[];
}

export default async function SessionPage({ params }: Readonly<SessionPageProps>) {
  const [{ sessionId }, locale, user] = await Promise.all([
    params,
    getLocale(),
    getServerSessionUser(),
  ]);
  let capabilities: SessionCapabilities | null = null;
  let archived = false;
  try {
    const api = await createServerApiClient();
    try {
      const collaboration = await api.get<{ live: boolean }>(
        `/v1/sessions/${encodeURIComponent(sessionId)}/collaboration`,
      );
      archived = !collaboration.data.live;
    } catch {}
    const { data } = await api.get<SessionCapabilitiesResponse>(
      `/v1/sessions/${encodeURIComponent(sessionId)}/capabilities`,
    );
    const granted = new Set(data.capabilities);
    capabilities = {
      canPublishAudio: granted.has("publish_media"),
      canPublishVideo: granted.has("publish_media"),
      canShareScreen: granted.has("share_screen"),
      canModerate: ["mute_others", "remove_participant", "admit_participant"].some((item) =>
        granted.has(item),
      ),
    };
  } catch {
    capabilities = null;
  }

  if (archived) {
    return <CollaborationWorkspace sessionId={sessionId} locale={locale} />;
  }

  if (!capabilities) {
    return (
      <SessionScreen
        title={messagesFor(locale).shell.sessions}
        locale={locale}
        state={{
          ...initialSessionState,
          phase: "unavailable",
          message: messagesFor(locale).sessions.retryMessage,
        }}
      />
    );
  }

  return (
    <SessionClient
      actorId={user?.userId}
      sessionId={sessionId}
      locale={locale}
      title={messagesFor(locale).sessions.sessionTitle}
      capabilities={capabilities}
    />
  );
}
