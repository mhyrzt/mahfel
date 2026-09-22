import { Button } from "@/components/ui/button";
import { LocaleSwitcher } from "@/features/i18n/components/locale-switcher";
import { publicApiBaseUrl } from "@/lib/api/config";
import { messagesFor } from "@/lib/i18n/messages";
import { getLocale } from "@/lib/i18n/server";

export default async function Home() {
  const locale = await getLocale();
  const messages = messagesFor(locale);

  return (
    <main className="mx-auto flex min-h-screen max-w-4xl flex-col justify-center gap-8 px-6 py-16">
      <div className="flex justify-end">
        <LocaleSwitcher locale={locale} />
      </div>
      <div className="space-y-4">
        <p className="text-sm font-medium uppercase tracking-[0.2em] text-neutral-500">
          {messages.home.eyebrow}
        </p>
        <h1 className="text-5xl font-semibold tracking-tight sm:text-7xl">{messages.home.title}</h1>
        <p className="max-w-2xl text-lg leading-8 text-neutral-600">{messages.home.description}</p>
      </div>
      <div className="flex flex-wrap items-center gap-4">
        <Button disabled>{messages.home.classroomsComingLater}</Button>
        <code className="text-sm text-neutral-500">
          {messages.home.api}: {publicApiBaseUrl}
        </code>
      </div>
    </main>
  );
}
