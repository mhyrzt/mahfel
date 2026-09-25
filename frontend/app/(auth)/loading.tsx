import { PageSkeleton } from "@/features/shell/components/page-skeleton";

export default function Loading() {
  return (
    <main className="mx-auto flex min-h-dvh max-w-md flex-col justify-center px-6 py-16">
      <PageSkeleton shape="form" />
    </main>
  );
}
