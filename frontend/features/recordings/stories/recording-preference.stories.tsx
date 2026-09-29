import type { Meta, StoryObj } from "@storybook/nextjs";
import { expect, userEvent, within } from "storybook/test";

import { SessionSettingsDialog } from "@/features/rooms/session-settings-dialog";
import { roomCopy } from "@/features/rooms/room-copy";
import { CreateSessionForm } from "@/features/sessions/components/create-session-form";
import type { components } from "@/lib/api/generated/schema";

const session = {
  id: "recording-preference-demo",
  room_id: "research-methods",
  title: "Weekly research discussion",
  title_overridden: true,
  status: "scheduled",
  auto_record: true,
  created_at: "2026-09-28T09:00:00Z",
  updated_at: "2026-09-28T09:00:00Z",
  scheduled_start_at: "2026-10-01T09:00:00Z",
  scheduled_duration_minutes: 60,
} satisfies components["schemas"]["SessionResponse"];

const rooms = [
  {
    id: session.room_id,
    title: "Research methods",
    status: "active",
    visibility: "invite_only",
    admission: "open",
    created_by: "manager-demo",
  },
] satisfies components["schemas"]["RoomResponse"][];

const meta = {
  title: "Recordings/Scheduled Preference",
  component: SessionSettingsDialog,
  args: { session, locale: "en" },
  decorators: [
    (Story, context) => (
      <div
        className="mx-auto max-w-xl"
        dir={context.args.locale === "fa" ? "rtl" : "ltr"}
        lang={context.args.locale}
      >
        <Story />
      </div>
    ),
  ],
} satisfies Meta<typeof SessionSettingsDialog>;

export default meta;
type Story = StoryObj<typeof meta>;

const openSettings: NonNullable<Story["play"]> = async ({ canvas, canvasElement, args }) => {
  const text = roomCopy(args.locale);
  await userEvent.click(
    canvas.getByRole("button", { name: `${text.sessionSettings}: ${args.session.title}` }),
  );
  const body = within(canvasElement.ownerDocument.body);
  const dialog = await body.findByRole("dialog", { name: text.sessionSettings });
  await expect(dialog).toHaveAttribute("dir", args.locale === "fa" ? "rtl" : "ltr");
  const checkbox = within(dialog).queryByRole("checkbox", { name: text.autoRecord });
  if (args.session.status !== "scheduled") {
    await expect(checkbox).not.toBeInTheDocument();
  } else if (args.session.auto_record) {
    await expect(checkbox).toBeChecked();
  } else {
    await expect(checkbox).not.toBeChecked();
  }
};

export const ScheduledEnabled: Story = { play: openSettings };
export const ScheduledDisabled: Story = {
  args: { session: { ...session, auto_record: false } },
  play: openSettings,
};
export const ScheduledPersian: Story = {
  args: { locale: "fa" },
  play: openSettings,
};
export const LivePreferenceLocked: Story = {
  args: { session: { ...session, status: "live" } },
  play: openSettings,
};

const createForm: Story["render"] = ({ locale }) => (
  <CreateSessionForm
    rooms={rooms}
    initialRoomId={session.room_id}
    initialCanManage
    locale={locale}
  />
);

export const CreateDefaultsOn: Story = {
  render: createForm,
  play: async ({ canvas, args }) => {
    await expect(
      canvas.getByRole("checkbox", { name: roomCopy(args.locale).autoRecord }),
    ).toBeChecked();
  },
};
export const CreateOptOut: Story = {
  render: createForm,
  play: async ({ canvas, args }) => {
    const checkbox = canvas.getByRole("checkbox", { name: roomCopy(args.locale).autoRecord });
    await userEvent.click(checkbox);
    await expect(checkbox).not.toBeChecked();
  },
};
export const CreatePersian: Story = {
  args: { locale: "fa" },
  render: createForm,
  play: CreateDefaultsOn.play,
};
export const CreateMember: Story = {
  render: ({ locale }) => (
    <CreateSessionForm
      rooms={rooms}
      initialRoomId={session.room_id}
      initialCanManage={false}
      locale={locale}
    />
  ),
  play: async ({ canvas, args }) => {
    await expect(
      canvas.getByRole("button", { name: roomCopy(args.locale).newSession }),
    ).toBeDisabled();
  },
};
