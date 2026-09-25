"use client";

import { useCallback, useEffect, useState, type FormEvent } from "react";
import { Plus, Trash2 } from "lucide-react";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { browserApiClient } from "@/lib/api/browser";
import type { Locale } from "@/lib/i18n/config";

import { roomCopy } from "./room-copy";

export interface RoomMemberRecord {
  user_id: string;
  role: "manager" | "member";
}

export function RoomMembers({
  roomId,
  actorId,
  locale,
  initialMembers,
}: Readonly<{
  roomId: string;
  actorId?: string;
  locale: Locale;
  initialMembers?: RoomMemberRecord[];
}>) {
  const text = roomCopy(locale);
  const path = `/v1/rooms/${encodeURIComponent(roomId)}/members`;
  const [members, setMembers] = useState<RoomMemberRecord[]>(initialMembers ?? []);
  const [loading, setLoading] = useState(initialMembers === undefined);
  const [pending, setPending] = useState(false);
  const [error, setError] = useState(false);
  const [userId, setUserId] = useState("");
  const [confirming, setConfirming] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    try {
      const { data } = await browserApiClient.get<{ members: RoomMemberRecord[] }>(path);
      setMembers(data.members);
      setError(false);
    } catch {
      setError(true);
    } finally {
      setLoading(false);
    }
  }, [path]);

  useEffect(() => {
    if (initialMembers === undefined) {
      void Promise.resolve().then(refresh);
    }
  }, [initialMembers, refresh]);

  async function add(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setPending(true);
    try {
      const { data } = await browserApiClient.post<RoomMemberRecord>(path, {
        user_id: userId.trim(),
        role: "member",
      });
      setMembers((current) => [...current, data]);
      setUserId("");
      setError(false);
    } catch {
      setError(true);
    } finally {
      setPending(false);
    }
  }

  async function changeRole(member: RoomMemberRecord, role: RoomMemberRecord["role"]) {
    setPending(true);
    try {
      const { data } = await browserApiClient.patch<RoomMemberRecord>(
        `${path}/${encodeURIComponent(member.user_id)}`,
        { user_id: member.user_id, role },
      );
      setMembers((current) => current.map((item) => (item.user_id === data.user_id ? data : item)));
      setError(false);
    } catch {
      setError(true);
    } finally {
      setPending(false);
    }
  }

  async function remove(member: RoomMemberRecord) {
    setPending(true);
    try {
      await browserApiClient.delete(`${path}/${encodeURIComponent(member.user_id)}`);
      setMembers((current) => current.filter((item) => item.user_id !== member.user_id));
      setConfirming(null);
      setError(false);
    } catch {
      setError(true);
    } finally {
      setPending(false);
    }
  }

  const lastManager = members.filter((member) => member.role === "manager").length === 1;

  return (
    <section className="space-y-5" data-testid="room-members">
      <div>
        <h2 className="text-lg font-semibold">{text.members}</h2>
        <p className="text-sm text-muted-foreground">{text.membersHelp}</p>
      </div>
      <form className="flex flex-col gap-2 sm:flex-row" onSubmit={(event) => void add(event)}>
        <Input
          aria-label={text.userId}
          className="min-w-0 flex-1"
          onChange={(event) => setUserId(event.target.value)}
          pattern="[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}"
          placeholder={text.userId}
          required
          value={userId}
        />
        <Button disabled={pending} type="submit">
          <Plus aria-hidden="true" className="size-4" />
          {text.addMember}
        </Button>
      </form>
      {error ? (
        <p className="text-sm text-destructive" role="alert">
          {text.memberActionFailed}
        </p>
      ) : null}
      {loading ? (
        <p className="text-sm text-muted-foreground" role="status">
          {text.loadingMembers}
        </p>
      ) : null}
      {!loading && members.length === 0 ? (
        <p className="text-sm text-muted-foreground">{text.noMembers}</p>
      ) : null}
      <ul className="divide-y" data-testid="room-member-list">
        {members.map((member) => {
          const protectedManager = member.role === "manager" && lastManager;
          return (
            <li
              className="flex flex-col gap-3 py-3 sm:flex-row sm:items-center"
              data-testid={`room-member-${member.user_id}`}
              key={member.user_id}
            >
              <span className="min-w-0 flex-1 truncate font-mono text-xs" title={member.user_id}>
                {member.user_id === actorId ? text.you : member.user_id}
              </span>
              <div className="flex flex-wrap items-center gap-2">
                <select
                  aria-label={`${text.memberRole}: ${member.user_id}`}
                  className="h-9 rounded-md border bg-background px-2 text-sm focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                  disabled={pending || protectedManager}
                  onChange={(event) =>
                    void changeRole(member, event.target.value as RoomMemberRecord["role"])
                  }
                  value={member.role}
                >
                  <option value="member">{text.member}</option>
                  <option value="manager">{text.manager}</option>
                </select>
                {confirming === member.user_id ? (
                  <>
                    <Button
                      disabled={pending}
                      onClick={() => void remove(member)}
                      size="sm"
                      type="button"
                      variant="destructive"
                    >
                      {text.confirmRemoveMember}
                    </Button>
                    <Button
                      onClick={() => setConfirming(null)}
                      size="sm"
                      type="button"
                      variant="ghost"
                    >
                      {text.cancel}
                    </Button>
                  </>
                ) : (
                  <Button
                    aria-label={`${text.removeMember}: ${member.user_id}`}
                    disabled={pending || protectedManager}
                    onClick={() => setConfirming(member.user_id)}
                    size="icon"
                    type="button"
                    variant="ghost"
                  >
                    <Trash2 aria-hidden="true" className="size-4" />
                  </Button>
                )}
              </div>
            </li>
          );
        })}
      </ul>
    </section>
  );
}
