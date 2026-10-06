import { Hand } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import { cn } from "@/lib/utils";
import { messagesFor } from "@/lib/i18n/messages";
import type { Locale } from "@/lib/i18n/config";

export function HandControl({
  raised,
  disabled,
  locale,
  onToggle,
}: Readonly<{ raised: boolean; disabled: boolean; locale: Locale; onToggle: () => void }>) {
  const text = messagesFor(locale).interactions;
  const label = raised ? text.lowerHand : text.raiseHand;
  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <Button
          size="icon"
          variant="ghost"
          disabled={disabled}
          aria-label={label}
          aria-pressed={raised}
          onClick={onToggle}
          className={cn(
            "size-10 shrink-0 rounded-xl text-slate-400 hover:bg-slate-800 hover:text-white",
            raised && "bg-warning/15 text-warning hover:text-warning",
          )}
        >
          <Hand className="size-[1.125rem]" aria-hidden="true" />
        </Button>
      </TooltipTrigger>
      <TooltipContent dir={locale === "fa" ? "rtl" : "ltr"} side="top">
        {label}
      </TooltipContent>
    </Tooltip>
  );
}
