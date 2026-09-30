import type { Meta, StoryObj } from "@storybook/nextjs";
import { RecordingScenePreview } from "../recording-scene-preview";
import { replayFixture } from "../tests/replay-fixtures";

const input = replayFixture();
const meta = {
  title: "Features/Recordings/Export scenes",
  component: RecordingScenePreview,
  args: { input, timestamp: "2000000000" },
} satisfies Meta<typeof RecordingScenePreview>;
export default meta;
type Story = StoryObj<typeof meta>;
export const Whiteboard: Story = {};
export const ChatToasts: Story = { args: { timestamp: "2500000000" } };
export const Paused: Story = { args: { timestamp: "11000000000" } };
export const Resumed: Story = { args: { timestamp: "13000000000" } };
export const Cameras: Story = {
  args: {
    input: { ...input, snapshots: [], events: [] },
    tracks: Array.from({ length: 12 }, (_, index) => ({
      id: `camera-${index}`,
      actorId: `actor-${index}`,
      source: "camera" as const,
    })),
  },
};
