import type { Meta, StoryObj } from "@storybook/nextjs";

import { AccountSettingsPanel } from "../components/account-settings-panel";

const meta = {
  title: "Account/Settings",
  component: AccountSettingsPanel,
  parameters: { layout: "centered" },
  args: {
    calendar: "gregorian",
    email: "roya@example.com",
    image: null,
    locale: "en",
    name: "Roya Azadi",
    theme: "system",
  },
  decorators: [
    (Story) => (
      <div className="w-[min(56rem,calc(100vw-2rem))] rounded-2xl border bg-background p-6">
        <Story />
      </div>
    ),
  ],
} satisfies Meta<typeof AccountSettingsPanel>;

export default meta;

type Story = StoryObj<typeof meta>;

export const Default: Story = {};

export const Persian: Story = {
  args: { locale: "fa", name: "رویا آزادی" },
  decorators: [
    (Story) => (
      <div
        dir="rtl"
        lang="fa"
        className="w-[min(56rem,calc(100vw-2rem))] rounded-2xl border bg-background p-6"
      >
        <Story />
      </div>
    ),
  ],
};
