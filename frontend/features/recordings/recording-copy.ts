import type { Locale } from "@/lib/i18n/config";

const copy = {
  en: {
    label: "Session recording",
    loading: "Checking recording status…",
    scope: "Media, shared workspace and session chat are captured while recording is active.",
    unavailable: "Recording status could not be verified. Try again shortly.",
    failed: "Recording capture has a problem. The live session can continue.",
    commandFailed: "Recording could not be updated. Check the status and try again.",
    start: "Start recording",
    pause: "Pause recording",
    resume: "Resume recording",
    stop: "Stop recording",
    starting: "Starting recording",
    recording: "Recording",
    paused: "Recording paused",
    stopping: "Stopping recording",
    finalizing: "Preparing recording",
    ready: "Recording ready",
    partial: "Recording has gaps",
    off: "Recording is off",
    failedStatus: "Recording failed",
    updating: "Updating recording…",
    readOnly: "You can view recording status here.",
  },
  fa: {
    label: "ضبط دیدار",
    loading: "در حال بررسی وضعیت ضبط…",
    scope: "هنگام ضبط، رسانه، فضای کار اشتراکی و گفتگوی دیدار ذخیره می‌شود.",
    unavailable: "بررسی وضعیت ضبط ممکن نشد. کمی بعد دوباره تلاش کنید.",
    failed: "ضبط با مشکل روبه‌رو شده است. دیدار می‌تواند ادامه یابد.",
    commandFailed: "تغییر ضبط ممکن نشد. وضعیت را بررسی و دوباره تلاش کنید.",
    start: "شروع ضبط",
    pause: "مکث ضبط",
    resume: "ادامه ضبط",
    stop: "توقف ضبط",
    starting: "در حال شروع ضبط",
    recording: "در حال ضبط",
    paused: "ضبط متوقف است",
    stopping: "در حال توقف ضبط",
    finalizing: "در حال آماده‌سازی ضبط",
    ready: "ضبط آماده است",
    partial: "بخش‌هایی از ضبط موجود نیست",
    off: "ضبط خاموش است",
    failedStatus: "ضبط ناموفق بود",
    updating: "در حال تغییر وضعیت ضبط…",
    readOnly: "اینجا می‌توانید وضعیت ضبط را ببینید.",
  },
} as const;

export function recordingCopy(locale: Locale) {
  return copy[locale];
}
