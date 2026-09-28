"use client";

import * as React from "react";
import useEmblaCarousel, { type UseEmblaCarouselType } from "embla-carousel-react";
import { ArrowLeft, ArrowRight } from "lucide-react";

import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";

type CarouselApi = UseEmblaCarouselType[1];
type CarouselOptions = Parameters<typeof useEmblaCarousel>[0];

interface CarouselContextValue {
  api: CarouselApi;
  ref: ReturnType<typeof useEmblaCarousel>[0];
  scrollPrev: () => void;
  scrollNext: () => void;
  canScrollPrev: boolean;
  canScrollNext: boolean;
}

const CarouselContext = React.createContext<CarouselContextValue | null>(null);

function useCarousel() {
  const context = React.useContext(CarouselContext);
  if (!context) {
    throw new Error("Carousel controls must be used inside Carousel");
  }
  return context;
}

function Carousel({
  opts,
  setApi,
  interactive = true,
  className,
  children,
  ...props
}: React.ComponentProps<"div"> & {
  opts?: CarouselOptions;
  setApi?: (api: CarouselApi) => void;
  interactive?: boolean;
}) {
  const [ref, api] = useEmblaCarousel({ ...opts, ...(!interactive ? { watchDrag: false } : {}) });
  const [canScrollPrev, setCanScrollPrev] = React.useState(false);
  const [canScrollNext, setCanScrollNext] = React.useState(false);

  React.useEffect(() => {
    if (!api) {
      return undefined;
    }
    setApi?.(api);
    const update = () => {
      setCanScrollPrev(api.canScrollPrev());
      setCanScrollNext(api.canScrollNext());
    };
    update();
    api.on("select", update);
    api.on("reInit", update);
    return () => {
      api.off("select", update);
      api.off("reInit", update);
    };
  }, [api, setApi]);

  return (
    <CarouselContext.Provider
      value={{
        api,
        ref,
        scrollPrev: () => api?.scrollPrev(),
        scrollNext: () => api?.scrollNext(),
        canScrollPrev,
        canScrollNext,
      }}
    >
      <div
        aria-roledescription="carousel"
        className={cn("relative", className)}
        onKeyDownCapture={(event) => {
          if (!interactive) {
            return;
          }
          if (event.key === "ArrowLeft") {
            event.preventDefault();
            if (props.dir === "rtl") {
              api?.scrollNext();
            } else {
              api?.scrollPrev();
            }
          } else if (event.key === "ArrowRight") {
            event.preventDefault();
            if (props.dir === "rtl") {
              api?.scrollPrev();
            } else {
              api?.scrollNext();
            }
          }
        }}
        role="region"
        {...props}
      >
        {children}
      </div>
    </CarouselContext.Provider>
  );
}

function CarouselContent({ className, ...props }: React.ComponentProps<"div">) {
  const { ref } = useCarousel();
  return (
    <div className="h-full overflow-hidden" ref={ref}>
      <div className={cn("flex h-full -ms-4", className)} {...props} />
    </div>
  );
}

function CarouselItem({ className, ...props }: React.ComponentProps<"div">) {
  return (
    <div
      aria-roledescription="slide"
      className={cn("min-w-0 shrink-0 grow-0 basis-full ps-4", className)}
      role="group"
      {...props}
    />
  );
}

function CarouselPrevious({ className, ...props }: React.ComponentProps<typeof Button>) {
  const { canScrollPrev, scrollPrev } = useCarousel();
  return (
    <Button
      aria-label="Previous slide"
      className={cn("absolute start-2 top-1/2 -translate-y-1/2 rounded-full", className)}
      disabled={!canScrollPrev}
      onClick={scrollPrev}
      size="icon"
      type="button"
      variant="secondary"
      {...props}
    >
      <ArrowLeft aria-hidden="true" className="size-4 rtl:rotate-180" />
    </Button>
  );
}

function CarouselNext({ className, ...props }: React.ComponentProps<typeof Button>) {
  const { canScrollNext, scrollNext } = useCarousel();
  return (
    <Button
      aria-label="Next slide"
      className={cn("absolute end-2 top-1/2 -translate-y-1/2 rounded-full", className)}
      disabled={!canScrollNext}
      onClick={scrollNext}
      size="icon"
      type="button"
      variant="secondary"
      {...props}
    >
      <ArrowRight aria-hidden="true" className="size-4 rtl:rotate-180" />
    </Button>
  );
}

export {
  Carousel,
  CarouselContent,
  CarouselItem,
  CarouselPrevious,
  CarouselNext,
  type CarouselApi,
};
