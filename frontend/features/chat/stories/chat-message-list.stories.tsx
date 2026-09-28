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
    author_id: "me",
    content: "Thanks. I’ll review it before the next session.",
    created_at: "2026-09-25T09:01:00Z",
  },
  {
    id: "message-3",
    channel_id: "room-1",
    author_id: "me",
    content: "I left a few comments on the second page.",
    created_at: "2026-09-25T09:02:00Z",
  },
];

const meta = {
  title: "Chat/Message List",
  component: ChatMessageList,
  args: { messages, roomId: "room-1", locale: "en", actorId: "me" },
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
