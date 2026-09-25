import { Skeleton } from "@/components/ui/skeleton";

export function WorkspaceLoading() {
  return (
    <div aria-busy="true" className="min-h-screen bg-muted/40" data-testid="workspace-loading">
      <aside className="fixed inset-y-0 start-0 hidden w-16 flex-col items-center gap-5 border-e bg-background py-3 md:flex">
        <Skeleton className="size-10 rounded-xl" />
        <Skeleton className="size-8" />
        <Skeleton className="size-8" />
        <Skeleton className="mt-auto size-9 rounded-full" />
      </aside>
      <main className="mx-auto w-full max-w-7xl space-y-8 p-4 md:ps-24 md:pe-8 md:py-8">
        <div className="space-y-3">
          <Skeleton className="h-8 w-56" />
          <Skeleton className="h-4 w-72" />
        </div>
        <div className="divide-y border-y">
          {Array.from({ length: 4 }, (_, index) => (
            <div className="flex h-16 items-center justify-between" key={index}>
              <Skeleton className="h-5 w-40" />
              <Skeleton className="h-4 w-16" />
            </div>
          ))}
        </div>
      </main>
      <nav className="fixed inset-x-0 bottom-0 flex h-16 items-center justify-around border-t bg-background md:hidden">
        <Skeleton className="size-8" />
        <Skeleton className="size-8" />
        <Skeleton className="size-8" />
      </nav>
    </div>
  );
}
