import type { Meta, StoryObj } from "@storybook/nextjs";
import { RecordingListView } from "../recording-list-view";

const meta = {
  title: "Features/Recordings/Exports",
  component: RecordingListView,
  args: { locale: "en", items: [] },
  decorators: [
    (Story) => (
      <div className="w-[460px] max-w-full p-4">
        <Story />
      </div>
    ),
  ],
} satisfies Meta<typeof RecordingListView>;
export default meta;
type Story = StoryObj<typeof meta>;
const record = {
  id: "recording",
  started_at: "2026-09-29T08:00:00Z",
  status: "ready",
  partial: false,
  export: null,
};
const exported = {
  id: "export",
  recording_id: record.id,
  status: "ready",
  progress: 100,
  duration_ms: 3600000,
  size_bytes: 12000000,
  failure_code: null,
  partial: false,
};
export const Empty: Story = {};
export const Loading: Story = { args: { loading: true } };
export const Preparing: Story = { args: { items: [{ ...record, status: "finalizing" }] } };
export const Queued: Story = {
  args: { items: [{ ...record, export: { ...exported, status: "queued", progress: 0 } }] },
};
export const Rendering: Story = {
  args: { items: [{ ...record, export: { ...exported, status: "rendering", progress: 42 } }] },
};
export const Ready: Story = { args: { items: [{ ...record, export: exported }] } };
export const Partial: Story = {
  args: { items: [{ ...record, partial: true, export: { ...exported, partial: true } }] },
};
export const Failed: Story = {
  args: {
    canManage: true,
    items: [
      {
        ...record,
        export: { ...exported, status: "failed", failure_code: "export_render_failed" },
      },
    ],
  },
};
export const Unavailable: Story = { args: { failed: true } };
export const Manager: Story = {
  args: { canManage: true, items: [{ ...record, export: exported }] },
};
export const Persian: Story = { args: { locale: "fa", items: [{ ...record, export: exported }] } };
