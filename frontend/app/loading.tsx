import { Skeleton } from "@/components/ui/skeleton";

export default function Loading() {
  return (
    <main
      aria-busy="true"
      className="mx-auto flex min-h-dvh w-full max-w-7xl flex-col px-6 py-6 sm:px-10 sm:py-8"
      data-testid="site-loading"
    >
      <header className="flex items-center justify-between">
        <Skeleton className="h-7 w-28" />
        <div className="flex gap-2">
          <Skeleton className="size-10" />
          <Skeleton className="size-10" />
        </div>
      </header>
      <div className="grid flex-1 items-center gap-14 py-20 lg:grid-cols-[minmax(0,1fr)_minmax(18rem,0.72fr)]">
        <div className="max-w-2xl space-y-8">
          <Skeleton className="h-4 w-28" />
          <div className="space-y-5">
            <Skeleton className="h-14 w-full max-w-xl" />
            <Skeleton className="h-14 w-4/5" />
            <Skeleton className="h-5 w-full max-w-lg" />
          </div>
          <Skeleton className="h-11 w-44" />
        </div>
        <div className="hidden space-y-5 border-s-2 ps-7 lg:block">
          <Skeleton className="h-8 w-full" />
          <Skeleton className="h-5 w-40" />
        </div>
      </div>
      <footer className="border-t pt-5">
        <Skeleton className="h-4 w-64" />
      </footer>
    </main>
  );
}
