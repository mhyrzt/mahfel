import type { Meta, StoryObj } from "@storybook/nextjs";

import { FileList } from "@/features/files/components/file-list";
import { InviteLinkList } from "@/features/invites/components/invite-link-list";

import { initialSessionState, sessionReducer } from "../session-state";
import { ParticipationSummary } from "../components/participation-summary";
import { SessionDetailView } from "../components/session-detail-view";
import { SessionLifecyclePanel } from "../components/session-lifecycle-panel";
import { SessionScreen } from "../components/session-screen";

const session = {
  id: "weekly-discussion",
  room_id: "research-methods",
  title: "Weekly research discussion",
  title_overridden: true,
  status: "scheduled" as const,
  created_at: "2026-09-24T09:00:00Z",
  updated_at: "2026-09-24T09:00:00Z",
  scheduled_start_at: "2026-09-26T09:00:00Z",
  scheduled_duration_minutes: 60,
};
const files = [
  {
    id: "agenda",
    file_name: "Agenda.pdf",
    owner_id: "manager-1",
    context_id: session.id,
    context_kind: "session" as const,
    checksum_sha256: "fixture",
    created_at: "2026-09-24T09:00:00Z",
    updated_at: "2026-09-24T09:00:00Z",
    mime: "application/pdf",
    size_bytes: 1024,
    status: "ready",
  },
];
const liveState = sessionReducer(initialSessionState, {
  type: "session-connected",
  sessionId: session.id,
  participantId: "participant-1",
  roomName: session.title,
  participants: [
    { id: "participant-1", name: "Ava Rahimi", initials: "AR", role: "moderator", speaking: true },
    { id: "participant-2", name: "Sam Parsa", initials: "SP", role: "member" },
  ],
  capabilities: {
    canPublishAudio: true,
    canPublishVideo: true,
    canShareScreen: true,
    canModerate: true,
  },
});

const meta = {
  title: "Sessions/Detail Page",
  component: SessionDetailView,
} satisfies Meta<typeof SessionDetailView>;

export default meta;
type Story = StoryObj<typeof meta>;

export const ScheduledManager: Story = {
  render: () => (
    <SessionDetailView
      lifecycle={<SessionLifecyclePanel session={session} canManage locale="en" />}
      files={<FileList files={files} locale="en" canDelete />}
      invites={
        <InviteLinkList
          links={[
            {
              id: "invite-1",
              code: "abc-123",
              expiresAt: "Sep 30",
              status: "active",
              target: "session",
            },
          ]}
        />
      }
    />
  ),
};
export const ScheduledMember: Story = {
  render: () => (
    <SessionDetailView
      lifecycle={<SessionLifecyclePanel session={session} canManage={false} locale="en" />}
      files={<FileList files={files} locale="en" canDelete={false} actorId="member-1" />}
    />
  ),
};
export const LiveManager: Story = {
  render: () => (
    <SessionDetailView
      stage={
        <SessionScreen
          roomId={session.room_id}
          roomName="Research methods"
          title={session.title}
          state={liveState}
          locale="en"
          admissionRequests={[]}
          endAction={
            <SessionLifecyclePanel
              session={{ ...session, status: "live" }}
              canManage
              locale="en"
              compact
            />
          }
          files={<FileList files={files} locale="en" canDelete onPresent={() => undefined} />}
        />
      }
    />
  ),
};
export const EndedManager: Story = {
  render: () => (
    <SessionDetailView
      lifecycle={
        <SessionLifecyclePanel session={{ ...session, status: "ended" }} canManage locale="en" />
      }
      participation={
        <ParticipationSummary
          locale="en"
          data={{
            session_id: session.id,
            participants: [
              {
                participant_id: "participant-1",
                actor_id: "manager-1",
                connected_seconds: 3180,
                corrected_intervals: 0,
              },
            ],
          }}
        />
      }
      files={<FileList files={files} locale="en" canDelete />}
    />
  ),
};
export const PersianMember: Story = {
  render: () => (
    <div dir="rtl" lang="fa">
      <SessionDetailView
        lifecycle={<SessionLifecyclePanel session={session} canManage={false} locale="fa" />}
        files={<FileList files={files} locale="fa" canDelete={false} />}
      />
    </div>
  ),
};
