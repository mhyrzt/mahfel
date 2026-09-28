"use client";

import { useState, type FormEvent } from "react";
import { useRouter } from "next/navigation";

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
      <div className="space-y-2">
        <label className="block text-sm font-medium" htmlFor="room-title">
          {text.title}
        </label>
        <Input
          autoComplete="off"
          id="room-title"
          maxLength={300}
          onChange={(event) => setTitle(event.target.value)}
          value={title}
        />
      </div>
      <fieldset className="space-y-2">
        <legend className="text-sm font-medium">{text.visibility}</legend>
        <div className="grid grid-cols-2 gap-2 rounded-xl border bg-muted/40 p-1">
          {(["invite_only", "public"] as const).map((option) => (
            <button
              aria-pressed={visibility === option}
              className={`min-h-11 rounded-lg px-3 text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring ${visibility === option ? "bg-background text-foreground shadow-sm" : "text-muted-foreground hover:text-foreground"}`}
              key={option}
              onClick={() => setVisibility(option)}
              type="button"
            >
              {option === "invite_only" ? text.inviteOnly : text.public}
            </button>
          ))}
        </div>
      </fieldset>
      {error ? (
        <p role="alert" className="text-sm text-destructive">
          {error}
        </p>
      ) : null}
      <Button disabled={pending} type="submit">
        {pending ? text.creating : text.create}
      </Button>
    </form>
  );
}
