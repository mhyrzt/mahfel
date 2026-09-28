import type { Meta, StoryObj } from "@storybook/nextjs";

import { ChatMessageList } from "../components/chat-message-list";

const messages = [
  {
    id: "message-1",
    channel_id: "room-1",
    author_id: "alex-1",
    content: "I added the draft agenda to the room files.",
    created_at: "2026-09-25T09:00:00Z",
  },
  {
    id: "message-2",
    channel_id: "room-1",
    author_id: "alex-1",
    content: "The slide deck is in the session attachments too.",
    created_at: "2026-09-25T09:00:30Z",
  },
  {
    id: "message-3",
    channel_id: "room-1",
    author_id: "me",
    content: "Thanks. I’ll review it before the next session.",
    created_at: "2026-09-25T09:01:00Z",
  },
  {
    id: "message-4",
    channel_id: "room-1",
    author_id: "me",
    content: "I left a few comments on the second page.",
    created_at: "2026-09-25T09:02:00Z",
  },
];

const sessionMessages = messages.map((message) => ({
  ...message,
  session_id: "session-1",
}));

const mediaAttachments = [
  {
    id: "attachment-1",
    owner_id: "alex-1",
    context_kind: "chat_message" as const,
    context_id: "message-1",
    status: "ready",
    checksum_sha256: "demo",
    created_at: "2026-09-25T09:00:00Z",
    updated_at: "2026-09-25T09:00:00Z",
    file_name: "whiteboard.png",
    mime: "image/png",
    size_bytes: 20_480,
  },
  {
    id: "attachment-2",
    owner_id: "alex-1",
    context_kind: "chat_message" as const,
    context_id: "message-1",
    status: "ready",
    checksum_sha256: "demo",
    created_at: "2026-09-25T09:00:00Z",
    updated_at: "2026-09-25T09:00:00Z",
    file_name: "diagram.png",
    mime: "image/png",
    size_bytes: 31_744,
  },
];

const videoAttachments = [
  {
    id: "attachment-3",
    owner_id: "alex-1",
    context_kind: "chat_message" as const,
    context_id: "message-1",
    status: "ready",
    checksum_sha256: "demo",
    created_at: "2026-09-25T09:00:00Z",
    updated_at: "2026-09-25T09:00:00Z",
    file_name: "lecture.mp4",
    mime: "video/mp4",
    size_bytes: 8_000_000,
  },
];

const noop = () => undefined;

const meta = {
  title: "Chat/Message List",
  component: ChatMessageList,
  args: {
    messages,
    roomId: "room-1",
    locale: "en",
    actorId: "me",
    onEdit: noop,
    onRemove: noop,
    onReply: noop,
    onToggleReplies: noop,
  },
  decorators: [
    (Story) => (
      <div className="mx-auto max-w-2xl p-6">
        <Story />
      </div>
    ),
  ],
} satisfies Meta<typeof ChatMessageList>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Conversation: Story = {};
export const WithActions: Story = { args: { canModerate: true } };
export const SessionScoped: Story = { args: { messages: sessionMessages } };
export const MediaAlbum: Story = {
  args: {
    messages: [
      {
        ...messages[0],
        content: "Here are the board photos from today.",
        attachments: mediaAttachments,
      },
    ],
  },
};
export const VideoAttachment: Story = {
  args: {
    messages: [
      {
        ...messages[2],
        content: "Recording is up.",
        attachments: videoAttachments,
      },
    ],
  },
};
export const Edited: Story = {
  args: {
    messages: [
      {
        ...messages[0],
        content: "I added the finalized agenda to the room files.",
        edited_at: "2026-09-25T09:05:00Z",
      },
      messages[3],
    ],
  },
};
export const Empty: Story = { args: { messages: [] } };
export const Persian: Story = {
  args: { locale: "fa" },
  decorators: [
    (Story) => (
      <div dir="rtl" lang="fa">
        <Story />
      </div>
    ),
  ],
};
