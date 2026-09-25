import type { Meta, StoryObj } from "@storybook/nextjs";

import { ChatMessageList } from "@/features/chat/components/chat-message-list";
import { FileList } from "@/features/files/components/file-list";
import { InviteLinkList } from "@/features/invites/components/invite-link-list";
import { AdmissionQueue } from "@/features/sessions/components/admission-queue";
import { roomCopy } from "@/features/rooms/room-copy";

import { RoomAdmissionSettings } from "../room-admission-settings";
import { RoomDetailView, RoomHeader, RoomSessions } from "../room-detail";

const room = {
  id: "research-methods",
  title: "Research methods",
  status: "active",
  visibility: "invite_only" as const,
  admission: "lobby" as const,
  created_by: "manager-1",
};
const sessions = [
  {
    id: "live-workshop",
    room_id: room.id,
    title: "Methods workshop",
    title_overridden: true,
    status: "live" as const,
    scheduled_start_at: "2026-09-25T13:00:00Z",
    scheduled_duration_minutes: 90,
    started_at: "2026-09-25T12:58:00Z",
    created_at: "2026-09-20T09:00:00Z",
    updated_at: "2026-09-25T12:58:00Z",
  },
  {
    id: "weekly-discussion",
    room_id: room.id,
    title: "Weekly research discussion",
    title_overridden: true,
    status: "scheduled" as const,
    scheduled_start_at: "2026-09-27T09:00:00Z",
    scheduled_duration_minutes: 60,
    created_at: "2026-09-24T09:00:00Z",
    updated_at: "2026-09-24T09:00:00Z",
  },
  {
    id: "prior-review",
    room_id: room.id,
    title: "Prior review",
    title_overridden: true,
    status: "ended" as const,
    started_at: "2026-09-19T09:00:00Z",
    ended_at: "2026-09-19T10:15:00Z",
    created_at: "2026-09-18T09:00:00Z",
    updated_at: "2026-09-19T10:15:00Z",
  },
];
const files = [
  {
    id: "reading-list",
    file_name: "Reading list.pdf",
    owner_id: "manager-1",
    context_id: room.id,
    context_kind: "room" as const,
    checksum_sha256: "fixture",
    created_at: "2026-09-24T09:00:00Z",
    updated_at: "2026-09-24T09:00:00Z",
    mime: "application/pdf",
    size_bytes: 2048,
    status: "ready",
  },
];

function ChatFixture({ locale = "en" }: Readonly<{ locale?: "en" | "fa" }>) {
  const text = roomCopy(locale);
  return (
    <section aria-label={text.chat} className="space-y-4" data-testid="room-chat">
      <h2 className="text-lg font-semibold">{text.chat}</h2>
      <ChatMessageList
        roomId={room.id}
        locale={locale}
        actorId="manager-1"
        messages={[
          {
            id: "message-1",
            author_id: "participant-8f3d",
            channel_id: room.id,
            content: "See the reading list before Friday.",
            created_at: "2026-09-24T09:00:00Z",
          },
          {
            id: "message-2",
            author_id: "manager-1",
            channel_id: room.id,
            content: "Thanks, I’ll add notes here.",
            created_at: "2026-09-24T09:01:00Z",
          },
        ]}
      />
    </section>
  );
}

const meta = {
  title: "Rooms/Detail Page",
  component: RoomDetailView,
  args: {
    room,
    sessions,
    locale: "en",
    canManageSession: true,
    chat: <ChatFixture />,
    files: <FileList files={files} locale="en" canDelete actorId="manager-1" />,
    invites: (
      <InviteLinkList
        links={[
          {
            id: "invite-1",
            code: "abc-123",
            expiresAt: "Sep 30",
            status: "active",
            target: "room",
          },
        ]}
      />
    ),
    lobby: <AdmissionQueue requests={[]} />,
    admissionSettings: <RoomAdmissionSettings roomId={room.id} initialPolicy="lobby" locale="en" />,
  },
} satisfies Meta<typeof RoomDetailView>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Manager: Story = {};
export const Member: Story = {
  args: {
    canManageSession: false,
    invites: null,
    lobby: null,
    admissionSettings: null,
    files: <FileList files={files} locale="en" canDelete={false} actorId="member-1" />,
  },
};
export const EmptyRoom: Story = {
  args: {
    sessions: [],
    chat: <ChatMessageList roomId={room.id} locale="en" messages={[]} />,
    files: <FileList files={[]} locale="en" canDelete={false} />,
    invites: null,
    lobby: null,
    admissionSettings: null,
  },
};
export const Header: Story = {
  render: () => <RoomHeader room={room} locale="en" canManageSession />,
};
export const SessionList: Story = {
  render: () => <RoomSessions sessions={sessions} locale="en" />,
};
export const Persian: Story = {
  args: {
    locale: "fa",
    chat: <ChatFixture locale="fa" />,
    files: <FileList files={files} locale="fa" canDelete={false} />,
    invites: null,
    lobby: null,
    admissionSettings: null,
  },
  decorators: [
    (Story) => (
      <div dir="rtl" lang="fa">
        <Story />
      </div>
    ),
  ],
};
