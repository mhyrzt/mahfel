import type { Meta, StoryObj } from "@storybook/nextjs";

import { SessionLifecyclePanel } from "../components/session-lifecycle-panel";

const session = {
  id: "weekly-discussion",
  room_id: "research-methods",
  title: "Weekly research discussion",
  title_overridden: true,
  status: "scheduled" as const,
  created_at: "2026-09-24T09:00:00Z",
  updated_at: "2026-09-24T09:00:00Z",
  scheduled_start_at: "2026-09-26T09:00:00Z",
  scheduled_duration_minutes: 60,
};

const meta = {
  title: "Sessions/Lifecycle",
  component: SessionLifecyclePanel,
  args: { session, canManage: true, locale: "en" },
} satisfies Meta<typeof SessionLifecyclePanel>;

export default meta;
type Story = StoryObj<typeof meta>;

export const ScheduledManager: Story = {};
export const ScheduledMember: Story = { args: { canManage: false } };
export const LiveManager: Story = { args: { session: { ...session, status: "live" } } };
export const Ended: Story = { args: { session: { ...session, status: "ended" } } };
export const Cancelled: Story = { args: { session: { ...session, status: "cancelled" } } };
