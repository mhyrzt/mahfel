import type { Meta, StoryObj } from "@storybook/nextjs";
import type { components } from "@/lib/api/generated/schema";

import { InviteLinkList } from "@/features/invites/components/invite-link-list";
import { AdmissionQueue } from "@/features/sessions/components/admission-queue";

import { RoomAdmissionSettings } from "../room-admission-settings";
import { RoomDetailView, RoomHeader, RoomSessions } from "../room-detail";
import { RoomMembers } from "../room-members";
import { RoomSettingsDialog } from "../room-settings-dialog";
import { SessionSettingsDialog } from "../session-settings-dialog";

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
type FileRecord = components["schemas"]["FileResponse"];
const roomFiles: FileRecord[] = [
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
const sessionFile: FileRecord = {
  ...roomFiles[0],
  id: "workshop-handout",
  file_name: "Workshop handout.pdf",
  context_id: "live-workshop",
  context_kind: "session" as const,
};
const fileData = Object.fromEntries(
  sessions.map((session) => [
    session.id,
    {
      files: session.id === "live-workshop" ? [sessionFile] : [],
      usedIds: [],
      roomFiles,
      attachRoomFile: async (file: FileRecord) => file,
      uploadSessionFile: async (file: File) => ({
        ...sessionFile,
        id: `demo-${file.name}`,
        file_name: file.name,
        mime: file.type || "application/octet-stream",
        size_bytes: file.size,
      }),
    },
  ]),
);
const populatedSessions = [
  ...sessions,
  ...Array.from({ length: 10 }, (_, index) => ({
    ...sessions[1],
    id: `weekly-discussion-${index + 2}`,
    title: `Weekly research discussion ${index + 2}`,
    scheduled_start_at: new Date(Date.UTC(2026, 8, 28 + index * 7, 9)).toISOString(),
  })),
];
const populatedFileData = Object.fromEntries(
  populatedSessions.map((session) => [
    session.id,
    fileData[session.id] ?? {
      files: [],
      usedIds: [],
      roomFiles,
      attachRoomFile: async (file: FileRecord) => file,
      uploadSessionFile: async (file: File) => ({
        ...sessionFile,
        id: `demo-${file.name}`,
        file_name: file.name,
        mime: file.type || "application/octet-stream",
        size_bytes: file.size,
      }),
    },
  ]),
);

const managerSettings = (
  <RoomSettingsDialog
    locale="en"
    requests={
      <div className="space-y-5">
        <RoomAdmissionSettings roomId={room.id} initialPolicy="lobby" locale="en" />
        <AdmissionQueue requests={[]} />
      </div>
    }
    members={
      <RoomMembers
        roomId={room.id}
        locale="en"
        actorId="manager-1"
        initialMembers={[
          { user_id: "manager-1", role: "manager" },
          { user_id: "member-2", role: "member" },
        ]}
      />
    }
    invites={
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
    }
  />
);

const meta = {
  title: "Rooms/Detail Page",
  component: RoomDetailView,
  args: {
    room,
    sessions,
    locale: "en",
    canManageSession: true,
    settings: managerSettings,
    fileData,
  },
} satisfies Meta<typeof RoomDetailView>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Manager: Story = {};
export const Member: Story = {
  args: {
    canManageSession: false,
    settings: null,
  },
};
export const EmptyRoom: Story = {
  args: {
    sessions: [],
    settings: null,
  },
};
export const Header: Story = {
  render: () => <RoomHeader room={room} locale="en" canManageSession />,
};
export const SettingsDialog: Story = {
  render: () => managerSettings,
};
export const SessionList: Story = {
  render: () => (
    <RoomSessions sessions={sessions} locale="en" fileData={fileData} canManageSession />
  ),
};
export const SessionSettings: Story = {
  render: () => <SessionSettingsDialog session={sessions[1]} locale="en" />,
};
export const PopulatedRoom: Story = {
  args: { sessions: populatedSessions, fileData: populatedFileData },
};
export const MobileRoom: Story = {
  args: { sessions: populatedSessions, fileData: populatedFileData },
  parameters: { viewport: { defaultViewport: "mobile1" } },
  decorators: [
    (Story) => (
      <div className="mx-auto max-w-[390px] px-3">
        <Story />
      </div>
    ),
  ],
};
export const Persian: Story = {
  args: {
    locale: "fa",
    settings: (
      <RoomSettingsDialog
        locale="fa"
        requests={
          <div className="space-y-5">
            <RoomAdmissionSettings roomId={room.id} initialPolicy="lobby" locale="fa" />
            <AdmissionQueue requests={[]} locale="fa" />
          </div>
        }
        members={
          <RoomMembers
            roomId={room.id}
            locale="fa"
            initialMembers={[{ user_id: "manager-1", role: "manager" }]}
          />
        }
        invites={<InviteLinkList links={[]} locale="fa" />}
      />
    ),
  },
  decorators: [
    (Story) => (
      <div dir="rtl" lang="fa">
        <Story />
      </div>
    ),
  ],
};
