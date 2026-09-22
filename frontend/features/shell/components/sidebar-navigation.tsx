import Link from "next/link";
import { CalendarDays, DoorOpen, List } from "lucide-react";

interface NavigationItem {
  href: string;
  icon: typeof DoorOpen;
  label: string;
  quickLinks: { href: string; label: string }[];
}

const navigation: NavigationItem[] = [
  {
    href: "/rooms",
    icon: DoorOpen,
    label: "Rooms",
    quickLinks: [{ href: "/rooms", label: "All rooms" }],
  },
  {
    href: "/sessions",
    icon: CalendarDays,
    label: "Sessions",
    quickLinks: [
      { href: "/sessions", label: "Upcoming sessions" },
      { href: "/sessions?date=today", label: "Today’s schedule" },
    ],
  },
];

export function SidebarNavigation() {
  return (
    <nav className="space-y-2 p-2" aria-label="Workspace navigation">
      {navigation.map(({ href, icon: Icon, label, quickLinks }) => (
        <div className="group relative" key={label}>
          <Link
            aria-label={label}
            className="flex size-12 items-center justify-center rounded-xl text-muted-foreground transition-colors hover:bg-accent hover:text-accent-foreground focus-visible:bg-accent focus-visible:text-accent-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
            href={href}
          >
            <Icon className="size-5" strokeWidth={1.8} />
          </Link>
          <div className="pointer-events-none absolute left-full top-0 z-30 w-48 translate-x-1 rounded-r-xl rounded-bl-xl border bg-popover p-1.5 opacity-0 shadow-lg transition-all duration-150 group-hover:pointer-events-auto group-hover:translate-x-0 group-hover:opacity-100 group-focus-within:pointer-events-auto group-focus-within:translate-x-0 group-focus-within:opacity-100">
            <p className="px-2 py-1.5 text-xs font-semibold text-muted-foreground">{label}</p>
            {quickLinks.map((quickLink) => (
              <Link
                className="flex items-center gap-2 rounded-lg px-2 py-2 text-sm transition-colors hover:bg-accent focus-visible:bg-accent focus-visible:outline-none"
                href={quickLink.href}
                key={quickLink.href}
              >
                <List className="size-3.5" />
                {quickLink.label}
              </Link>
            ))}
          </div>
        </div>
      ))}
    </nav>
  );
}
