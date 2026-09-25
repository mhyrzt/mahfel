import type { Meta, StoryObj } from "@storybook/nextjs";

import { FileList } from "../components/file-list";

const files = [
  {
    id: "file-1",
    file_name: "Reading list.pdf",
    owner_id: "teacher-1",
    context_id: "research-methods",
    context_kind: "room" as const,
    checksum_sha256: "demo",
    created_at: "2026-09-24T09:00:00Z",
    updated_at: "2026-09-24T09:00:00Z",
    mime: "application/pdf",
    size_bytes: 2048,
    status: "ready",
  },
  {
    id: "file-2",
    file_name: "Workshop notes.txt",
    owner_id: "student-1",
    context_id: "research-methods",
    context_kind: "room" as const,
    checksum_sha256: "demo",
    created_at: "2026-09-24T09:00:00Z",
    updated_at: "2026-09-24T09:00:00Z",
    mime: "text/plain",
    size_bytes: 512,
    status: "ready",
  },
];

const meta = {
  title: "Files/File List",
  component: FileList,
  args: {
    files,
    locale: "en",
    canDelete: false,
    actorId: "student-1",
    onPreview: () => undefined,
    onDownload: () => undefined,
    onRemove: () => undefined,
  },
} satisfies Meta<typeof FileList>;

export default meta;
type Story = StoryObj<typeof meta>;

export const OwnerView: Story = {};
export const ManagerView: Story = { args: { canDelete: true } };
export const GridView: Story = { args: { initialView: "grid" } };
export const Empty: Story = { args: { files: [] } };
export const Persian: Story = {
  args: { locale: "fa" },
  decorators: [
    (Story) => (
      <div dir="rtl">
        <Story />
      </div>
    ),
  ],
};
