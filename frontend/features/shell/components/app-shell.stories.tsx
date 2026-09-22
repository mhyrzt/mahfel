import type { Meta, StoryObj } from "@storybook/nextjs";

import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";

import { AppShell } from "./app-shell";

const meta = {
  title: "Shell/App Shell",
  component: AppShell,
  parameters: { layout: "fullscreen" },
  args: {
    children: (
      <Card>
        <CardHeader>
          <CardTitle>Welcome to Mahfel</CardTitle>
        </CardHeader>
        <CardContent className="text-sm text-muted-foreground">
          Workspace content renders here.
        </CardContent>
      </Card>
    ),
  },
} satisfies Meta<typeof AppShell>;

export default meta;

type Story = StoryObj<typeof meta>;

export const Default: Story = {};

export const NarrowViewport: Story = {
  parameters: { viewport: { defaultViewport: "mobile1" } },
};
