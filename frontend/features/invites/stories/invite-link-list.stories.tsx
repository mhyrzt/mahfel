import type { Meta, StoryObj } from "@storybook/nextjs";

import { InviteLinkList } from "../components/invite-link-list";

const links = [
  {
    id: "a",
    code: "qwe-rty-uio",
    expiresAt: "Sep 29",
    status: "active" as const,
    target: "room" as const,
  },
  {
    id: "b",
    code: "asd-fgh-jkl",
    expiresAt: "Sep 18",
    status: "expired" as const,
    target: "session" as const,
  },
  {
    id: "c",
    code: "zxc-vbn-mas",
    expiresAt: "Sep 24",
    status: "revoked" as const,
    target: "room" as const,
  },
];

const meta = {
  title: "Invites/Invite Link List",
  component: InviteLinkList,
  args: { links },
} satisfies Meta<typeof InviteLinkList>;

export default meta;

type Story = StoryObj<typeof meta>;

export const Manageable: Story = {};

export const CapabilityRestricted: Story = { args: { canManage: false } };

export const Empty: Story = { args: { links: [] } };
