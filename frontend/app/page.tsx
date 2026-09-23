import Link from "next/link";

import { Button } from "@/components/ui/button";
import { ThemeSwitcher } from "@/features/appearance/components/theme-switcher";
import { isTheme, themeCookieName } from "@/features/appearance/theme";
import { LocaleSwitcher } from "@/features/i18n/components/locale-switcher";
import { cookies } from "next/headers";
import { messagesFor } from "@/lib/i18n/messages";
import { getLocale } from "@/lib/i18n/server";

export default async function Home() {
  const locale = await getLocale();
  const messages = messagesFor(locale);
  const cookieStore = await cookies();
  const themeCookie = cookieStore.get(themeCookieName)?.value;
  const theme = isTheme(themeCookie) ? themeCookie : "system";

  return (
    <main className="relative isolate flex min-h-dvh flex-col overflow-hidden bg-background px-6 py-6 sm:px-10 sm:py-8">
      <div
        aria-hidden="true"
        className="pointer-events-none absolute inset-y-0 end-0 -z-10 hidden w-[42%] border-s bg-muted/50 lg:block"
      />
      <header className="mx-auto flex w-full max-w-7xl items-center justify-between">
        <Link className="text-xl font-semibold tracking-tight" href="/">
          Mahfel<span className="text-primary">.</span>
        </Link>
        <div className="flex items-center gap-2">
          <ThemeSwitcher locale={locale} theme={theme} />
          <LocaleSwitcher locale={locale} />
        </div>
      </header>
      <div className="mx-auto grid w-full max-w-7xl flex-1 items-center gap-14 py-20 lg:grid-cols-[minmax(0,1fr)_minmax(18rem,0.72fr)] lg:gap-24 lg:py-28">
        <section className="max-w-2xl space-y-8">
          <p className="text-sm font-medium text-primary">{messages.home.eyebrow}</p>
          <div className="space-y-5">
            <h1 className="max-w-xl text-balance text-4xl font-semibold leading-[1.2] tracking-tight sm:text-6xl sm:leading-[1.15]">
              {messages.home.title}
            </h1>
            <p className="max-w-xl text-base leading-8 text-muted-foreground sm:text-lg">
              {messages.home.description}
            </p>
          </div>
          <Button asChild className="h-11 px-5 text-sm font-medium">
            <Link href="/login">{messages.home.signIn}</Link>
          </Button>
        </section>
        <aside
          className="hidden max-w-md justify-self-end lg:block"
          aria-label={messages.home.supportingText}
        >
          <div className="space-y-8 border-s-2 border-primary/30 ps-7">
            <p className="text-2xl font-medium leading-relaxed tracking-tight text-foreground">
              {messages.home.supportingText}
            </p>
            <div className="flex items-center gap-3 text-sm text-muted-foreground">
              <span aria-hidden="true" className="size-2 rounded-full bg-success" />
              <span>{messages.home.status}</span>
            </div>
          </div>
        </aside>
      </div>
      <footer className="mx-auto flex w-full max-w-7xl items-center justify-between border-t pt-5 text-xs text-muted-foreground">
        <span>{messages.home.footer}</span>
        <span>© Mahfel</span>
      </footer>
    </main>
  );
}
