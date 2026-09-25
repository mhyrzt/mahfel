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
      <div className="space-y-2">
        <label className="block text-sm font-medium" htmlFor="room-visibility">
          {text.visibility}
        </label>
        <select
          className="h-10 w-full rounded-md border border-input bg-background px-3 text-sm focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
          id="room-visibility"
          onChange={(event) => setVisibility(event.target.value as "public" | "invite_only")}
          value={visibility}
        >
          <option value="invite_only">{text.inviteOnly}</option>
          <option value="public">{text.public}</option>
        </select>
      </div>
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
