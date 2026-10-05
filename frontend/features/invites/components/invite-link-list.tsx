import { Copy, Link2, Plus, RotateCw, Trash2 } from "lucide-react";

import { Empty, EmptyDescription } from "@/components/ui/empty";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "@/components/ui/tooltip";
import type { Locale } from "@/lib/i18n/config";

export interface InviteLink {
  id: string;
  code: string;
  expiresAt: string;
  status: "active" | "revoked" | "expired";
  target: "room" | "session";
}

export function InviteLinkList({
  links,
  canManage = true,
  locale = "en",
  onCreate,
  onCopy,
  onRotate,
  onRevoke,
}: Readonly<{
  links: InviteLink[];
  canManage?: boolean;
  locale?: Locale;
  onCreate?: () => void;
  onCopy?: (link: InviteLink) => void;
  onRotate?: (link: InviteLink) => void;
  onRevoke?: (link: InviteLink) => void;
}>) {
  const text =
    locale === "fa"
      ? {
          title: "پیوندهای دعوت",
          create: "ساخت پیوند",
          empty: "هنوز پیوند دعوتی ساخته نشده است.",
          copy: "کپی کد",
          rotate: "ساخت کد تازه",
          revoke: "لغو پیوند",
          expires: "انقضا",
          room: "محفل",
          session: "دیدار",
        }
      : {
          title: "Invite links",
          create: "Create link",
          empty: "No invite links have been created.",
          copy: "Copy code",
          rotate: "Rotate code",
          revoke: "Revoke link",
          expires: "Expires",
          room: "Room",
          session: "Session",
        };
  return (
    <TooltipProvider>
      <section className="min-w-0 space-y-3" data-testid="invite-links">
        <header className="flex items-center justify-between gap-2">
          <h3 className="flex min-w-0 items-center gap-2 text-sm font-semibold">
            <Link2 className="size-4" aria-hidden="true" />
            {text.title}
          </h3>
          {canManage ? (
            <Button aria-label={text.create} onClick={onCreate} size="sm" variant="outline">
              <Plus aria-hidden="true" className="size-4" />
              <span className="hidden sm:inline">{text.create}</span>
            </Button>
          ) : null}
        </header>
        <div>
          {links.length === 0 ? (
            <Empty className="rounded-lg border border-dashed p-6 text-center text-sm text-muted-foreground">
              <EmptyDescription>{text.empty}</EmptyDescription>
            </Empty>
          ) : (
            <ul className="space-y-3">
              {links.map((link) => (
                <li
                  className="min-w-0 space-y-3 rounded-lg border bg-background/60 p-3"
                  key={link.id}
                >
                  <div className="flex min-w-0 items-start justify-between gap-2">
                    <code className="min-w-0 break-all font-mono text-sm font-semibold">
                      {link.code}
                    </code>
                    <Badge
                      className="shrink-0"
                      variant={link.status === "active" ? "secondary" : "outline"}
                    >
                      {link.status}
                    </Badge>
                  </div>
                  <div className="flex items-center justify-between gap-2">
                    <p className="min-w-0 text-xs text-muted-foreground">
                      {text[link.target]} · {text.expires} {link.expiresAt}
                    </p>
                    {canManage && link.status === "active" ? (
                      <div className="flex shrink-0 gap-0.5">
                        <Tooltip>
                          <TooltipTrigger asChild>
                            <Button
                              aria-label={`${text.copy} ${link.code}`}
                              onClick={() => onCopy?.(link)}
                              size="icon"
                              variant="ghost"
                            >
                              <Copy className="size-4" />
                            </Button>
                          </TooltipTrigger>
                          <TooltipContent>{text.copy}</TooltipContent>
                        </Tooltip>
                        <Tooltip>
                          <TooltipTrigger asChild>
                            <Button
                              aria-label={`${text.rotate} ${link.code}`}
                              onClick={() => onRotate?.(link)}
                              size="icon"
                              variant="ghost"
                            >
                              <RotateCw className="size-4" />
                            </Button>
                          </TooltipTrigger>
                          <TooltipContent>{text.rotate}</TooltipContent>
                        </Tooltip>
                        <Tooltip>
                          <TooltipTrigger asChild>
                            <Button
                              aria-label={`${text.revoke} ${link.code}`}
                              onClick={() => onRevoke?.(link)}
                              size="icon"
                              variant="ghost"
                            >
                              <Trash2 className="size-4 text-destructive" />
                            </Button>
                          </TooltipTrigger>
                          <TooltipContent>{text.revoke}</TooltipContent>
                        </Tooltip>
                      </div>
                    ) : null}
                  </div>
                </li>
              ))}
            </ul>
          )}
        </div>
      </section>
    </TooltipProvider>
  );
}
