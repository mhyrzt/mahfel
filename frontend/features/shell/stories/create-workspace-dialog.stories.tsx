import type { Meta, StoryObj } from "@storybook/nextjs";

import { CreateWorkspaceDialog } from "../components/create-workspace-dialog";

const room = {
  id: "research-methods",
  title: "Research methods",
  status: "active",
  visibility: "invite_only" as const,
  admission: "lobby" as const,
  created_by: "manager-1",
};

const meta = {
  title: "Shell/Create Dialog",
  component: CreateWorkspaceDialog,
  args: { kind: "room", locale: "en", open: true, onOpenChange: () => undefined },
} satisfies Meta<typeof CreateWorkspaceDialog>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Room: Story = {};
export const Session: Story = {
  args: { kind: "session", initialRooms: [room], initialRoomId: room.id, initialCanManage: true },
};
export const PersianRoom: Story = {
  args: { locale: "fa" },
  decorators: [
    (Story) => (
      <div dir="rtl" lang="fa">
        <Story />
      </div>
    ),
  ],
};
