import Link from "next/link";

import { Button } from "@/components/ui/button";
import { getLocale } from "@/lib/i18n/server";

export default async function NotFound() {
  const locale = await getLocale();
  const text =
    locale === "fa"
      ? {
          title: "این صفحه پیدا نشد",
          detail: "ممکن است حذف شده باشد یا به آن دسترسی نداشته باشید.",
          back: "رفتن به فضای کاری",
        }
      : {
          title: "Page not found",
          detail: "It may have been removed or you may not have access.",
          back: "Go to workspace",
        };
  return (
    <section
      className="mx-auto flex min-h-80 max-w-xl flex-col justify-center gap-5 py-12"
      data-testid="workspace-not-found"
    >
      <div className="space-y-2">
        <h1 className="text-2xl font-semibold tracking-tight">{text.title}</h1>
        <p className="text-sm leading-6 text-muted-foreground">{text.detail}</p>
      </div>
      <Button asChild className="w-fit">
        <Link href="/workspace">{text.back}</Link>
      </Button>
    </section>
  );
}
