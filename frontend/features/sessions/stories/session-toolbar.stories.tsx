import type { Meta, StoryObj } from "@storybook/nextjs";

import { SessionToolbar } from "../components/session-toolbar";

const meta = {
  title: "Sessions/Toolbar",
  component: SessionToolbar,
} satisfies Meta<typeof SessionToolbar>;

export default meta;
type Story = StoryObj<typeof meta>;

export const MediaUnavailable: Story = {};
export const ListenOnly: Story = {
  args: { canPublishAudio: false, canPublishVideo: false, canShareScreen: false },
};
export const Persian: Story = {
  args: { locale: "fa" },
  decorators: [
    (Story) => (
      <div dir="rtl">
        <Story />
      </div>
    ),
  ],
};
