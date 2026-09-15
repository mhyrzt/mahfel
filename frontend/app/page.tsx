import { Button } from "@/components/ui/button";
import { publicApiBaseUrl } from "@/lib/api/config";

export default function Home() {
  return (
    <main className="mx-auto flex min-h-screen max-w-4xl flex-col justify-center gap-8 px-6 py-16">
      <div className="space-y-4">
        <p className="text-sm font-medium uppercase tracking-[0.2em] text-neutral-500">
          Foundation ready
        </p>
        <h1 className="text-5xl font-semibold tracking-tight sm:text-7xl">Mahfel</h1>
        <p className="max-w-2xl text-lg leading-8 text-neutral-600">
          An education-focused realtime communication and collaboration platform. Product features
          will be built on this independently deployable web foundation.
        </p>
      </div>
      <div className="flex flex-wrap items-center gap-4">
        <Button disabled>Classrooms coming later</Button>
        <code className="text-sm text-neutral-500">API: {publicApiBaseUrl}</code>
      </div>
    </main>
  );
}
