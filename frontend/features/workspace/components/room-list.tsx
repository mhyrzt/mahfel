import { ArrowUpRight, CircleDot, DoorOpen, UsersRound } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { ScrollArea } from "@/components/ui/scroll-area";

import type { Room } from "./workspace-types";

interface RoomListProps {
  rooms: Room[];
}

export function RoomList({ rooms }: Readonly<RoomListProps>) {
  return (
    <section className="flex min-h-0 flex-1 flex-col" aria-labelledby="rooms-heading">
      <div className="mb-4">
        <div>
          <p className="text-sm font-semibold" id="rooms-heading">
            Rooms
          </p>
          <p className="text-sm text-muted-foreground">Your spaces and active sessions.</p>
        </div>
      </div>
      {rooms.length === 0 ? (
        <Card className="border-dashed shadow-none">
          <CardContent className="flex min-h-60 flex-col items-center justify-center gap-3 text-center">
            <DoorOpen className="size-7 text-muted-foreground" aria-hidden="true" />
            <div>
              <p className="font-medium">No rooms available</p>
              <p className="text-sm text-muted-foreground">
                Create a room or ask a manager for access.
              </p>
            </div>
          </CardContent>
        </Card>
      ) : (
        <ScrollArea className="min-h-0 flex-1 pr-3">
          <div className="grid gap-3 sm:grid-cols-2 xl:grid-cols-3">
            {rooms.map((room) => (
              <RoomCard key={room.id} room={room} />
            ))}
          </div>
        </ScrollArea>
      )}
    </section>
  );
}

function RoomCard({ room }: Readonly<{ room: Room }>) {
  const statusLabel =
    room.status === "live" ? "Live" : room.status === "scheduled" ? "Scheduled" : "No session";

  return (
    <Card className="group shadow-none transition-all duration-200 hover:-translate-y-0.5 hover:border-primary/30 hover:shadow-md">
      <CardContent className="flex min-h-44 flex-col p-5">
        <div className="flex items-start justify-between gap-3">
          <Badge className="gap-1" variant={room.status === "live" ? "default" : "secondary"}>
            {room.status === "live" ? <CircleDot className="size-3" aria-hidden="true" /> : null}
            {statusLabel}
          </Badge>
          <Button
            aria-label={`Open ${room.name}`}
            className="opacity-0 transition-opacity group-hover:opacity-100 focus-visible:opacity-100"
            size="icon"
            variant="ghost"
          >
            <ArrowUpRight className="size-4" />
          </Button>
        </div>
        <div className="mt-auto space-y-2 pt-6">
          <div>
            <h2 className="text-base font-semibold tracking-tight">{room.name}</h2>
            <p className="mt-1 line-clamp-2 text-sm leading-5 text-muted-foreground">
              {room.description}
            </p>
          </div>
          <div className="flex items-center justify-between text-xs text-muted-foreground">
            <span className="flex items-center gap-1.5">
              <UsersRound className="size-3.5" aria-hidden="true" />
              {room.participantCount} people
            </span>
            {room.sessionLabel ? <span className="truncate pl-2">{room.sessionLabel}</span> : null}
          </div>
        </div>
      </CardContent>
    </Card>
  );
}
