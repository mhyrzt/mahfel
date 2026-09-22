import { Skeleton } from "@/components/ui/skeleton";

export function WorkspaceLoading() {
  return (
    <div className="min-h-screen bg-muted/40 lg:pl-16">
      <header className="h-16 border-b bg-background" />
      <main className="mx-auto w-full max-w-7xl space-y-6 p-4 lg:p-8">
        <div className="space-y-3">
          <Skeleton className="h-5 w-24" />
          <Skeleton className="h-9 w-72" />
        </div>
        <div className="grid gap-4 xl:grid-cols-[minmax(0,1fr)_20rem]">
          <Skeleton className="aspect-video" />
          <Skeleton className="h-96" />
        </div>
      </main>
    </div>
  );
}
