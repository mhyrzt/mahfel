import Link from "next/link";
import { CalendarPlus, Plus, SquarePlus } from "lucide-react";

import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";

export function SidebarActions() {
  return (
    <div className="mt-auto border-t p-2">
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <Button aria-label="Create" className="size-12 rounded-xl" size="icon">
            <Plus className="size-5" strokeWidth={1.8} />
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="start" side="right" sideOffset={8}>
          <DropdownMenuLabel>Create</DropdownMenuLabel>
          <DropdownMenuSeparator />
          <DropdownMenuItem asChild>
            <Link href="/rooms/new">
              <SquarePlus className="size-4" />
              Create room
            </Link>
          </DropdownMenuItem>
          <DropdownMenuItem asChild>
            <Link href="/sessions/new">
              <CalendarPlus className="size-4" />
              Schedule session
            </Link>
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>
    </div>
  );
}
