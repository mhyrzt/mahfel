"use client";

import { useEffect, useState, type FormEvent } from "react";
import Link from "next/link";

import { Field, FieldLabel } from "@/components/ui/field";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { browserApiClient } from "@/lib/api/browser";
import type { Locale } from "@/lib/i18n/config";

type JoinStatus = "entry" | "waiting" | "approved" | "rejected" | "unavailable";

const copy = {
  en: {
    title: "Join with an invite code",
    description: "Enter the code shared by a room or session manager.",
    code: "Invite code",
    continue: "Continue",
    waiting: "Waiting for approval",
    waitingDetail: "A manager will review your request before you can join.",
    approved: "Access approved",
    approvedDetail: "You can now open the room or session.",
    rejected: "Request not approved",
    rejectedDetail: "Ask the manager for another invitation if you still need access.",
    unavailable: "Invite unavailable",
    unavailableDetail: "The code may have expired, been revoked, or could not be found.",
    open: "Open destination",
    retry: "Try another code",
  },
  fa: {
    title: "ورود با کد دعوت",
    description: "کدی را که مدیر محفل یا دیدار فرستاده وارد کنید.",
    code: "کد دعوت",
    continue: "ادامه",
    waiting: "در انتظار تأیید",
    waitingDetail: "مدیر درخواست شما را بررسی می‌کند.",
    approved: "دسترسی تأیید شد",
    approvedDetail: "اکنون می‌توانید محفل یا دیدار را باز کنید.",
    rejected: "درخواست تأیید نشد",
    rejectedDetail: "اگر هنوز به دسترسی نیاز دارید، از مدیر دعوت تازه‌ای بخواهید.",
    unavailable: "دعوت در دسترس نیست",
    unavailableDetail: "کد ممکن است منقضی یا لغو شده باشد، یا وجود نداشته باشد.",
    open: "باز کردن مقصد",
    retry: "امتحان کد دیگر",
  },
} as const;

interface Redeemed {
  outcome: "already_member" | "waiting";
  request_id: string | null;
  target_kind: "room" | "session";
  target_id: string;
}

export function JoinByCodePanel({
  status = "entry",
  locale = "en",
  initialCode = "",
}: Readonly<{ status?: JoinStatus; locale?: Locale; initialCode?: string }>) {
  const text = copy[locale];
  const [current, setCurrent] = useState<JoinStatus>(status);
  const [code, setCode] = useState(initialCode);
  const [requestId, setRequestId] = useState<string | null>(null);
  const [destination, setDestination] = useState<string | null>(null);
  const [pending, setPending] = useState(false);

  useEffect(() => {
    if (!requestId || current !== "waiting") {
      return undefined;
    }
    let active = true;
    const interval = window.setInterval(() => {
      void browserApiClient
        .get<{ status: string }>(`/v1/invite-links/requests/${encodeURIComponent(requestId)}`)
        .then(({ data }) => {
          if (!active) {
            return;
          }
          if (data.status === "admitted") {
            setCurrent("approved");
          } else if (data.status === "rejected" || data.status === "expired") {
            setCurrent("rejected");
          }
        })
        .catch(() => undefined);
    }, 3_000);
    return () => {
      active = false;
      window.clearInterval(interval);
    };
  }, [current, requestId]);

  async function redeem(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!code.trim()) {
      return;
    }
    setPending(true);
    try {
      const { data } = await browserApiClient.post<Redeemed>("/v1/invite-links/redeem", {
        code: code.trim(),
      });
      setDestination(`/${data.target_kind === "room" ? "room" : "session"}/${data.target_id}`);
      setRequestId(data.request_id);
      setCurrent(data.outcome === "already_member" ? "approved" : "waiting");
    } catch {
      setCurrent("unavailable");
    } finally {
      setPending(false);
    }
  }

  let title: string = text.unavailable;
  let detail: string = text.unavailableDetail;
  if (current === "waiting") {
    title = text.waiting;
    detail = text.waitingDetail;
  } else if (current === "approved") {
    title = text.approved;
    detail = text.approvedDetail;
  } else if (current === "rejected") {
    title = text.rejected;
    detail = text.rejectedDetail;
  }

  return (
    <section
      className="mx-auto max-w-md space-y-6 rounded-xl border bg-card p-6"
      data-testid="join-by-code"
    >
      {current === "entry" ? (
        <form className="space-y-5" onSubmit={(event) => void redeem(event)}>
          <div className="space-y-2">
            <h1 className="text-xl font-semibold">{text.title}</h1>
            <p className="text-sm text-muted-foreground">{text.description}</p>
          </div>
          <Field>
            <FieldLabel className="block text-sm font-medium" htmlFor="invite-code">
              {text.code}
            </FieldLabel>
            <Input
              autoComplete="off"
              id="invite-code"
              onChange={(event) => setCode(event.target.value)}
              placeholder="xxx-yyyy-zzz"
              value={code}
            />
          </Field>
          <Button disabled={pending || !code.trim()} type="submit">
            {text.continue}
          </Button>
        </form>
      ) : (
        <div className="space-y-5" role="status">
          <div className="space-y-2">
            <h1 className="text-xl font-semibold">{title}</h1>
            <p className="text-sm text-muted-foreground">{detail}</p>
          </div>
          {current === "approved" && destination ? (
            <Button asChild>
              <Link href={destination}>{text.open}</Link>
            </Button>
          ) : null}
          {current === "rejected" || current === "unavailable" ? (
            <Button
              onClick={() => {
                setCurrent("entry");
                setCode("");
              }}
              variant="outline"
            >
              {text.retry}
            </Button>
          ) : null}
        </div>
      )}
    </section>
  );
}
