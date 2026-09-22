import type { Meta, StoryObj } from "@storybook/nextjs";

import { WaitingRoomPanel } from "../components/waiting-room-panel";

const meta = {
  title: "Sessions/Waiting Room Panel",
  component: WaitingRoomPanel,
  args: { roomName: "Research methods" },
} satisfies Meta<typeof WaitingRoomPanel>;

export default meta;

type Story = StoryObj<typeof meta>;

export const Waiting: Story = {};

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
