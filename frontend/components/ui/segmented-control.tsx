"use client";

import type { LucideIcon } from "lucide-react";

import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";

export interface SegmentOption<Value extends string> {
  value: Value;
  label: string;
  icon?: LucideIcon;
}

export function SegmentedControl<Value extends string>({
  label,
  value,
  options,
  onValueChange,
  className,
}: Readonly<{
  label: string;
  value: Value;
  options: readonly SegmentOption<Value>[];
  onValueChange: (value: Value) => void;
  className?: string;
}>) {
  return (
    <div
      aria-label={label}
      className={cn(
        "grid w-full grid-flow-col auto-cols-fr gap-1 rounded-xl border bg-muted/40 p-1",
        className,
      )}
      role="group"
    >
      {options.map((option) => {
        const Icon = option.icon;
        const selected = value === option.value;
        return (
          <Button
            aria-pressed={selected}
            className={cn(
              "h-9 min-w-0 gap-1.5 rounded-lg px-2 text-xs font-medium shadow-none sm:text-sm",
              selected
                ? "bg-background text-foreground shadow-sm ring-1 ring-border/80 hover:bg-background"
                : "text-muted-foreground hover:bg-background/60 hover:text-foreground",
            )}
            key={option.value}
            onClick={() => onValueChange(option.value)}
            type="button"
            variant="ghost"
          >
            {Icon ? <Icon aria-hidden="true" className="size-4 shrink-0" /> : null}
            <span className="truncate">{option.label}</span>
          </Button>
        );
      })}
    </div>
  );
}
