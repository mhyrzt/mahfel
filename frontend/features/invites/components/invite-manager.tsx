"use client";

import { Alert, AlertDescription } from "@/components/ui/alert";
import { useCallback, useEffect, useState } from "react";

import { browserApiClient } from "@/lib/api/browser";
import type { components } from "@/lib/api/generated/schema";
import type { Locale } from "@/lib/i18n/config";

import { InviteLinkList, type InviteLink } from "./invite-link-list";

type ApiLink = components["schemas"]["InviteLinkResponse"];

function displayLinks(links: ApiLink[], locale: Locale): InviteLink[] {
  return links.map((link) => ({
    id: link.id,
    code: link.display_code,
    expiresAt: new Intl.DateTimeFormat(locale === "fa" ? "fa-IR" : "en-US", {
      dateStyle: "medium",
    }).format(new Date(link.expires_at)),
    status: inviteStatus(link.status),
    target: link.target_kind,
  }));
}

function inviteStatus(status: ApiLink["status"]): InviteLink["status"] {
  if (status === "active" || status === "revoked") {
    return status;
  }
  return "expired";
}

export function InviteManager({
  target,
  targetId,
  locale,
}: Readonly<{ target: "room" | "session"; targetId: string; locale: Locale }>) {
  const [links, setLinks] = useState<InviteLink[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const path = `/v1/${target === "room" ? "rooms" : "sessions"}/${encodeURIComponent(targetId)}/invite-links`;
  const unavailable =
    locale === "fa" ? "به‌روزرسانی پیوندها ممکن نشد." : "Invite links could not be updated.";

  const refresh = useCallback(async () => {
    try {
      const { data } = await browserApiClient.get<{ links: ApiLink[] }>(path);
      setLinks(displayLinks(data.links, locale));
      setError(null);
    } catch {
      setError(unavailable);
    }
  }, [locale, path, unavailable]);

  useEffect(() => {
    void Promise.resolve().then(refresh);
  }, [refresh]);

  async function mutate(operation: "create" | "rotate" | "revoke", link?: InviteLink) {
    try {
      if (operation === "create") {
        await browserApiClient.post(path, {});
      } else {
        await browserApiClient.post(
          `/v1/invite-links/${encodeURIComponent(link?.id ?? "")}/${operation}`,
          {},
        );
      }
      await refresh();
    } catch {
      setError(unavailable);
    }
  }

  async function copy(link: InviteLink) {
    try {
      await navigator.clipboard.writeText(
        `${window.location.origin}/join?code=${encodeURIComponent(link.code)}`,
      );
      setNotice(locale === "fa" ? "پیوند دعوت کپی شد." : "Invite link copied.");
    } catch {
      setError(unavailable);
    }
  }

  return (
    <section className="space-y-3" data-testid={`${target}-invite-manager`}>
      {error ? (
        <Alert variant="destructive" className="text-sm text-destructive">
          <AlertDescription>{error}</AlertDescription>
        </Alert>
      ) : null}
      {notice ? (
        <p role="status" className="text-sm text-muted-foreground">
          {notice}
        </p>
      ) : null}
      <InviteLinkList
        links={links}
        locale={locale}
        onCopy={(link) => void copy(link)}
        onCreate={() => void mutate("create")}
        onRevoke={(link) => void mutate("revoke", link)}
        onRotate={(link) => void mutate("rotate", link)}
      />
    </section>
  );
}
