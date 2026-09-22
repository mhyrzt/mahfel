import type { Meta, StoryObj } from "@storybook/nextjs";

import { AdmissionQueue } from "../components/admission-queue";

const meta = {
  title: "Sessions/Admission Queue",
  component: AdmissionQueue,
  args: {
    requests: [
      { id: "a", name: "Misato Katsuragi", initials: "MK", requestedAt: "2 minutes ago" },
      { id: "b", name: "Rei Ayanami", initials: "RA", requestedAt: "just now" },
    ],
  },
} satisfies Meta<typeof AdmissionQueue>;

export default meta;

type Story = StoryObj<typeof meta>;

export const Waiting: Story = {};

export const Empty: Story = { args: { requests: [] } };

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
