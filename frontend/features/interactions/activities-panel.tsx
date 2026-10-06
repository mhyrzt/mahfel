"use client";

import { useState } from "react";
import { ChevronLeft, ChevronRight, Plus } from "lucide-react";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import { formatMessage, messagesFor } from "@/lib/i18n/messages";
import type { Locale } from "@/lib/i18n/config";
import { ActivityCard } from "./activity-card";
import { ActivityComposer } from "./activity-composer";
import type { Activity, InteractionsSnapshot, SendInteraction } from "./types";
import { useSessionInteractions } from "./use-session-interactions";

export function ActivitiesPanel({
  sessionId,
  locale,
  snapshot,
  error,
  busy,
  send,
  onRetry,
  selectedActivityId,
  onSelectActivity,
}: Readonly<{
  sessionId: string;
  locale: Locale;
  snapshot: InteractionsSnapshot | null;
  error: string | null;
  busy: boolean;
  send: SendInteraction;
  onRetry: () => void;
  selectedActivityId?: string | null;
  onSelectActivity?: (id: string) => void;
}>) {
  const text = messagesFor(locale).interactions;
  const [composer, setComposer] = useState<{ activity?: Activity } | null>(null);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const activities = (snapshot?.activities ?? [])
    .slice()
    .sort((a, b) => Number(b.status === "open") - Number(a.status === "open"));
  const selectedIndex = Math.max(
    0,
    activities.findIndex(
      (activity) =>
        activity.id === (selectedActivityId !== undefined ? selectedActivityId : selectedId),
    ),
  );
  const selectActivity = (id: string) => {
    setSelectedId(id);
    onSelectActivity?.(id);
  };
  const numbers = new Intl.NumberFormat(locale);
  const slideLabel = formatMessage(text.questionPosition, {
    current: numbers.format(selectedIndex + 1),
    total: numbers.format(activities.length),
  });
  const hasOpen = snapshot?.activities.some((a) => a.status === "open") ?? false;
  return (
    <section
      className={
        composer
          ? "flex h-full min-h-0 min-w-0 flex-1 flex-col gap-3 py-2 sm:py-4"
          : "flex min-h-full min-w-0 flex-1 flex-col gap-4 py-2 sm:py-4"
      }
      dir={locale === "fa" ? "rtl" : "ltr"}
      aria-label={text.title}
      aria-roledescription={text.slideshow}
    >
      <div className="flex flex-wrap items-center justify-between gap-2">
        <h2 className="text-base font-semibold">{text.title}</h2>
        {snapshot?.can_manage && snapshot.live && !composer ? (
          <Button size="sm" variant="outline" onClick={() => setComposer({})}>
            <Plus className="size-4" aria-hidden="true" />
            {text.create}
          </Button>
        ) : null}
      </div>
      {error ? (
        <Alert variant="destructive">
          <AlertDescription>
            {error}
            <Button className="mt-2" size="sm" variant="outline" disabled={busy} onClick={onRetry}>
              {text.retry}
            </Button>
          </AlertDescription>
        </Alert>
      ) : null}
      {!snapshot && !error ? (
        <p className="text-sm text-muted-foreground" role="status">
          {text.loading}
        </p>
      ) : null}
      {!composer && snapshot?.activities.length === 0 ? (
        <p className="py-6 text-center text-sm text-muted-foreground">{text.empty}</p>
      ) : null}
      <div
        hidden={Boolean(composer)}
        className="flex flex-1 flex-col justify-center py-2 sm:py-6 [&[hidden]]:hidden"
      >
        {snapshot &&
          activities.map((activity, index) => (
            <div
              key={activity.id}
              data-activity-slide
              hidden={index !== selectedIndex}
              role="group"
              aria-roledescription={text.slide}
              aria-label={formatMessage(text.questionPosition, {
                current: numbers.format(index + 1),
                total: numbers.format(activities.length),
              })}
            >
              <ActivityCard
                activity={activity}
                sessionId={sessionId}
                locale={locale}
                canManage={snapshot.can_manage && snapshot.live}
                canRespond={snapshot.can_respond}
                hasOpenActivity={hasOpen}
                busy={busy}
                send={send}
                onEdit={() => setComposer({ activity })}
              />
            </div>
          ))}
      </div>
      {!composer && activities.length > 0 ? (
        <nav
          className="flex shrink-0 items-center justify-between gap-3 border-t pt-4"
          aria-label={text.questionNavigation}
        >
          <Button
            variant="outline"
            className="h-10 px-4"
            size="sm"
            disabled={selectedIndex === 0}
            onClick={() => selectActivity(activities[selectedIndex - 1].id)}
          >
            <ChevronLeft className="size-4 rtl:rotate-180" aria-hidden="true" />
            {text.previousQuestion}
          </Button>
          <span className="text-center text-sm text-muted-foreground tabular-nums" role="status">
            {slideLabel}
          </span>
          <Button
            variant="outline"
            className="h-10 px-4"
            size="sm"
            disabled={selectedIndex >= activities.length - 1}
            onClick={() => selectActivity(activities[selectedIndex + 1].id)}
          >
            {text.nextQuestion}
            <ChevronRight className="size-4 rtl:rotate-180" aria-hidden="true" />
          </Button>
        </nav>
      ) : null}
      {composer ? (
        <ActivityComposer
          activity={composer.activity}
          locale={locale}
          onClose={() => setComposer(null)}
          busy={busy}
          send={send}
        />
      ) : null}
    </section>
  );
}

export function SessionActivitiesHistory({
  sessionId,
  locale,
}: Readonly<{ sessionId: string; locale: Locale }>) {
  const interactions = useSessionInteractions(sessionId, locale, true);
  return (
    <ActivitiesPanel
      sessionId={sessionId}
      locale={locale}
      snapshot={interactions.snapshot}
      error={interactions.error}
      busy={interactions.busy}
      send={interactions.send}
      onRetry={() => void interactions.refresh()}
    />
  );
}
