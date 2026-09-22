import type { Meta, StoryObj } from "@storybook/nextjs";

import { SessionStage } from "../components/session-stage";

const participants = [
  {
    id: "a",
    name: "Ritsuko Akagi",
    initials: "RA",
    role: "moderator" as const,
    speaking: true,
    videoEnabled: true,
  },
  {
    id: "b",
    name: "Asuka Langley",
    initials: "AL",
    role: "member" as const,
    muted: true,
    videoEnabled: false,
  },
  { id: "c", name: "Shinji Ikari", initials: "SI", role: "member" as const, videoEnabled: true },
  { id: "d", name: "Rei Ayanami", initials: "RA", role: "member" as const, videoEnabled: true },
];
const largerGroup = [
  ...participants,
  {
    id: "e",
    name: "Misato Katsuragi",
    initials: "MK",
    role: "member" as const,
    videoEnabled: true,
  },
  { id: "f", name: "Kaworu Nagisa", initials: "KN", role: "member" as const, videoEnabled: true },
];

const meta = {
  title: "Sessions/Session Stage",
  component: SessionStage,
  args: {
    title: "Weekly research discussion",
    status: "live",
    participants,
    participantActions: ["mute", "remove", "block"],
  },
} satisfies Meta<typeof SessionStage>;

export default meta;

type Story = StoryObj<typeof meta>;

export const Live: Story = {};

export const Solo: Story = {
  args: { participants: participants.slice(0, 1), participantActions: [] },
};

export const CameraOff: Story = {
  args: { participants: participants.slice(1, 2), participantActions: [] },
};

export const Pair: Story = {
  args: { participants: participants.slice(0, 2), participantActions: [] },
};

export const LargerGroup: Story = {
  args: { participants: largerGroup },
};

export const Scheduled: Story = {
  args: { status: "scheduled", participants: [], scheduledFor: "Thursday at 10:00" },
};

export const Ended: Story = {
  args: { status: "ended", participants: [] },
};

export const NarrowViewport: Story = {
  parameters: { viewport: { defaultViewport: "mobile1" } },
};

export const MemberView: Story = {
  args: { participantActions: [] },
};

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
