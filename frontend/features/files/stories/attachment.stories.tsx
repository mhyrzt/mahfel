import type { Meta, StoryObj } from "@storybook/nextjs";
import { FileText, UploadCloud } from "lucide-react";

import {
  Attachment,
  AttachmentContent,
  AttachmentDescription,
  AttachmentMedia,
  AttachmentTitle,
} from "@/components/ui/attachment";

function AttachmentExample({
  state,
}: Readonly<{ state: "idle" | "uploading" | "error" | "done" }>) {
  const detail = {
    idle: "Ready to upload",
    uploading: "Uploading…",
    error: "Upload failed. Try again.",
    done: "PDF · 2.4 MB",
  }[state];
  return (
    <Attachment className="max-w-sm" state={state}>
      <AttachmentMedia>
        {state === "idle" ? <UploadCloud aria-hidden="true" /> : <FileText aria-hidden="true" />}
      </AttachmentMedia>
      <AttachmentContent>
        <AttachmentTitle>research-notes.pdf</AttachmentTitle>
        <AttachmentDescription>{detail}</AttachmentDescription>
      </AttachmentContent>
    </Attachment>
  );
}

const meta = {
  title: "Files/Attachment",
  component: AttachmentExample,
  args: { state: "done" },
  decorators: [
    (Story) => (
      <div className="p-6">
        <Story />
      </div>
    ),
  ],
} satisfies Meta<typeof AttachmentExample>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Done: Story = {};
export const Ready: Story = { args: { state: "idle" } };
export const Uploading: Story = { args: { state: "uploading" } };
export const Error: Story = { args: { state: "error" } };
