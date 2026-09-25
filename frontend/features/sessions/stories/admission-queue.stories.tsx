import type { Meta, StoryObj } from "@storybook/nextjs";

import { AdmissionQueue } from "../components/admission-queue";

const meta = {
  title: "Sessions/Admission Queue",
  component: AdmissionQueue,
  args: {
    requests: [
      { id: "a", name: "Misato Katsuragi", initials: "MK", requestedAt: minutesAgo(2) },
      { id: "b", name: "Rei Ayanami", initials: "RA", requestedAt: minutesAgo(0) },
    ],
  },
} satisfies Meta<typeof AdmissionQueue>;

export default meta;

type Story = StoryObj<typeof meta>;

function minutesAgo(minutes: number) {
  return new Date(Date.now() - minutes * 60_000).toISOString();
}

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
