import type { components } from "@/lib/api/generated/schema";

export type RecordingExport = components["schemas"]["RecordingExportResponse"];
export type RecordingPlayback = components["schemas"]["RecordingPlaybackResponse"];
export interface RecordingListItem {
  id: string;
  started_at: string;
  status: string;
  partial: boolean;
  warning?: string | null;
  export: RecordingExport | null;
}
