import Link from "next/link";
import Image from "next/image";
import { ArrowUpRight } from "lucide-react";

import { KeycloakSignInButton } from "./keycloak-sign-in-button";
import { ThemeSwitcher } from "@/features/appearance/components/theme-switcher";
import type { Theme } from "@/features/appearance/theme";
import { LocaleSwitcher } from "@/features/i18n/components/locale-switcher";
import type { Locale } from "@/lib/i18n/config";
import { messagesFor } from "@/lib/i18n/messages";

export function LandingPageView({ locale, theme }: Readonly<{ locale: Locale; theme: Theme }>) {
  const messages = messagesFor(locale);
  return (
    <main className="relative isolate flex min-h-dvh flex-col overflow-hidden bg-background px-5 py-5 sm:px-10 sm:py-7">
      <div
        aria-hidden="true"
        className="pointer-events-none absolute inset-0 -z-10 bg-[radial-gradient(circle_at_78%_38%,color-mix(in_srgb,var(--primary)_12%,transparent),transparent_34%)]"
      />
      <header className="mx-auto flex w-full max-w-7xl items-center justify-between gap-4 border-b border-border/60 pb-5">
        <Link
          className="flex shrink-0 items-center gap-2 text-lg font-semibold tracking-tight"
          href="/"
        >
          <Image alt="" height={36} priority src="/mahfel-mark.svg" width={36} />
          <span className="hidden sm:inline">Mahfel</span>
        </Link>
        <div className="flex min-w-0 items-center gap-1.5">
          <ThemeSwitcher locale={locale} theme={theme} compact />
          <LocaleSwitcher locale={locale} compact />
        </div>
      </header>
      <div className="mx-auto grid w-full max-w-7xl flex-1 items-center gap-12 py-16 lg:grid-cols-[minmax(0,1.2fr)_minmax(20rem,0.8fr)] lg:gap-16 lg:py-24">
        <section className="max-w-3xl space-y-9">
          <p className="w-fit border-b-2 border-primary pb-2 text-xs font-semibold uppercase tracking-[0.16em] text-primary">
            {messages.home.eyebrow}
          </p>
          <div className="space-y-6">
            <h1 className="max-w-3xl text-balance text-4xl font-semibold leading-[1.12] tracking-[-0.045em] sm:text-7xl sm:leading-[1.08]">
              {messages.home.title}
            </h1>
            <p className="max-w-xl text-base leading-8 text-muted-foreground sm:text-lg">
              {messages.home.description}
            </p>
          </div>
          <KeycloakSignInButton>
            {messages.home.signIn}
            <ArrowUpRight aria-hidden="true" className="size-4 rtl:-rotate-90" />
          </KeycloakSignInButton>
        </section>
        <aside
          className="relative mx-auto flex aspect-square w-full max-w-[30rem] items-center justify-center"
          aria-label={messages.home.supportingText}
        >
          <div
            aria-hidden="true"
            className="absolute inset-[6%] rounded-full border border-primary/20"
          />
          <div
            aria-hidden="true"
            className="absolute inset-[18%] rounded-full border border-primary/15"
          />
          <div className="relative flex size-[58%] items-center justify-center rounded-[2.5rem] border border-primary/15 bg-card/80 shadow-[0_32px_90px_-45px_color-mix(in_srgb,var(--primary)_55%,transparent)] backdrop-blur-sm">
            <Image
              alt=""
              className="h-auto w-[56%]"
              height={180}
              src="/mahfel-mark.svg"
              width={180}
            />
          </div>
          <div className="absolute bottom-[4%] start-[2%] max-w-[14rem] rounded-2xl border bg-card/95 p-4 shadow-lg backdrop-blur-sm">
            <span className="mb-2 block size-2 rounded-full bg-success" aria-hidden="true" />
            <p className="text-sm font-medium leading-6">{messages.home.supportingText}</p>
            <p className="mt-2 text-xs text-muted-foreground">{messages.home.status}</p>
          </div>
        </aside>
      </div>
      <footer className="mx-auto flex w-full max-w-7xl items-center justify-between gap-4 border-t border-border/60 pt-5 text-xs text-muted-foreground">
        <span>{messages.home.footer}</span>
        <span>© Mahfel</span>
      </footer>
    </main>
  );
}
