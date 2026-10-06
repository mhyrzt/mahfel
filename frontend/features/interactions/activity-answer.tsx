"use client";

import {
  Questionnaire,
  QuestionnaireActions,
  QuestionnaireChoice,
  QuestionnaireChoices,
  QuestionnaireDescription,
  QuestionnaireError,
  QuestionnaireItem,
  QuestionnaireSubmit,
  QuestionnaireTitle,
} from "@/components/ui/questionnaire";
import { messagesFor } from "@/lib/i18n/messages";
import type { Locale } from "@/lib/i18n/config";
import type { Activity, SendInteraction } from "./types";

export function ActivityAnswer({
  activity,
  locale,
  busy,
  send,
  onAnswered,
}: Readonly<{
  activity: Activity;
  locale: Locale;
  busy: boolean;
  send: SendInteraction;
  onAnswered: () => void;
}>) {
  const text = messagesFor(locale).interactions;
  const items = [
    {
      name: "answer",
      required: true,
      choices: activity.options.map((_, index) => ({ value: String(index) })),
    },
  ];
  return (
    <Questionnaire
      items={items}
      dir={locale === "fa" ? "rtl" : "ltr"}
      onSubmit={(event) => {
        event.preventDefault();
        const selected = new FormData(event.currentTarget)
          .getAll("answer")
          .map((value) => Number(value));
        if (selected.length > 0) {
          void send({
            action: "vote",
            id: activity.id,
            selected,
            expected_version: activity.vote_version,
          }).then((success) => {
            if (success) {
              onAnswered();
            }
          });
        }
      }}
    >
      <QuestionnaireItem name="answer" multiple={activity.multiple} required disabled={busy}>
        <QuestionnaireTitle
          className="break-words text-lg leading-relaxed sm:text-xl [overflow-wrap:anywhere]"
          dir="auto"
        >
          {activity.question}
        </QuestionnaireTitle>
        <QuestionnaireDescription>
          {activity.multiple ? text.selectMany : text.selectOne}
        </QuestionnaireDescription>
        <QuestionnaireChoices>
          {activity.options.map((option, index) => (
            <QuestionnaireChoice
              key={`${activity.id}-${index}`}
              value={String(index)}
              defaultChecked={activity.selected.includes(index)}
            >
              <span dir="auto" className="break-words [overflow-wrap:anywhere]">
                {option}
              </span>
            </QuestionnaireChoice>
          ))}
        </QuestionnaireChoices>
        <QuestionnaireError>{text.chooseAnswer}</QuestionnaireError>
      </QuestionnaireItem>
      <QuestionnaireActions className="flex justify-end">
        <QuestionnaireSubmit disabled={busy}>
          {activity.kind === "quiz" ? text.submit : text.vote}
        </QuestionnaireSubmit>
      </QuestionnaireActions>
    </Questionnaire>
  );
}
