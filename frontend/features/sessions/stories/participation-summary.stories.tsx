import type { Meta, StoryObj } from "@storybook/nextjs";

import { ParticipationSummary } from "../components/participation-summary";

const data = {
  session_id: "weekly-discussion",
  participants: [
    {
      participant_id: "one",
      actor_id: "8f3d1188-0123-4567-8901-000000000001",
      connected_seconds: 3180,
      corrected_intervals: 0,
    },
    {
      participant_id: "two",
      actor_id: "9a4e2299-0123-4567-8901-000000000002",
      connected_seconds: 1440,
      corrected_intervals: 1,
    },
  ],
};

const meta = {
  title: "Sessions/Participation Summary",
  component: ParticipationSummary,
  args: { data, locale: "en" },
} satisfies Meta<typeof ParticipationSummary>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Participants: Story = {};
export const Empty: Story = { args: { data: { ...data, participants: [] } } };
export const Unavailable: Story = { args: { data: null } };
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
