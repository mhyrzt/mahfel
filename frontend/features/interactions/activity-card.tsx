"use client";

import { useState } from "react";
import { Check, LockKeyhole, LockKeyholeOpen, VenetianMask, UsersRound } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader } from "@/components/ui/card";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { messagesFor, formatMessage } from "@/lib/i18n/messages";
import type { Locale } from "@/lib/i18n/config";
import { ActivityAnswer } from "./activity-answer";
import { ActivityResults } from "./activity-results";
import { ActivityVoters } from "./activity-voters";
import type { Activity, SendInteraction } from "./types";

export function ActivityCard({
  activity,
  sessionId,
  locale,
  canManage,
  canRespond,
  hasOpenActivity,
  busy,
  send,
  onEdit,
}: Readonly<{
  activity: Activity;
  sessionId: string;
  locale: Locale;
  canManage: boolean;
  canRespond: boolean;
  hasOpenActivity: boolean;
  busy: boolean;
  send: SendInteraction;
  onEdit: () => void;
}>) {
  const text = messagesFor(locale).interactions;
  const [changing, setChanging] = useState(false);
  const [closing, setClosing] = useState(false);
  const answering =
    activity.status === "open" &&
    canRespond &&
    (!activity.submitted || (activity.kind === "poll" && changing));
  return (
    <Card className="min-w-0 rounded-2xl border-border/70 bg-card/95 shadow-sm">
      <CardHeader className="space-y-2 px-5 pb-4 pt-5 sm:px-8 sm:pt-7">
        <div className="flex flex-wrap items-center justify-between gap-2">
          <Badge variant="secondary">{text[activity.kind]}</Badge>
          <Badge variant="outline" aria-label={text[activity.status]} title={text[activity.status]}>
            {activity.status === "draft" ? text.draft : null}
            {activity.status === "open" ? (
              <LockKeyholeOpen className="size-3.5" aria-hidden="true" />
            ) : null}
            {activity.status === "closed" ? (
              <LockKeyhole className="size-3.5" aria-hidden="true" />
            ) : null}
          </Badge>
        </div>
        <p className="flex items-center gap-1.5 text-xs text-muted-foreground">
          {activity.anonymous ? (
            <VenetianMask className="size-3.5 shrink-0" aria-hidden="true" />
          ) : (
            <UsersRound className="size-3.5 shrink-0" aria-hidden="true" />
          )}
          {activity.anonymous ? text.anonymous : text.named}
        </p>
        <p className="text-xs text-muted-foreground">
          {activity.anonymous ? text.anonymousHint : text.namedHint}
        </p>
      </CardHeader>
      <CardContent className="space-y-5 px-5 pb-5 sm:px-8 sm:pb-7">
        {answering ? (
          <ActivityAnswer
            key={`${activity.id}-${activity.vote_version}`}
            activity={activity}
            locale={locale}
            busy={busy}
            send={send}
            onAnswered={() => setChanging(false)}
          />
        ) : (
          <h3
            className="break-words text-lg font-semibold leading-relaxed sm:text-xl [overflow-wrap:anywhere]"
            dir="auto"
          >
            {activity.question}
          </h3>
        )}
        {!answering && activity.results ? (
          <ActivityResults activity={activity} locale={locale} />
        ) : null}
        {!answering && !activity.results ? (
          <ul className="space-y-2">
            {activity.options.map((option, index) => (
              <li
                className="flex items-start gap-2 rounded-lg border px-3 py-2 text-sm"
                key={`${activity.id}-${index}`}
              >
                <span className="min-w-0 flex-1 break-words [overflow-wrap:anywhere]" dir="auto">
                  {option}
                </span>
                {activity.selected.includes(index) ? (
                  <Check className="size-4 shrink-0 text-primary" aria-label={text.selected} />
                ) : null}
              </li>
            ))}
          </ul>
        ) : null}
        {activity.kind === "quiz" && activity.status === "open" ? (
          <p className="text-xs text-muted-foreground">
            {activity.submitted ? text.waitingResults : text.finalAnswer}
          </p>
        ) : null}
        {activity.submitted && activity.kind === "quiz" && activity.status === "open" ? (
          <p role="status" className="flex items-center gap-2 text-sm font-medium">
            <Check className="size-4" aria-hidden="true" />
            {text.submitted}
          </p>
        ) : null}
        {activity.participation_count !== null &&
        activity.participation_count !== undefined &&
        !activity.results ? (
          <p className="text-xs text-muted-foreground">
            {formatMessage(text.responses, {
              count: new Intl.NumberFormat(locale).format(activity.participation_count),
            })}
          </p>
        ) : null}
        <div className="flex flex-wrap gap-2">
          {activity.status === "open" &&
          activity.kind === "poll" &&
          activity.submitted &&
          canRespond ? (
            <>
              <Button
                size="sm"
                variant="outline"
                disabled={busy}
                onClick={() => setChanging((value) => !value)}
              >
                {changing ? text.cancel : text.changeVote}
              </Button>
              <Button
                size="sm"
                variant="ghost"
                disabled={busy}
                onClick={() =>
                  void send({
                    action: "retract",
                    id: activity.id,
                    expected_version: activity.vote_version,
                  }).then((success) => {
                    if (success) {
                      setChanging(false);
                    }
                  })
                }
              >
                {text.retract}
              </Button>
            </>
          ) : null}
          {canManage && activity.status === "draft" ? (
            <>
              <Button size="sm" variant="outline" disabled={busy} onClick={onEdit}>
                {text.edit}
              </Button>
              <Button
                size="sm"
                disabled={busy || hasOpenActivity}
                onClick={() => void send({ action: "launch", id: activity.id })}
              >
                <LockKeyholeOpen className="size-4" aria-hidden="true" />
                {text.launch}
              </Button>
            </>
          ) : null}
          {canManage && activity.status === "open" ? (
            <Button
              size="sm"
              variant="outline"
              disabled={busy}
              onClick={() => {
                if (activity.kind === "quiz") {
                  setClosing(true);
                } else {
                  void send({ action: "close", id: activity.id });
                }
              }}
            >
              <LockKeyhole className="size-4" aria-hidden="true" />
              {text.close}
            </Button>
          ) : null}
          {!activity.anonymous && activity.results && !answering ? (
            <ActivityVoters activity={activity} sessionId={sessionId} locale={locale} />
          ) : null}
        </div>
        {canManage && activity.status === "draft" && hasOpenActivity ? (
          <p className="text-xs text-muted-foreground">{text.activeFirst}</p>
        ) : null}
      </CardContent>
      <Dialog open={closing} onOpenChange={setClosing}>
        <DialogContent dir={locale === "fa" ? "rtl" : "ltr"}>
          <DialogHeader>
            <DialogTitle>{text.close}</DialogTitle>
            <DialogDescription>{text.closeQuizHint}</DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="outline" disabled={busy} onClick={() => setClosing(false)}>
              {text.cancel}
            </Button>
            <Button
              disabled={busy}
              onClick={() =>
                void send({ action: "close", id: activity.id }).then((success) => {
                  if (success) {
                    setClosing(false);
                  }
                })
              }
            >
              {text.close}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </Card>
  );
}
