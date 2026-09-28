import type { Meta, StoryObj } from "@storybook/nextjs";

import { PresentationPlayer } from "../components/presentation-player";

const meta = {
  title: "Sessions/Presentation Player",
  component: PresentationPlayer,
  args: {
    url: "https://interactive-examples.mdn.mozilla.net/media/cc0-videos/flower.mp4",
    canPresent: true,
    playback: { positionMs: 0, paused: true, rate: 1 },
    locale: "en",
    onPlayback: () => undefined,
  },
  parameters: { layout: "fullscreen" },
  decorators: [
    (Story) => (
      <div className="h-96 w-full p-4">
        <Story />
      </div>
    ),
  ],
} satisfies Meta<typeof PresentationPlayer>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Presenter: Story = {};
export const Viewer: Story = { args: { canPresent: false } };
export const PersianViewer: Story = {
  args: { canPresent: false, locale: "fa" },
  decorators: [
    (Story) => (
      <div dir="rtl" lang="fa" className="h-96 w-full p-4">
        <Story />
      </div>
    ),
  ],
};
