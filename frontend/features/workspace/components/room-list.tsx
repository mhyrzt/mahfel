import Link from "next/link";
import { DoorOpen, UsersRound } from "lucide-react";

import { Card, CardContent } from "@/components/ui/card";
import { ScrollArea } from "@/components/ui/scroll-area";

import type { Room } from "./workspace-types";

interface RoomListProps {
  rooms: Room[];
}

export function RoomList({ rooms }: Readonly<RoomListProps>) {
  return (
    <section className="flex min-h-0 flex-1 flex-col" aria-label="Room list">
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
  return (
    <Link
      className="group block rounded-lg focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2"
      href={`/rooms/${room.id}`}
    >
      <Card className="shadow-none transition-all duration-200 group-hover:-translate-y-0.5 group-hover:border-primary/30 group-hover:shadow-md">
        <CardContent className="flex min-h-36 flex-col p-4">
          <div className="space-y-1.5">
            <div>
              <h2 className="text-base font-semibold tracking-tight">{room.name}</h2>
              <p className="mt-1 line-clamp-2 text-sm leading-5 text-muted-foreground">
                {room.description}
              </p>
            </div>
            <div className="flex items-center justify-between gap-2 text-[11px] leading-4 text-muted-foreground">
              <span className="flex items-center gap-1">
                <UsersRound className="size-3" aria-hidden="true" />
                {room.participantCount} people
              </span>
              {room.sessionLabel ? (
                <span className="truncate pl-2">{room.sessionLabel}</span>
              ) : null}
            </div>
          </div>
        </CardContent>
      </Card>
    </Link>
  );
}
