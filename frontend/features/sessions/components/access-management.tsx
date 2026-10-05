"use client";

import { Alert, AlertDescription } from "@/components/ui/alert";
import { Checkbox } from "@/components/ui/checkbox";
import {
  Table,
  TableHeader,
  TableBody,
  TableRow,
  TableHead,
  TableCell,
} from "@/components/ui/table";
import { useEffect, useState } from "react";

import { browserApiClient } from "@/lib/api/browser";
import type { Locale } from "@/lib/i18n/config";
import { messagesFor } from "@/lib/i18n/messages";

type AccessField =
  "can_publish_audio" | "can_publish_video" | "can_share_screen" | "can_present" | "can_edit";

interface Grant {
  actor_id: string;
  display_name: string;
  can_edit: boolean;
  can_publish_audio: boolean;
  can_publish_video: boolean;
  can_share_screen: boolean;
  can_present: boolean;
  is_manager: boolean;
}

export function AccessManagement({
  actorId,
  sessionId,
  locale,
}: Readonly<{ actorId?: string; sessionId: string; locale: Locale }>) {
  const text = messagesFor(locale).collaboration;
  const [grants, setGrants] = useState<Grant[]>([]);
  const [error, setError] = useState(false);
  const [busy, setBusy] = useState<string | null>(null);

  useEffect(() => {
    let active = true;
    void browserApiClient
      .get<{ participants: Grant[] }>(
        `/v1/sessions/${encodeURIComponent(sessionId)}/collaboration/grants`,
      )
      .then(({ data }) => {
        if (active) {
          setGrants(data.participants);
        }
      })
      .catch(() => {
        if (active) {
          setError(true);
        }
      });
    return () => {
      active = false;
    };
  }, [sessionId]);

  async function setAccess(grant: Grant, field: AccessField, enabled: boolean) {
    if (grant.actor_id === actorId || grant.is_manager) {
      return;
    }
    setBusy(grant.actor_id);
    setError(false);
    try {
      await browserApiClient.put(
        `/v1/sessions/${encodeURIComponent(sessionId)}/collaboration/grants/${encodeURIComponent(grant.actor_id)}`,
        { [field]: enabled },
      );
      setGrants((current) =>
        current.map((item) =>
          item.actor_id === grant.actor_id ? { ...item, [field]: enabled } : item,
        ),
      );
    } catch {
      setError(true);
    } finally {
      setBusy(null);
    }
  }

  const fields: { key: AccessField; label: string }[] = [
    { key: "can_publish_audio", label: text.accessTalk },
    { key: "can_publish_video", label: text.accessCamera },
    { key: "can_share_screen", label: text.accessScreen },
    { key: "can_present", label: text.accessPresent },
    { key: "can_edit", label: text.accessEdit },
  ];

  return (
    <div className="space-y-3">
      <h3 className="font-semibold">{text.permissions}</h3>
      {error ? (
        <Alert variant="destructive" className="text-sm text-destructive">
          <AlertDescription>{text.unavailable}</AlertDescription>
        </Alert>
      ) : null}
      <div className="overflow-x-auto rounded-xl border bg-card">
        <Table className="w-full min-w-[28rem] border-collapse text-xs">
          <TableHeader className="bg-muted/60 text-muted-foreground">
            <TableRow>
              <TableHead
                className="sticky start-0 z-10 min-w-28 bg-muted/60 px-3 py-3 text-start font-medium"
                scope="col"
              >
                {text.permissions}
              </TableHead>
              {fields.map(({ key, label }) => (
                <TableHead className="px-2 py-3 text-center font-medium" key={key} scope="col">
                  {label}
                </TableHead>
              ))}
            </TableRow>
          </TableHeader>
          <TableBody>
            {grants.map((grant) => {
              const locked = grant.actor_id === actorId || grant.is_manager;
              return (
                <TableRow className="border-t border-border/70" key={grant.actor_id}>
                  <TableHead
                    className="sticky start-0 z-10 bg-card px-3 py-3 text-start font-medium"
                    scope="row"
                  >
                    <span className="block max-w-32 truncate" title={grant.display_name}>
                      {grant.display_name}
                    </span>
                    {locked ? (
                      <span className="block text-[11px] font-normal text-muted-foreground">
                        {grant.actor_id === actorId ? text.accessSelf : text.accessManaged}
                      </span>
                    ) : null}
                  </TableHead>
                  {fields.map(({ key, label }) => (
                    <TableCell className="px-2 py-3 text-center" key={key}>
                      <Checkbox
                        aria-label={`${grant.display_name}: ${label}`}
                        checked={grant[key]}
                        className="size-4 cursor-pointer accent-primary disabled:cursor-not-allowed disabled:opacity-50"
                        disabled={locked || busy !== null}
                        onCheckedChange={(checked) => void setAccess(grant, key, checked === true)}
                      />
                    </TableCell>
                  ))}
                </TableRow>
              );
            })}
          </TableBody>
        </Table>
      </div>
    </div>
  );
}
