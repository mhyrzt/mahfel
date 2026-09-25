import { Skeleton } from "@/components/ui/skeleton";

type Shape = "detail" | "form" | "session" | "account" | "join";

export function PageSkeleton({ shape }: Readonly<{ shape: Shape }>) {
  if (shape === "form" || shape === "account" || shape === "join") {
    return (
      <section
        aria-busy="true"
        aria-label="Loading page"
        className="max-w-xl space-y-8"
        data-testid={`${shape}-loading`}
      >
        <div className="space-y-3">
          <Skeleton className="h-8 w-56" />
          {shape === "account" ? <Skeleton className="h-4 w-72" /> : null}
        </div>
        <div className="space-y-6">
          {Array.from({ length: shape === "join" ? 1 : 3 }, (_, index) => (
            <div className="space-y-2" key={index}>
              <Skeleton className="h-4 w-24" />
              <Skeleton className="h-10 w-full" />
            </div>
          ))}
        </div>
        <Skeleton className="h-10 w-36" />
      </section>
    );
  }

  if (shape === "session") {
    return (
      <section
        aria-busy="true"
        aria-label="Loading session"
        className="space-y-6"
        data-testid="session-loading"
      >
        <div className="space-y-3 border-b pb-6">
          <Skeleton className="h-7 w-64" />
          <Skeleton className="h-4 w-32" />
        </div>
        <div className="grid gap-4 lg:grid-cols-[minmax(0,1fr)_18rem]">
          <Skeleton className="aspect-video w-full rounded-xl" />
          <Skeleton className="h-72 rounded-xl" />
        </div>
        <Skeleton className="mx-auto h-14 w-64 rounded-2xl" />
      </section>
    );
  }

  return (
    <section
      aria-busy="true"
      aria-label="Loading room"
      className="space-y-8"
      data-testid="detail-loading"
    >
      <header className="flex flex-wrap items-end justify-between gap-4 border-b pb-6">
        <div className="space-y-3">
          <Skeleton className="h-4 w-20" />
          <Skeleton className="h-8 w-56" />
          <Skeleton className="h-4 w-44" />
        </div>
        <Skeleton className="h-10 w-32" />
      </header>
      <div className="divide-y border-y">
        {Array.from({ length: 2 }, (_, index) => (
          <div className="flex min-h-16 items-center justify-between gap-4" key={index}>
            <Skeleton className="h-5 w-48 max-w-[60%]" />
            <Skeleton className="h-4 w-16" />
          </div>
        ))}
      </div>
      <div className="space-y-3">
        <Skeleton className="h-6 w-20" />
        <Skeleton className="h-28 w-full rounded-xl" />
      </div>
    </section>
  );
}
