import type { Meta, StoryObj } from "@storybook/nextjs";

import { JoinByCodePanel } from "../components/join-by-code-panel";

const meta = {
  title: "Invites/Join by Code Panel",
  component: JoinByCodePanel,
} satisfies Meta<typeof JoinByCodePanel>;

export default meta;

type Story = StoryObj<typeof meta>;

export const Entry: Story = { args: { status: "entry" } };

export const Waiting: Story = { args: { status: "waiting" } };

export const Approved: Story = { args: { status: "approved" } };

export const Rejected: Story = { args: { status: "rejected" } };

export const Unavailable: Story = { args: { status: "unavailable" } };
