"use client";

import { Hand, SmilePlus } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import type { Locale } from "@/lib/i18n/config";
import { messagesFor } from "@/lib/i18n/messages";

export const reactionPalette = [
  { emoji: "👍", label: "like" },
  { emoji: "👎", label: "dislike" },
  { emoji: "👏", label: "applause" },
  { emoji: "❤️", label: "love" },
  { emoji: "😂", label: "laugh" },
  { emoji: "😮", label: "surprise" },
  { emoji: "🎉", label: "celebrate" },
  { emoji: "🔥", label: "fire" },
] as const;

export function ReactionControl({
  locale,
  disabled,
  handRaised,
  onReact,
  onToggleHand,
}: Readonly<{
  locale: Locale;
  disabled: boolean;
  handRaised: boolean;
  onReact: (emoji: string) => void;
  onToggleHand: () => void;
}>) {
  const text = messagesFor(locale).interactions;
  return (
    <DropdownMenu dir={locale === "fa" ? "rtl" : "ltr"}>
      <Tooltip>
        <TooltipTrigger asChild>
          <DropdownMenuTrigger asChild>
            <Button
              size="icon"
              variant="ghost"
              disabled={disabled}
              aria-label={text.reactions}
              className="size-10 shrink-0 rounded-xl text-slate-400 hover:bg-slate-800 hover:text-white"
            >
              <SmilePlus className="size-[1.125rem]" aria-hidden="true" />
            </Button>
          </DropdownMenuTrigger>
        </TooltipTrigger>
        <TooltipContent side="top">{text.reactions}</TooltipContent>
      </Tooltip>
      <DropdownMenuContent side="top" align="center" sideOffset={12} className="w-56">
        <DropdownMenuLabel>{text.reactions}</DropdownMenuLabel>
        {reactionPalette.map(({ emoji, label }) => (
          <DropdownMenuItem key={emoji} onSelect={() => onReact(emoji)} className="min-h-10">
            <span className="text-xl" aria-hidden="true">
              {emoji}
            </span>
            {text[label]}
          </DropdownMenuItem>
        ))}
        <DropdownMenuSeparator />
        <DropdownMenuItem onSelect={onToggleHand} className="min-h-10">
          <Hand aria-hidden="true" />
          {handRaised ? text.lowerHand : text.raiseHand}
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
