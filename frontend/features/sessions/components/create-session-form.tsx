"use client";

import { useEffect, useId, useState, type FormEvent } from "react";
import { useRouter } from "next/navigation";

import { Field, FieldLabel, FieldError } from "@/components/ui/field";
import { NativeSelect, NativeSelectOption } from "@/components/ui/native-select";
import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import { Input } from "@/components/ui/input";
import { roomCopy } from "@/features/rooms/room-copy";
import { browserApiClient } from "@/lib/api/browser";
import { apiFailureFrom } from "@/lib/api/errors";
import type { components } from "@/lib/api/generated/schema";
import type { Locale } from "@/lib/i18n/config";

type Room = components["schemas"]["RoomResponse"];
type Session = components["schemas"]["SessionResponse"];

export function CreateSessionForm({
  rooms,
  locale,
  initialRoomId,
  initialCanManage,
  onCreated,
}: Readonly<{
  rooms: Room[];
  locale: Locale;
  initialRoomId?: string;
  initialCanManage?: boolean;
  onCreated?: () => void;
}>) {
  const text = roomCopy(locale);
  const router = useRouter();
  const [roomId, setRoomId] = useState(initialRoomId ?? rooms[0]?.id ?? "");
  const [title, setTitle] = useState("");
  const [start, setStart] = useState("");
  const [duration, setDuration] = useState("");
  const autoRecordId = useId();
  const [autoRecord, setAutoRecord] = useState(true);
  const [pending, setPending] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [canManage, setCanManage] = useState(initialCanManage ?? false);
  const [checkingRole, setCheckingRole] = useState(initialCanManage === undefined);

  useEffect(() => {
    if (!roomId || (initialCanManage !== undefined && roomId === (initialRoomId ?? rooms[0]?.id))) {
      return undefined;
    }
    let active = true;
    void browserApiClient
      .get<{ can_manage_session: boolean }>(`/v1/rooms/${encodeURIComponent(roomId)}/membership`)
      .then(({ data }) => {
        if (active) {
          setCanManage(data.can_manage_session);
          setCheckingRole(false);
        }
      })
      .catch(() => {
        if (active) {
          setCanManage(false);
          setCheckingRole(false);
        }
      });
    return () => {
      active = false;
    };
  }, [initialCanManage, initialRoomId, roomId, rooms]);

  async function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!roomId || !canManage || checkingRole) {
      setError(text.chooseRoom);
      return;
    }
    setPending(true);
    setError(null);
    try {
      const { data } = await browserApiClient.post<Session>(
        `/v1/rooms/${encodeURIComponent(roomId)}/sessions`,
        {
          auto_record: autoRecord,
          title: title.trim() || null,
          scheduled_start_at: start ? new Date(start).toISOString() : null,
          scheduled_duration_minutes: duration ? Number(duration) : null,
        },
      );
      onCreated?.();
      router.push(`/session/${data.id}`);
      router.refresh();
    } catch (failure) {
      setError(apiFailureFrom(failure).message);
      setPending(false);
    }
  }

  if (rooms.length === 0) {
    return <p className="text-sm text-muted-foreground">{text.noRoomsForSession}</p>;
  }

  return (
    <form className="max-w-xl space-y-6" data-testid="create-session-form" onSubmit={submit}>
      <h1 className="text-2xl font-semibold tracking-tight">{text.newSession}</h1>
      <Field>
        <FieldLabel className="block text-sm font-medium" htmlFor="session-room">
          {text.roomSelection}
        </FieldLabel>
        <NativeSelect
          className="h-10 w-full bg-background"
          id="session-room"
          onChange={(event) => {
            setCanManage(false);
            setCheckingRole(true);
            setRoomId(event.target.value);
          }}
          value={roomId}
        >
          {rooms.map((room) => (
            <NativeSelectOption key={room.id} value={room.id}>
              {room.title}
            </NativeSelectOption>
          ))}
        </NativeSelect>
      </Field>
      {!checkingRole && !canManage ? (
        <p className="text-sm text-muted-foreground" role="status">
          {text.managerOnly}
        </p>
      ) : null}
      <Field>
        <FieldLabel className="block text-sm font-medium" htmlFor="session-title">
          {text.sessionTitle}
        </FieldLabel>
        <Input
          id="session-title"
          maxLength={300}
          onChange={(event) => setTitle(event.target.value)}
          value={title}
        />
      </Field>
      <Field>
        <FieldLabel className="block text-sm font-medium" htmlFor="session-start">
          {text.plannedStart}
        </FieldLabel>
        <Input
          id="session-start"
          type="datetime-local"
          onChange={(event) => setStart(event.target.value)}
          value={start}
        />
      </Field>
      <Field>
        <FieldLabel className="block text-sm font-medium" htmlFor="session-duration">
          {text.duration}
        </FieldLabel>
        <Input
          id="session-duration"
          min={1}
          max={10080}
          type="number"
          onChange={(event) => setDuration(event.target.value)}
          value={duration}
        />
      </Field>
      <Field orientation="horizontal">
        <Checkbox
          id={autoRecordId}
          checked={autoRecord}
          onCheckedChange={(checked) => setAutoRecord(checked === true)}
        />
        <FieldLabel htmlFor={autoRecordId}>{text.autoRecord}</FieldLabel>
      </Field>
      {error ? <FieldError>{error}</FieldError> : null}
      <Button disabled={pending || checkingRole || !canManage} type="submit">
        {pending ? text.creating : text.newSession}
      </Button>
    </form>
  );
}
