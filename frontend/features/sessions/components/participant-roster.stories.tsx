import type { Meta, StoryObj } from "@storybook/nextjs";

import { ParticipantRoster } from "./participant-roster";

const participants = [
  { id: "a", name: "Ritsuko Akagi", initials: "RA", role: "moderator" as const },
  { id: "b", name: "Asuka Langley", initials: "AL", role: "member" as const, muted: true },
  { id: "c", name: "Shinji Ikari", initials: "SI", role: "member" as const },
];

const meta = {
  title: "Sessions/Participant Roster",
  component: ParticipantRoster,
  args: { participants },
} satisfies Meta<typeof ParticipantRoster>;

export default meta;

type Story = StoryObj<typeof meta>;

export const Populated: Story = {};

export const Empty: Story = { args: { participants: [] } };
