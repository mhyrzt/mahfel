import { Check, X } from "lucide-react";
import { messagesFor, formatMessage } from "@/lib/i18n/messages";
import type { Locale } from "@/lib/i18n/config";
import { cn } from "@/lib/utils";
import type { Activity } from "./types";

export function ActivityResults({
  activity,
  locale,
}: Readonly<{ activity: Activity; locale: Locale }>) {
  const text = messagesFor(locale).interactions;
  const results = activity.results;
  if (!results) {
    return null;
  }
  const numbers = new Intl.NumberFormat(locale);
  const percentage = new Intl.NumberFormat(locale, { style: "percent", maximumFractionDigits: 0 });
  return (
    <div className="space-y-3">
      {results.correct !== null && results.correct !== undefined ? (
        <p
          className={cn(
            "flex items-center gap-2 text-sm font-medium",
            results.correct ? "text-success" : "text-destructive",
          )}
        >
          {results.correct ? (
            <Check className="size-4" aria-hidden="true" />
          ) : (
            <X className="size-4" aria-hidden="true" />
          )}
          {results.correct ? text.correct : text.incorrect}
        </p>
      ) : null}
      <ul className="space-y-3" aria-label={text.title}>
        {activity.options.map((option, index) => {
          const count = results.counts[index] ?? 0;
          const ratio = results.respondents > 0 ? count / results.respondents : 0;
          const correct = results.correct_options?.includes(index);
          const selected = activity.selected.includes(index);
          return (
            <li className="space-y-1.5" key={`${index}-${option}`}>
              <div className="flex items-start justify-between gap-3 text-sm">
                <div className="min-w-0 flex-1 break-words [overflow-wrap:anywhere]">
                  <span dir="auto">{option}</span>
                  {correct ? (
                    <span className="ms-2 inline-flex items-center gap-1 text-xs font-medium text-success">
                      <Check className="size-3" aria-hidden="true" />
                      {text.correct}
                    </span>
                  ) : null}
                  {selected ? (
                    <span className="ms-2 text-xs text-muted-foreground">{text.selected}</span>
                  ) : null}
                </div>
                <span className="shrink-0 tabular-nums">
                  <bdi>{percentage.format(ratio)}</bdi>
                  <span className="ms-2 text-xs text-muted-foreground">
                    (<bdi>{numbers.format(count)}</bdi>)
                  </span>
                </span>
              </div>
              <div className="h-2 overflow-hidden rounded-full bg-muted" aria-hidden="true">
                <div
                  data-result-bar
                  className={cn(
                    "h-full w-full origin-left rounded-full bg-primary rtl:origin-right",
                    correct && "bg-success",
                  )}
                  style={{ transform: `scaleX(${ratio})` }}
                />
              </div>
            </li>
          );
        })}
      </ul>
      <p className="text-xs text-muted-foreground">
        {formatMessage(text.respondents, { count: numbers.format(results.respondents) })}
      </p>
      {activity.multiple ? (
        <p className="text-xs text-muted-foreground">{text.multiPercentage}</p>
      ) : null}
      {results.explanation ? (
        <p
          className="break-words text-sm text-muted-foreground [overflow-wrap:anywhere]"
          dir="auto"
        >
          {results.explanation}
        </p>
      ) : null}
    </div>
  );
}
