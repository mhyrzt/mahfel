"use client";

import { useState, type FormEvent } from "react";
import { useRouter } from "next/navigation";

import { Field, FieldError, FieldLabel, FieldLegend, FieldSet } from "@/components/ui/field";
import { RadioGroup, RadioGroupItem } from "@/components/ui/radio-group";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { apiFailureFrom } from "@/lib/api/errors";
import { browserApiClient } from "@/lib/api/browser";
import type { components } from "@/lib/api/generated/schema";
import type { Locale } from "@/lib/i18n/config";

import { roomCopy } from "./room-copy";

type Room = components["schemas"]["RoomResponse"];

export function CreateRoomForm({
  locale,
  onCreated,
}: Readonly<{ locale: Locale; onCreated?: () => void }>) {
  const router = useRouter();
  const text = roomCopy(locale);
  const [title, setTitle] = useState("");
  const [visibility, setVisibility] = useState<"public" | "invite_only">("invite_only");
  const [pending, setPending] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!title.trim()) {
      setError(text.invalidTitle);
      return;
    }
    setPending(true);
    setError(null);
    try {
      const { data } = await browserApiClient.post<Room>("/v1/rooms", {
        title: title.trim(),
        visibility,
      });
      onCreated?.();
      router.push(`/room/${data.id}`);
      router.refresh();
    } catch (failure) {
      setError(apiFailureFrom(failure).message);
      setPending(false);
    }
  }

  return (
    <form className="max-w-xl space-y-6" data-testid="create-room-form" onSubmit={submit}>
      <h1 className="text-2xl font-semibold tracking-tight">{text.create}</h1>
      <Field data-invalid={Boolean(error && !title.trim())}>
        <FieldLabel className="block text-sm font-medium" htmlFor="room-title">
          {text.title}
        </FieldLabel>
        <Input
          autoComplete="off"
          id="room-title"
          aria-invalid={Boolean(error && !title.trim())}
          aria-describedby={error && !title.trim() ? "room-title-error" : undefined}
          maxLength={300}
          onChange={(event) => setTitle(event.target.value)}
          value={title}
        />
        {error && !title.trim() ? <FieldError id="room-title-error">{error}</FieldError> : null}
      </Field>
      <FieldSet className="gap-2">
        <FieldLegend className="mb-0 text-sm font-medium">{text.visibility}</FieldLegend>
        <RadioGroup
          dir={locale === "fa" ? "rtl" : "ltr"}
          className="grid grid-cols-2 gap-2 rounded-xl border bg-muted/40 p-1"
          aria-label={text.visibility}
          value={visibility}
          onValueChange={(next) => {
            if (next === "public" || next === "invite_only") {
              setVisibility(next);
            }
          }}
        >
          {(["invite_only", "public"] as const).map((option) => (
            <FieldLabel
              htmlFor={`room-visibility-${option}`}
              className={`relative flex w-full min-h-11 cursor-pointer items-center justify-center rounded-lg has-[:focus-visible]:ring-2 has-[:focus-visible]:ring-ring px-3 text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring ${visibility === option ? "bg-background text-foreground shadow-sm" : "text-muted-foreground hover:text-foreground"}`}
              key={option}
            >
              <RadioGroupItem className="sr-only" id={`room-visibility-${option}`} value={option} />
              {option === "invite_only" ? text.inviteOnly : text.public}
            </FieldLabel>
          ))}
        </RadioGroup>
      </FieldSet>
      {error && title.trim() ? <FieldError>{error}</FieldError> : null}
      <Button disabled={pending} type="submit">
        {pending ? text.creating : text.create}
      </Button>
    </form>
  );
}
