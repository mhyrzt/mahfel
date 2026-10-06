"use client";

import { useId, useState } from "react";
import { Plus, X, VenetianMask, ListChecks } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import { Field, FieldDescription, FieldLabel } from "@/components/ui/field";
import { Input } from "@/components/ui/input";
import { RadioGroup, RadioGroupItem } from "@/components/ui/radio-group";
import { Switch } from "@/components/ui/switch";
import { Textarea } from "@/components/ui/textarea";
import { messagesFor, formatMessage } from "@/lib/i18n/messages";
import type { Locale } from "@/lib/i18n/config";
import type { Activity, SendInteraction } from "./types";

export function ActivityComposer({
  activity,
  locale,
  onClose,
  busy,
  send,
}: Readonly<{
  activity?: Activity;
  locale: Locale;
  onClose: () => void;
  busy: boolean;
  send: SendInteraction;
}>) {
  const text = messagesFor(locale).interactions;
  return (
    <section
      dir={locale === "fa" ? "rtl" : "ltr"}
      className="flex min-h-0 flex-1 flex-col overflow-hidden rounded-2xl border bg-card shadow-sm"
      aria-label={activity ? text.edit : text.create}
    >
      <header className="flex shrink-0 items-center justify-between gap-3 border-b px-4 py-3 sm:px-5">
        <h3 className="text-sm font-semibold">{activity ? text.edit : text.create}</h3>
        <Button
          size="icon"
          variant="ghost"
          disabled={busy}
          onClick={onClose}
          aria-label={text.cancel}
        >
          <X className="size-4" aria-hidden="true" />
        </Button>
      </header>
      <ComposerForm
        key={activity?.id ?? "new"}
        activity={activity}
        locale={locale}
        busy={busy}
        send={send}
        onSaved={onClose}
      />
    </section>
  );
}

function ComposerForm({
  activity,
  locale,
  busy,
  send,
  onSaved,
}: Readonly<{
  activity?: Activity;
  locale: Locale;
  busy: boolean;
  send: SendInteraction;
  onSaved: () => void;
}>) {
  const text = messagesFor(locale).interactions;
  const formId = useId();
  const [activityId] = useState(() => activity?.id ?? crypto.randomUUID());
  const [question, setQuestion] = useState(activity?.question ?? "");
  const [kind, setKind] = useState<"poll" | "quiz">(activity?.kind ?? "poll");
  const [multiple, setMultiple] = useState(activity?.multiple ?? false);
  const [anonymous, setAnonymous] = useState(activity?.anonymous ?? true);
  const [options, setOptions] = useState(() =>
    (activity?.options ?? ["", ""]).map((label, index) => ({
      id: index,
      label,
      correct: activity?.draft_correct_options?.includes(index) ?? false,
    })),
  );
  const [nextId, setNextId] = useState(options.length);
  const [explanation, setExplanation] = useState(activity?.draft_explanation ?? "");
  const [invalid, setInvalid] = useState(false);
  const [saveFailed, setSaveFailed] = useState(false);
  const numbers = new Intl.NumberFormat(locale);
  return (
    <form
      className="flex min-h-0 flex-1 flex-col"
      onSubmit={(event) => {
        event.preventDefault();
        const trimmed = options.map((o) => o.label.trim());
        const correctOptions = options.flatMap((o, i) => (o.correct ? [i] : []));
        if (
          !question.trim() ||
          trimmed.some((o) => !o) ||
          new Set(trimmed).size !== trimmed.length ||
          (kind === "quiz" &&
            (correctOptions.length === 0 || (!multiple && correctOptions.length !== 1)))
        ) {
          setInvalid(true);
          return;
        }
        setInvalid(false);
        setSaveFailed(false);
        void send({
          action: "save_draft",
          id: activityId,
          definition: {
            question: question.trim(),
            options: trimmed,
            kind,
            multiple,
            anonymous,
            correct_options: kind === "quiz" ? correctOptions : [],
            explanation: kind === "quiz" ? explanation.trim() || null : null,
          },
        }).then((success) => {
          if (success) {
            onSaved();
          } else {
            setSaveFailed(true);
          }
        });
      }}
    >
      <div className="min-h-0 flex-1 space-y-4 overflow-y-auto overscroll-contain px-4 py-4 sm:px-5">
        <Field>
          <FieldLabel className="sr-only">{text.title}</FieldLabel>
          <RadioGroup
            value={kind}
            onValueChange={(value) => {
              if (value === "poll" || value === "quiz") {
                setKind(value);
              }
            }}
            className="grid grid-cols-2 gap-2 rounded-lg bg-muted/50 p-1"
            dir={locale === "fa" ? "rtl" : "ltr"}
          >
            {(["poll", "quiz"] as const).map((value) => (
              <Field
                key={value}
                orientation="horizontal"
                className="rounded-md px-3 py-2 has-[[data-state=checked]]:bg-background has-[[data-state=checked]]:shadow-sm"
              >
                <RadioGroupItem id={`${formId}-${value}`} value={value} disabled={busy} />
                <FieldLabel htmlFor={`${formId}-${value}`}>{text[value]}</FieldLabel>
              </Field>
            ))}
          </RadioGroup>
        </Field>
        <Field>
          <FieldLabel htmlFor={`${formId}-question`}>{text.question}</FieldLabel>
          <Textarea
            className="min-h-20 resize-none"
            rows={2}
            id={`${formId}-question`}
            dir="auto"
            maxLength={500}
            value={question}
            onChange={(event) => setQuestion(event.target.value)}
            disabled={busy}
            required
          />
        </Field>
        <div className="space-y-2">
          {kind === "quiz" ? (
            <p className="text-xs text-muted-foreground">{text.exactSet}</p>
          ) : null}
          {options.map((option, index) => (
            <Field key={option.id} className="gap-1.5">
              <FieldLabel className="sr-only" htmlFor={`${formId}-option-${option.id}`}>
                {formatMessage(text.option, { number: numbers.format(index + 1) })}
              </FieldLabel>
              <div className="flex items-center gap-2">
                <span
                  className="w-5 shrink-0 text-center text-xs tabular-nums text-muted-foreground"
                  aria-hidden="true"
                >
                  {numbers.format(index + 1)}
                </span>
                <Input
                  className="min-w-0 flex-1"
                  id={`${formId}-option-${option.id}`}
                  dir="auto"
                  maxLength={200}
                  value={option.label}
                  onChange={(event) =>
                    setOptions((current) =>
                      current.map((o) =>
                        o.id === option.id ? { ...o, label: event.target.value } : o,
                      ),
                    )
                  }
                  disabled={busy}
                  required
                />
                {kind === "quiz" ? (
                  <Field
                    orientation="horizontal"
                    className="w-auto shrink-0 gap-1"
                    title={formatMessage(text.correctOption, { number: numbers.format(index + 1) })}
                  >
                    <Checkbox
                      id={`${formId}-correct-${option.id}`}
                      checked={option.correct}
                      disabled={busy}
                      onCheckedChange={(checked) =>
                        setOptions((current) =>
                          current.map((o) => ({
                            ...o,
                            correct: o.id === option.id ? checked === true : multiple && o.correct,
                          })),
                        )
                      }
                    />
                    <FieldLabel className="sr-only" htmlFor={`${formId}-correct-${option.id}`}>
                      {formatMessage(text.correctOption, { number: numbers.format(index + 1) })}
                    </FieldLabel>
                  </Field>
                ) : null}
                <Button
                  type="button"
                  variant="ghost"
                  size="icon"
                  disabled={busy || options.length <= 2}
                  aria-label={formatMessage(text.removeOption, {
                    number: numbers.format(index + 1),
                  })}
                  onClick={() => setOptions((current) => current.filter((o) => o.id !== option.id))}
                >
                  <X className="size-4" aria-hidden="true" />
                </Button>
              </div>
            </Field>
          ))}
          <Button
            type="button"
            variant="outline"
            size="sm"
            disabled={busy || options.length >= 10}
            onClick={() => {
              setOptions((current) => [...current, { id: nextId, label: "", correct: false }]);
              setNextId((id) => id + 1);
            }}
          >
            <Plus className="size-4" aria-hidden="true" />
            {text.addOption}
          </Button>
        </div>
        <div className="divide-y rounded-lg border bg-muted/10">
          <Field orientation="horizontal" className="justify-between gap-4 px-3 py-3">
            <FieldLabel htmlFor={`${formId}-multiple`} className="min-w-0 gap-2 text-sm">
              <ListChecks className="size-4 shrink-0 text-muted-foreground" aria-hidden="true" />
              {text.multiple}
            </FieldLabel>
            <Switch
              id={`${formId}-multiple`}
              checked={multiple}
              disabled={busy}
              onCheckedChange={(value) => {
                setMultiple(value);
                if (!value) {
                  const first = options.find((o) => o.correct)?.id;
                  setOptions((current) => current.map((o) => ({ ...o, correct: o.id === first })));
                }
              }}
            />
          </Field>
          <div className="space-y-1.5 px-3 py-3">
            <Field orientation="horizontal" className="justify-between gap-4">
              <FieldLabel htmlFor={`${formId}-anonymous`} className="min-w-0 gap-2 text-sm">
                <VenetianMask
                  className="size-4 shrink-0 text-muted-foreground"
                  aria-hidden="true"
                />
                {text.anonymous}
              </FieldLabel>
              <Switch
                id={`${formId}-anonymous`}
                checked={anonymous}
                disabled={busy}
                aria-describedby={`${formId}-anonymous-hint`}
                onCheckedChange={setAnonymous}
              />
            </Field>
            <FieldDescription
              id={`${formId}-anonymous-hint`}
              className="max-w-prose text-xs leading-relaxed"
            >
              {anonymous ? text.anonymousHint : text.namedHint}
            </FieldDescription>
          </div>
        </div>
        {kind === "quiz" ? (
          <Field>
            <FieldLabel htmlFor={`${formId}-explanation`}>{text.explanation}</FieldLabel>
            <Textarea
              className="min-h-16 resize-none"
              rows={2}
              id={`${formId}-explanation`}
              dir="auto"
              maxLength={1000}
              value={explanation}
              onChange={(event) => setExplanation(event.target.value)}
              disabled={busy}
            />
            <FieldDescription>{text.exactSet}</FieldDescription>
          </Field>
        ) : null}
        {invalid ? (
          <p className="text-sm text-destructive" role="alert">
            {text.invalid}
          </p>
        ) : null}
        {saveFailed ? (
          <p className="text-sm text-destructive" role="alert">
            {text.failed}
          </p>
        ) : null}
      </div>
      <footer className="flex shrink-0 justify-end border-t bg-muted/20 px-4 py-3 sm:px-5">
        <Button className="min-w-32" type="submit" disabled={busy}>
          {text.saveDraft}
        </Button>
      </footer>
    </form>
  );
}
