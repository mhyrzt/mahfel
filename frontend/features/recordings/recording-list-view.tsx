import { Download, LoaderCircle, Play, RotateCcw, Trash2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import type { Locale } from "@/lib/i18n/config";
import { messagesFor } from "@/lib/i18n/messages";
import type { RecordingListItem } from "./export-types";

export function RecordingListView({
  items,
  locale,
  loading = false,
  failed = false,
  actionFailed = false,
  canManage = false,
  busyId = null,
  onDownload,
  onRetry,
  onPlay,
  onRemove,
}: Readonly<{
  items: RecordingListItem[];
  locale: Locale;
  loading?: boolean;
  failed?: boolean;
  actionFailed?: boolean;
  canManage?: boolean;
  busyId?: string | null;
  onDownload?: (id: string) => void;
  onRetry?: (id: string) => void;
  onPlay?: (id: string) => void;
  onRemove?: (id: string) => void;
}>) {
  const text = messagesFor(locale).recordingExports;
  return (
    <div className="space-y-4" dir={locale === "fa" ? "rtl" : "ltr"}>
      {loading ? (
        <p role="status" className="text-sm text-muted-foreground">
          {text.loading}
        </p>
      ) : null}
      {failed || actionFailed ? (
        <p role="alert" className="text-sm text-destructive">
          {actionFailed ? text.retryFailed : text.unavailable}
        </p>
      ) : null}
      {!loading && !failed && items.length === 0 ? (
        <p className="py-8 text-center text-sm text-muted-foreground">{text.empty}</p>
      ) : null}
      <ul className="space-y-3">
        {items.map((item) => {
          const exportRecord = item.export;
          const status = exportRecord?.status ?? "preparing";
          const labels: Record<string, string> = {
            queued: text.queued,
            rendering: text.rendering,
            ready: text.ready,
            failed: text.failed,
          };
          const label = labels[status] ?? (item.status === "failed" ? text.failed : text.preparing);
          return (
            <li key={item.id} className="space-y-3 rounded-xl border bg-card p-4">
              <div className="flex flex-wrap items-start justify-between gap-3">
                <div className="space-y-1">
                  <time dateTime={item.started_at} className="text-sm font-medium">
                    {new Intl.DateTimeFormat(locale === "fa" ? "fa-IR" : "en-US", {
                      dateStyle: "medium",
                      timeStyle: "short",
                    }).format(new Date(item.started_at))}
                  </time>
                  <p role="status" className="text-sm text-muted-foreground">
                    {label}
                    {status === "rendering" ? ` · ${exportRecord?.progress ?? 0}%` : ""}
                  </p>
                </div>
                {status === "ready" ? (
                  <div className="flex items-center gap-2">
                    <Button
                      size="sm"
                      variant="outline"
                      disabled={busyId === item.id}
                      onClick={() => onPlay?.(item.id)}
                    >
                      <Play aria-hidden="true" className="size-4" />
                      {text.play}
                    </Button>
                    <Button
                      size="sm"
                      variant="outline"
                      disabled={busyId === item.id}
                      onClick={() => onDownload?.(item.id)}
                    >
                      <Download aria-hidden="true" className="size-4" />
                      {text.download}
                    </Button>
                  </div>
                ) : null}
                {status === "failed" && canManage ? (
                  <Button
                    size="sm"
                    variant="outline"
                    disabled={busyId === item.id}
                    onClick={() => onRetry?.(item.id)}
                  >
                    <RotateCcw aria-hidden="true" className="size-4" />
                    {text.retry}
                  </Button>
                ) : null}
                {canManage ? (
                  <Button
                    aria-label={text.remove}
                    size="sm"
                    variant="ghost"
                    disabled={busyId === item.id}
                    onClick={() => onRemove?.(item.id)}
                  >
                    <Trash2 aria-hidden="true" className="size-4" />
                  </Button>
                ) : null}
              </div>
              {status === "rendering" || status === "queued" ? (
                <div className="flex items-center gap-2">
                  <LoaderCircle
                    aria-hidden="true"
                    className="size-4 animate-spin text-muted-foreground motion-reduce:animate-none"
                  />
                  <progress
                    aria-label={text.rendering}
                    className="h-1.5 w-full accent-primary"
                    max={100}
                    value={exportRecord?.progress ?? 0}
                  />
                </div>
              ) : null}
              {item.partial || exportRecord?.partial ? (
                <p className="text-xs text-warning">{text.partial}</p>
              ) : null}
            </li>
          );
        })}
      </ul>
    </div>
  );
}
