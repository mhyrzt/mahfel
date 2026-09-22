import { Clock3 } from "lucide-react";

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";

export function WaitingRoomPanel({ roomName }: Readonly<{ roomName: string }>) {
  return (
    <Card className="mx-auto max-w-lg">
      <CardHeader className="items-center text-center">
        <div className="mb-2 rounded-full bg-muted p-3">
          <Clock3 className="size-6 text-muted-foreground" aria-hidden="true" />
        </div>
        <CardTitle>Waiting for admission</CardTitle>
        <CardDescription>
          Your request to join {roomName} is pending. Media controls will be available after
          approval.
        </CardDescription>
      </CardHeader>
      <CardContent>
        <p className="rounded-lg bg-muted px-3 py-2 text-center text-sm text-muted-foreground">
          Keep this page open while a manager reviews your request.
        </p>
      </CardContent>
    </Card>
  );
}
