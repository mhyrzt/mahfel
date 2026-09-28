"use client";

import { useEffect, useState, type ReactNode } from "react";
import { Presentation, Square } from "lucide-react";

import { Button } from "@/components/ui/button";
import {
  Carousel,
  CarouselContent,
  CarouselItem,
  CarouselNext,
  CarouselPrevious,
  type CarouselApi,
} from "@/components/ui/carousel";
import type { Locale } from "@/lib/i18n/config";
import { messagesFor } from "@/lib/i18n/messages";

export function PresentationViewer({
  currentPage,
  totalPages,
  canPresent,
  live,
  locale,
  failed = false,
  renderPage,
  onPageChange,
  onStop,
}: Readonly<{
  currentPage: number;
  totalPages: number;
  canPresent: boolean;
  live: boolean;
  locale: Locale;
  failed?: boolean;
  renderPage: (page: number) => ReactNode;
  onPageChange: (page: number) => void;
  onStop: () => void;
}>) {
  const text = messagesFor(locale).collaboration;
  const [api, setApi] = useState<CarouselApi>();
  const interactive = canPresent && live;
  const direction = locale === "fa" ? "rtl" : "ltr";

  useEffect(() => {
    if (!api || !totalPages) {
      return undefined;
    }
    const page = Math.max(0, currentPage - 1);
    if (api.selectedScrollSnap() !== page) {
      api.scrollTo(page, true);
    }
    if (!interactive) {
      return undefined;
    }
    const onSelect = () => {
      const selected = api.selectedScrollSnap() + 1;
      if (selected !== currentPage) {
        onPageChange(selected);
      }
    };
    api.on("select", onSelect);
    return () => {
      api.off("select", onSelect);
    };
  }, [api, currentPage, interactive, onPageChange, totalPages]);

  return (
    <div className="flex h-full min-h-0 flex-1 flex-col gap-3 overflow-hidden rounded-xl border bg-background p-3">
      {failed ? (
        <p role="alert" className="text-sm text-destructive">
          {text.unavailable}
        </p>
      ) : null}
      {totalPages > 0 ? (
        <>
          <Carousel
            aria-label={text.presentation}
            className="min-h-0 flex-1"
            dir={direction}
            interactive={interactive}
            opts={{ direction, watchDrag: interactive }}
            setApi={setApi}
          >
            <CarouselContent>
              {Array.from({ length: totalPages }, (_, index) => (
                <CarouselItem
                  aria-label={`${index + 1} / ${totalPages}`}
                  className="h-full"
                  key={index}
                >
                  <div className="flex h-full min-h-0 items-center justify-center overflow-auto rounded-lg bg-muted/50 p-3">
                    {renderPage(index + 1)}
                  </div>
                </CarouselItem>
              ))}
            </CarouselContent>
            {interactive && totalPages > 1 ? (
              <>
                <CarouselPrevious aria-label={text.previous} />
                <CarouselNext aria-label={text.next} />
              </>
            ) : null}
          </Carousel>
          <div className="flex items-center justify-between gap-3 border-t pt-3">
            <span
              aria-live="polite"
              className="text-xs font-medium tabular-nums text-muted-foreground"
            >
              {currentPage} / {totalPages}
            </span>
            {interactive ? (
              <Button size="sm" variant="outline" onClick={onStop}>
                <Square aria-hidden="true" className="size-3 fill-current" />
                {text.stop}
              </Button>
            ) : null}
          </div>
        </>
      ) : (
        <div className="flex min-h-0 flex-1 flex-col items-center justify-center gap-3 px-6 text-center">
          <span className="rounded-2xl bg-primary/10 p-4 text-primary">
            <Presentation aria-hidden="true" className="size-7" />
          </span>
          <p className="text-xl font-semibold tracking-tight sm:text-2xl">{text.noPresentation}</p>
          {interactive ? (
            <p className="max-w-sm text-sm text-muted-foreground">{text.selectFiles}</p>
          ) : null}
        </div>
      )}
    </div>
  );
}
