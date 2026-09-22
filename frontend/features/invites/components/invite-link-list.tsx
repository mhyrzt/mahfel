import { Copy, Link2, RotateCw, Trash2 } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "@/components/ui/tooltip";

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
}: Readonly<{ links: InviteLink[]; canManage?: boolean }>) {
  return (
    <TooltipProvider>
      <Card>
        <CardHeader className="flex-row items-center justify-between space-y-0">
          <CardTitle className="flex items-center gap-2 text-base">
            <Link2 className="size-4" aria-hidden="true" />
            Invite links
          </CardTitle>
          {canManage ? <Button size="sm">Create link</Button> : null}
        </CardHeader>
        <CardContent>
          {links.length === 0 ? (
            <div className="rounded-lg border border-dashed p-6 text-center text-sm text-muted-foreground">
              No invite links have been created.
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
                      {link.target} · Expires {link.expiresAt}
                    </p>
                  </div>
                  <Badge variant={link.status === "active" ? "secondary" : "outline"}>
                    {link.status}
                  </Badge>
                  {canManage && link.status === "active" ? (
                    <div className="flex gap-1">
                      <Tooltip>
                        <TooltipTrigger asChild>
                          <Button aria-label={`Copy ${link.code}`} size="icon" variant="ghost">
                            <Copy className="size-4" />
                          </Button>
                        </TooltipTrigger>
                        <TooltipContent>Copy invite link</TooltipContent>
                      </Tooltip>
                      <Tooltip>
                        <TooltipTrigger asChild>
                          <Button aria-label={`Rotate ${link.code}`} size="icon" variant="ghost">
                            <RotateCw className="size-4" />
                          </Button>
                        </TooltipTrigger>
                        <TooltipContent>Rotate invite link</TooltipContent>
                      </Tooltip>
                      <Tooltip>
                        <TooltipTrigger asChild>
                          <Button aria-label={`Revoke ${link.code}`} size="icon" variant="ghost">
                            <Trash2 className="size-4 text-destructive" />
                          </Button>
                        </TooltipTrigger>
                        <TooltipContent>Revoke invite link</TooltipContent>
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
