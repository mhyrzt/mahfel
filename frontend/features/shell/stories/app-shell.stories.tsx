import type { Meta, StoryObj } from "@storybook/nextjs";
import { expect } from "storybook/test";

import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";

import { AppShell } from "../components/app-shell";

const meta = {
  title: "Shell/App Shell",
  component: AppShell,
  parameters: { layout: "fullscreen" },
  play: async ({ canvasElement, args }) => {
    const rail = canvasElement.querySelector("aside");
    const content = canvasElement.querySelector("main");
    if (!rail || !content || !rail.parentElement) {
      throw new Error("Shell rail and content must be rendered");
    }

    const shellBounds = rail.parentElement.getBoundingClientRect();
    const contentBounds = content.getBoundingClientRect();
    const railBounds = rail.getBoundingClientRect();
    const desktop = getComputedStyle(rail).display !== "none";
    const rtl = args.locale === "fa";

    await expect(contentBounds.left).toBe(shellBounds.left + (desktop && !rtl ? 64 : 0));
    await expect(contentBounds.right).toBe(shellBounds.right - (desktop && rtl ? 64 : 0));
    if (desktop) {
      await expect(railBounds.width).toBe(64);
      await expect(rtl ? railBounds.right : railBounds.left).toBe(
        rtl ? shellBounds.right : shellBounds.left,
      );
    }
  },
  args: {
    accountName: "Roya Azadi",
    accountEmail: "roya@example.com",
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
  globals: { viewport: { value: "mobile1", isRotated: false } },
};

export const PersianRTL: Story = {
  args: { locale: "fa", accountName: "رویا آزادی" },
};

export const PersianNarrowViewport: Story = {
  ...PersianRTL,
  globals: { viewport: { value: "mobile1", isRotated: false } },
};
