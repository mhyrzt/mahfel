import type { Meta, StoryObj } from "@storybook/nextjs";

import { PageSkeleton } from "../components/page-skeleton";
import { RouteError } from "../components/route-error";

const meta = {
  title: "Shell/Route Feedback",
  component: RouteError,
  args: { scope: "workspace", reset: () => undefined, locale: "en" },
} satisfies Meta<typeof RouteError>;

export default meta;
type Story = StoryObj<typeof meta>;

export const WorkspaceError: Story = {};
export const RoomError: Story = { args: { scope: "rooms" } };
export const PersianError: Story = {
  args: { scope: "sessions", locale: "fa" },
  decorators: [
    (Story) => (
      <div dir="rtl">
        <Story />
      </div>
    ),
  ],
};
export const RoomLoading: Story = { render: () => <PageSkeleton shape="detail" /> };
export const SessionLoading: Story = { render: () => <PageSkeleton shape="session" /> };
export const FormLoading: Story = { render: () => <PageSkeleton shape="form" /> };
