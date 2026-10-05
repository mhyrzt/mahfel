"use client";

import type { LucideIcon } from "lucide-react";

import { RadioGroup, RadioGroupItem } from "@/components/ui/radio-group";
import { Label } from "@/components/ui/label";
import { useId } from "react";
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
  dir,
  disabled = false,
}: Readonly<{
  label: string;
  value: Value;
  options: readonly SegmentOption<Value>[];
  onValueChange: (value: Value) => void;
  className?: string;
  dir?: "ltr" | "rtl";
  disabled?: boolean;
}>) {
  const groupId = useId();
  return (
    <RadioGroup
      dir={dir}
      disabled={disabled}
      aria-label={label}
      className={cn(
        "grid w-full grid-flow-col auto-cols-fr gap-1 rounded-xl border bg-muted/40 p-1",
        className,
      )}
      orientation="horizontal"
      value={value}
      onValueChange={(nextValue) => {
        const option = options.find((item) => item.value === nextValue);
        if (option) {
          onValueChange(option.value);
        }
      }}
    >
      {options.map((option) => {
        const Icon = option.icon;
        const selected = value === option.value;
        return (
          <Label
            htmlFor={`${groupId}-${option.value}`}
            className={cn(
              "relative flex h-9 min-w-0 cursor-pointer items-center justify-center gap-1.5 rounded-lg px-2 text-xs font-medium shadow-none has-[:focus-visible]:ring-2 has-[:focus-visible]:ring-ring has-[:disabled]:cursor-not-allowed has-[:disabled]:opacity-50 sm:text-sm",
              selected
                ? "bg-background text-foreground shadow-sm ring-1 ring-border/80 hover:bg-background"
                : "text-muted-foreground hover:bg-background/60 hover:text-foreground",
            )}
            key={option.value}
          >
            <RadioGroupItem
              className="sr-only"
              id={`${groupId}-${option.value}`}
              value={option.value}
            />
            {Icon ? <Icon aria-hidden="true" className="size-4 shrink-0" /> : null}
            <span className="truncate">{option.label}</span>
          </Label>
        );
      })}
    </RadioGroup>
  );
}
