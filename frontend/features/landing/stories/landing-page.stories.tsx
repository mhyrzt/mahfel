import type { Meta, StoryObj } from "@storybook/nextjs";

import { LandingPageView } from "../components/landing-page-view";

const meta = {
  title: "Pages/Landing",
  component: LandingPageView,
  args: { locale: "en", theme: "light" },
  parameters: { layout: "fullscreen" },
} satisfies Meta<typeof LandingPageView>;

export default meta;
type Story = StoryObj<typeof meta>;

export const English: Story = {};
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
