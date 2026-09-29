"use client";

import { useEffect, useId, useState, type FormEvent } from "react";
import { useRouter } from "next/navigation";

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
      <div className="space-y-2">
        <label className="block text-sm font-medium" htmlFor="session-room">
          {text.roomSelection}
        </label>
        <select
          className="h-10 w-full rounded-md border border-input bg-background px-3 text-sm focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
          id="session-room"
          onChange={(event) => {
            setCanManage(false);
            setCheckingRole(true);
            setRoomId(event.target.value);
          }}
          value={roomId}
        >
          {rooms.map((room) => (
            <option key={room.id} value={room.id}>
              {room.title}
            </option>
          ))}
        </select>
      </div>
      {!checkingRole && !canManage ? (
        <p className="text-sm text-muted-foreground" role="status">
          {text.managerOnly}
        </p>
      ) : null}
      <div className="space-y-2">
        <label className="block text-sm font-medium" htmlFor="session-title">
          {text.sessionTitle}
        </label>
        <Input
          id="session-title"
          maxLength={300}
          onChange={(event) => setTitle(event.target.value)}
          value={title}
        />
      </div>
      <div className="space-y-2">
        <label className="block text-sm font-medium" htmlFor="session-start">
          {text.plannedStart}
        </label>
        <Input
          id="session-start"
          type="datetime-local"
          onChange={(event) => setStart(event.target.value)}
          value={start}
        />
      </div>
      <div className="space-y-2">
        <label className="block text-sm font-medium" htmlFor="session-duration">
          {text.duration}
        </label>
        <Input
          id="session-duration"
          min={1}
          max={10080}
          type="number"
          onChange={(event) => setDuration(event.target.value)}
          value={duration}
        />
      </div>
      <div className="flex items-center gap-2 text-sm">
        <Checkbox
          id={autoRecordId}
          checked={autoRecord}
          onCheckedChange={(checked) => setAutoRecord(checked === true)}
        />
        <label htmlFor={autoRecordId}>{text.autoRecord}</label>
      </div>
      {error ? (
        <p className="text-sm text-destructive" role="alert">
          {error}
        </p>
      ) : null}
      <Button disabled={pending || checkingRole || !canManage} type="submit">
        {pending ? text.creating : text.newSession}
      </Button>
    </form>
  );
}
