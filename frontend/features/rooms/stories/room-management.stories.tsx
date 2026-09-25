import type { Meta, StoryObj } from "@storybook/nextjs";

import { CreateRoomForm } from "../create-room-form";
import { RoomAdmissionSettings } from "../room-admission-settings";

const meta = {
  title: "Rooms/Management",
  component: CreateRoomForm,
  args: { locale: "en" },
} satisfies Meta<typeof CreateRoomForm>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Create: Story = {};
export const CreatePersian: Story = {
  args: { locale: "fa" },
  decorators: [
    (Story) => (
      <div dir="rtl">
        <Story />
      </div>
    ),
  ],
};
export const OpenAdmission: Story = {
  render: () => (
    <RoomAdmissionSettings roomId="research-methods" initialPolicy="open" locale="en" />
  ),
};
export const LobbyAdmission: Story = {
  render: () => <RoomAdmissionSettings roomId="project-studio" initialPolicy="lobby" locale="en" />,
};
