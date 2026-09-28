import type { Meta, StoryObj } from "@storybook/nextjs";

import { AppShell } from "@/features/shell/components/app-shell";

import { WorkspaceHome } from "../components/workspace-home";

const rooms = [
  {
    id: "1",
    name: "NERV briefing",
    description: "Operations planning and weekly coordination.",
    status: "live" as const,
    sessionLabel: "In progress",
    participantCount: 5,
  },
  {
    id: "2",
    name: "Pilot study group",
    description: "Preparation, reading notes, and peer discussion.",
    status: "scheduled" as const,
    sessionLabel: "14:30 today",
    participantCount: 4,
  },
  {
    id: "3",
    name: "Technical review",
    description: "Architecture discussions and implementation review.",
    status: "idle" as const,
    participantCount: 3,
  },
  {
    id: "4",
    name: "Research archive",
    description: "Shared material and asynchronous questions.",
    status: "idle" as const,
    participantCount: 6,
  },
];

const sessions = [
  {
    id: "1",
    title: "Pilot preparation",
    roomName: "Pilot study group",
    scheduledAt: "2026-09-22T14:30:00Z",
    durationMinutes: 45,
    participantCount: 4,
  },
  {
    id: "2",
    title: "Technical review",
    roomName: "Technical review",
    scheduledAt: "2026-09-22T17:00:00Z",
    durationMinutes: 60,
    participantCount: 3,
  },
];

const meta = {
  title: "Workspace/Workspace Home",
  component: WorkspaceHome,
  parameters: { layout: "fullscreen" },
  decorators: [
    (Story) => (
      <AppShell>
        <Story />
      </AppShell>
    ),
  ],
  args: { rooms, sessions, referenceDate: "2026-09-22T12:00:00Z" },
} satisfies Meta<typeof WorkspaceHome>;

export default meta;

type Story = StoryObj<typeof meta>;

export const Populated: Story = {};

export const Empty: Story = { args: { rooms: [], sessions: [] } };

export const NarrowViewport: Story = { parameters: { viewport: { defaultViewport: "mobile1" } } };
