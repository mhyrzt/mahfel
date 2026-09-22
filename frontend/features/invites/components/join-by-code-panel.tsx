import { CheckCircle2, CircleAlert, Clock3, Link2 } from "lucide-react";

import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";

type JoinStatus = "entry" | "waiting" | "approved" | "rejected" | "unavailable";

const details: Record<
  Exclude<JoinStatus, "entry">,
  { title: string; description: string; icon: typeof Clock3 }
> = {
  waiting: {
    title: "Waiting for approval",
    description: "A manager needs to approve this request before you can join.",
    icon: Clock3,
  },
  approved: {
    title: "You are approved",
    description: "Your access has been confirmed. You can now join the target.",
    icon: CheckCircle2,
  },
  rejected: {
    title: "Request not approved",
    description: "A manager did not approve this request. Ask them for a new invitation if needed.",
    icon: CircleAlert,
  },
  unavailable: {
    title: "Link unavailable",
    description: "This invitation may have expired, been revoked, or no longer be available.",
    icon: CircleAlert,
  },
};

export function JoinByCodePanel({ status = "entry" }: Readonly<{ status?: JoinStatus }>) {
  if (status !== "entry") {
    const { title, description, icon: Icon } = details[status];

    return (
      <Card className="mx-auto max-w-md">
        <CardHeader className="items-center text-center">
          <div className="mb-2 rounded-full bg-muted p-3">
            <Icon className="size-6 text-muted-foreground" aria-hidden="true" />
          </div>
          <CardTitle>{title}</CardTitle>
          <CardDescription>{description}</CardDescription>
        </CardHeader>
        {status === "approved" ? (
          <CardContent>
            <Button className="w-full">Join now</Button>
          </CardContent>
        ) : null}
      </Card>
    );
  }

  return (
    <Card className="mx-auto max-w-md">
      <CardHeader>
        <div className="mb-2 rounded-full bg-muted p-3 w-fit">
          <Link2 className="size-6 text-muted-foreground" aria-hidden="true" />
        </div>
        <CardTitle>Join with an invite code</CardTitle>
        <CardDescription>
          Enter the code you received from a room or session manager.
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-3">
        <label className="grid gap-2 text-sm font-medium" htmlFor="invite-code">
          Invite code
          <Input id="invite-code" placeholder="xxx-yyyy-zzz" />
        </label>
        <Button className="w-full">Continue</Button>
      </CardContent>
    </Card>
  );
}
