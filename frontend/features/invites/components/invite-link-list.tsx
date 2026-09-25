import { Copy, Link2, RotateCw, Trash2 } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
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
        }
      : {
          title: "Invite links",
          create: "Create link",
          empty: "No invite links have been created.",
          copy: "Copy code",
          rotate: "Rotate code",
          revoke: "Revoke link",
          expires: "Expires",
        };
  return (
    <TooltipProvider>
      <Card data-testid="invite-links">
        <CardHeader className="flex-row items-center justify-between space-y-0">
          <CardTitle className="flex items-center gap-2 text-base">
            <Link2 className="size-4" aria-hidden="true" />
            {text.title}
          </CardTitle>
          {canManage ? (
            <Button onClick={onCreate} size="sm">
              {text.create}
            </Button>
          ) : null}
        </CardHeader>
        <CardContent>
          {links.length === 0 ? (
            <div className="rounded-lg border border-dashed p-6 text-center text-sm text-muted-foreground">
              {text.empty}
            </div>
          ) : (
            <ul className="space-y-3">
              {links.map((link) => (
                <li
                  className="flex flex-wrap items-center gap-3 rounded-lg border p-3"
                  key={link.id}
                >
                  <div className="min-w-0 flex-1">
                    <p className="font-mono text-sm font-medium">{link.code}</p>
                    <p className="text-xs text-muted-foreground">
                      {link.target} · {text.expires} {link.expiresAt}
                    </p>
                  </div>
                  <Badge variant={link.status === "active" ? "secondary" : "outline"}>
                    {link.status}
                  </Badge>
                  {canManage && link.status === "active" ? (
                    <div className="flex gap-1">
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
                </li>
              ))}
            </ul>
          )}
        </CardContent>
      </Card>
    </TooltipProvider>
  );
}
