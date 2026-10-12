import Link from "next/link";
import Image from "next/image";
import { ArrowUpRight, FolderOpen, MessageSquare, PencilLine, Video } from "lucide-react";

import { PartoCompanion } from "@/features/parto/components/parto-companion";
import { KeycloakSignInButton } from "./keycloak-sign-in-button";
import { ThemeSwitcher } from "@/features/appearance/components/theme-switcher";
import type { Theme } from "@/features/appearance/theme";
import { LocaleSwitcher } from "@/features/i18n/components/locale-switcher";
import type { Locale } from "@/lib/i18n/config";
import { messagesFor } from "@/lib/i18n/messages";
import styles from "./landing-page.module.css";

export function LandingPageView({ locale, theme }: Readonly<{ locale: Locale; theme: Theme }>) {
  const copy = messagesFor(locale).home;
  const capabilities = [
    { icon: Video, title: copy.sessionTitle, description: copy.sessionDescription },
    { icon: MessageSquare, title: copy.chatTitle, description: copy.chatDescription },
    { icon: PencilLine, title: copy.ideasTitle, description: copy.ideasDescription },
    { icon: FolderOpen, title: copy.filesTitle, description: copy.filesDescription },
  ];
  return (
    <main
      className={`${styles.page} relative isolate flex min-h-dvh flex-col px-5 sm:px-10 lg:px-14`}
    >
      <a
        href="#landing-content"
        className="absolute start-5 top-3 z-20 -translate-y-24 rounded-lg bg-primary px-4 py-3 text-sm font-medium text-primary-foreground focus:translate-y-0"
      >
        {copy.skipToContent}
      </a>
      <header className="mx-auto flex w-full max-w-6xl items-center justify-between gap-4 py-5 sm:py-7">
        <Link
          className="flex shrink-0 items-center gap-2.5 rounded-md text-xl font-semibold tracking-tight focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
          href="/"
          aria-label={copy.brand}
        >
          <Image alt="" height={36} priority src="/mahfel-mark.svg" width={36} />
          <span>{copy.brand}</span>
        </Link>
        <div className="flex min-w-0 items-center gap-2">
          <ThemeSwitcher locale={locale} theme={theme} compact />
          <LocaleSwitcher locale={locale} compact />
        </div>
      </header>
      <div className="mx-auto flex w-full max-w-6xl flex-1 flex-col justify-center">
        <div className="grid items-center gap-8 pb-9 pt-8 sm:gap-10 sm:pb-12 sm:pt-10 lg:grid-cols-[minmax(0,1.35fr)_minmax(19rem,0.9fr)] lg:gap-10 lg:pb-12 lg:pt-8">
          <section
            id="landing-content"
            tabIndex={-1}
            className={`${styles.enter} relative z-10 min-w-0 max-w-3xl outline-none`}
          >
            <p className="mb-5 flex items-center gap-3 text-sm font-medium text-muted-foreground">
              <span aria-hidden="true" className="h-px w-8 bg-primary/60" />
              {copy.eyebrow}
            </p>
            <h1 className="text-balance text-[clamp(2.125rem,5.1vw,4.75rem)] font-semibold leading-[1.2] rtl:leading-[1.4] tracking-[-0.045em] rtl:text-[clamp(1.875rem,8.8vw,2.5rem)] rtl:tracking-normal lg:rtl:text-[clamp(2.25rem,4vw,3.75rem)]">
              <span className="block">{copy.titleLead}</span>
              <span className="block text-primary transition-colors duration-200 motion-reduce:transition-none">
                {copy.titleAccent}
              </span>
            </h1>
            <p className="mt-5 max-w-[34rem] text-pretty text-base leading-8 text-foreground/75 sm:text-lg sm:leading-9">
              {copy.description}
            </p>
            <div className="mt-7 flex flex-col items-start gap-3 sm:mt-8">
              <KeycloakSignInButton pendingLabel={copy.signingIn} errorMessage={copy.signInFailed}>
                {copy.signIn}
                <ArrowUpRight
                  aria-hidden="true"
                  className="size-4 transition-transform motion-reduce:transition-none group-hover:-translate-y-0.5 group-hover:translate-x-0.5 rtl:-rotate-90 rtl:group-hover:-translate-x-0.5"
                />
              </KeycloakSignInButton>
            </div>
          </section>
          <aside
            className={`${styles.companion} relative mx-auto min-w-0 w-full max-w-[25rem] lg:justify-self-end`}
            aria-label={copy.partoName}
          >
            <div
              aria-hidden="true"
              className={`${styles.halo} pointer-events-none absolute inset-x-0 -bottom-4 top-6 sm:-inset-x-6`}
            />
            <PartoCompanion locale={locale} />
          </aside>
        </div>
        <section
          aria-label={copy.capabilitiesLabel}
          className="border-t border-border/70 py-7 sm:py-9"
        >
          <ul className="grid grid-cols-1 gap-x-6 gap-y-7 min-[360px]:grid-cols-2 lg:grid-cols-4 lg:gap-7">
            {capabilities.map(({ icon: Icon, title, description }) => (
              <li key={title} className="group flex items-start gap-3 sm:gap-3.5">
                <Icon
                  aria-hidden="true"
                  className="mt-0.5 size-5 shrink-0 text-primary/80 transition-transform duration-200 motion-safe:group-hover:-translate-y-0.5"
                  strokeWidth={1.6}
                />
                <div>
                  <h2 className="text-sm font-semibold leading-6">{title}</h2>
                  <p className="mt-1.5 max-w-[15rem] text-[0.8125rem] leading-6 text-muted-foreground">
                    {description}
                  </p>
                </div>
              </li>
            ))}
          </ul>
        </section>
      </div>
      <footer className="mx-auto flex w-full max-w-6xl items-center border-t border-border/60 py-5 text-xs leading-6 text-muted-foreground">
        <span>© {copy.brand}</span>
      </footer>
    </main>
  );
}
