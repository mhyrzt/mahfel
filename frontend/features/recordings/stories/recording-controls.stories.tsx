import type { Meta, StoryObj } from "@storybook/nextjs";
import { useEffect, useRef, useState, type ComponentProps } from "react";
import { expect, fn, userEvent, waitFor, within } from "storybook/test";
import { SessionToolbar } from "@/features/sessions/components/session-toolbar";

import { RecordingControlsView } from "../recording-controls-view";

type Props = ComponentProps<typeof RecordingControlsView>;

function recording(status: string, partial = false, warning: string | null = null) {
  return { status, partial, warning };
}

const meta = {
  title: "Recordings/Toolbar Menu",
  component: RecordingControlsView,
  parameters: { layout: "fullscreen" },
  decorators: [
    (Story, context) => (
      <div
        className="flex min-h-80 w-full items-end justify-center bg-muted/40 px-4 pb-6"
        dir={context.args.locale === "fa" ? "rtl" : "ltr"}
        lang={context.args.locale}
      >
        <SessionToolbar
          locale={context.args.locale}
          recordingControl={<Story />}
          onToggleMicrophone={fn()}
          onToggleCamera={fn()}
          onToggleScreenShare={fn()}
        />
      </div>
    ),
  ],
  args: {
    locale: "en",
    recording: null,
    verified: true,
    canControl: true,
    busy: false,
    commandFailed: false,
    unavailable: false,
    onCommand: fn(),
  },
} satisfies Meta<typeof RecordingControlsView>;

export default meta;
type Story = StoryObj<typeof meta>;

const openMenu: NonNullable<Story["play"]> = async ({ canvas, canvasElement, args }) => {
  await userEvent.click(canvas.getByRole("button", { name: /Session recording|ضبط دیدار/ }));
  const menu = await within(canvasElement.ownerDocument.body).findByRole("menu");
  await expect(menu).toHaveAttribute("dir", args.locale === "fa" ? "rtl" : "ltr");
};

export const Off: Story = {};
export const OpenMenu: Story = { args: { recording: recording("recording") }, play: openMenu };
export const Starting: Story = { args: { recording: recording("starting") } };
export const Recording: Story = { args: { recording: recording("recording") } };
export const Paused: Story = { args: { recording: recording("paused") } };
export const Stopping: Story = { args: { recording: recording("stopping") } };
export const Finalizing: Story = { args: { recording: recording("finalizing") } };
export const Ready: Story = { args: { recording: recording("ready") } };
export const PartialReady: Story = {
  args: { recording: recording("ready", true, "partial_recording") },
};
export const Failed: Story = {
  args: { recording: recording("failed", true, "unusable_recording") },
};
export const CaptureFailure: Story = {
  args: { recording: recording("recording", true, "upload_unavailable") },
};
export const MemberNotice: Story = {
  args: { recording: recording("recording"), canControl: false },
  play: async (context) => {
    const { canvas, canvasElement } = context;
    await expect(canvas.getByRole("status")).toHaveTextContent("Recording");
    await openMenu(context);
    await expect(
      within(canvasElement.ownerDocument.body).queryByRole("menuitem"),
    ).not.toBeInTheDocument();
  },
};
export const MemberFailureNotice: Story = {
  args: { recording: recording("recording", true, "spool_exhausted"), canControl: false },
};
export const Loading: Story = { args: { verified: false } };
export const StatusUnavailable: Story = { args: { verified: false, unavailable: true } };
export const CommandPending: Story = {
  args: { recording: recording("recording"), busy: true },
  play: async (context) => {
    await openMenu(context);
    const body = within(context.canvasElement.ownerDocument.body);
    for (const item of body.getAllByRole("menuitem")) {
      await expect(item).toHaveAttribute("aria-disabled", "true");
    }
  },
};
export const CommandFailure: Story = {
  args: { recording: recording("paused"), commandFailed: true },
};
export const PersianRecording: Story = {
  args: { locale: "fa", recording: recording("recording") },
  play: openMenu,
};
export const PersianPaused: Story = {
  args: { locale: "fa", recording: recording("paused") },
};
export const PersianFailure: Story = {
  args: { locale: "fa", recording: recording("recording", true, "capture_unavailable") },
  play: openMenu,
};
export const NarrowPersian: Story = {
  args: { locale: "fa", recording: recording("recording", true, "spool_exhausted") },
  decorators: [
    (Story) => (
      <div className="mx-auto w-full max-w-sm">
        <Story />
      </div>
    ),
  ],
  play: openMenu,
};

function InteractiveRecording(args: Props) {
  const [current, setCurrent] = useState<Props["recording"]>(null);
  const [busy, setBusy] = useState(false);
  const timer = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);
  useEffect(() => () => clearTimeout(timer.current), []);

  function command(action: Parameters<Props["onCommand"]>[0]) {
    args.onCommand(action);
    setBusy(true);
    if (action === "start" || action === "resume") {
      setCurrent(recording("starting"));
    } else if (action === "stop") {
      setCurrent(recording("stopping"));
    }
    timer.current = setTimeout(() => {
      const next = { start: "recording", pause: "paused", resume: "recording", stop: "ready" };
      setCurrent(recording(next[action]));
      setBusy(false);
    }, 400);
  }

  return <RecordingControlsView {...args} recording={current} busy={busy} onCommand={command} />;
}

export const InteractiveLifecycle: Story = {
  render: (args) => <InteractiveRecording {...args} />,
  play: async (context) => {
    const { canvas, canvasElement, args } = context;
    await openMenu(context);
    const body = within(canvasElement.ownerDocument.body);
    await userEvent.click(body.getByRole("menuitem", { name: "Start recording" }));
    await waitFor(() => expect(canvas.getByRole("status")).toHaveTextContent(/^Recording$/));
    await userEvent.click(body.getByRole("menuitem", { name: "Pause recording" }));
    await waitFor(() => expect(canvas.getByRole("status")).toHaveTextContent("Recording paused"));
    await userEvent.click(body.getByRole("menuitem", { name: "Resume recording" }));
    await waitFor(() => expect(canvas.getByRole("status")).toHaveTextContent(/^Recording$/));
    await userEvent.click(body.getByRole("menuitem", { name: "Stop recording" }));
    await waitFor(() => expect(canvas.getByRole("status")).toHaveTextContent("Recording ready"));
    await expect(args.onCommand.mock.calls.map(([action]) => action)).toEqual([
      "start",
      "pause",
      "resume",
      "stop",
    ]);
  },
};
