"use client";

import { useEffect, useState, useSyncExternalStore } from "react";

import type { Locale } from "@/lib/i18n/config";

const subscribe = () => () => {};
const browserTimeZone = () => Intl.DateTimeFormat().resolvedOptions().timeZone || "UTC";
const serverTimeZone = () => "UTC";

export function WorkspaceGreeting({
  accountName,
  locale,
  referenceDate,
}: Readonly<{ accountName?: string | null; locale: Locale; referenceDate: string }>) {
  const [now, setNow] = useState(referenceDate);
  const timeZone = useSyncExternalStore(subscribe, browserTimeZone, serverTimeZone);

  useEffect(() => {
    const timer = window.setInterval(() => setNow(new Date().toISOString()), 1_000);
    return () => window.clearInterval(timer);
  }, []);

  const time = new Intl.DateTimeFormat(locale === "fa" ? "fa-IR" : "en-US", {
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
    hourCycle: "h23",
    timeZone,
  }).format(new Date(now));

  return (
    <div className="space-y-2">
      <h2 className="text-2xl font-semibold tracking-tight sm:text-3xl">
        {locale === "fa" ? "سلام" : "Hello"}{" "}
        {accountName?.trim() || (locale === "fa" ? "دوست من" : "there")} 👋
      </h2>
      <time className="text-sm tabular-nums text-muted-foreground" dateTime={now}>
        {time}
      </time>
    </div>
  );
}
