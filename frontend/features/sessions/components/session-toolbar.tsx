import {
  Presentation,
  Mic,
  MicOff,
  MonitorUp,
  NotebookPen,
  PhoneOff,
  UsersRound,
  Video,
  VideoOff,
  PencilRuler,
} from "lucide-react";
import type { ReactNode } from "react";

import { Button } from "@/components/ui/button";
import { defaultLocale, type Locale } from "@/lib/i18n/config";
import { messagesFor } from "@/lib/i18n/messages";
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "@/components/ui/tooltip";
import { roomCopy } from "@/features/rooms/room-copy";

export interface SessionToolbarCapabilities {
  canPublishAudio: boolean;
  canPublishVideo: boolean;
  canShareScreen: boolean;
}

interface SessionToolbarProps extends Partial<SessionToolbarCapabilities> {
  activeSurface?: "participants" | "notes" | "whiteboard" | "presentation";
  onSelectSurface?: (surface: "participants" | "notes" | "whiteboard" | "presentation") => void;
  cameraEnabled?: boolean;
  microphoneEnabled?: boolean;
  screenShareEnabled?: boolean;
  mediaBusy?: boolean;
  locale?: Locale;
  onLeave?: () => void;
  onToggleMicrophone?: () => void;
  onToggleCamera?: () => void;
  onToggleScreenShare?: () => void;
  endAction?: ReactNode;
}

export function SessionToolbar({
  activeSurface = "participants",
  onSelectSurface,
  canPublishAudio = true,
  canPublishVideo = true,
  canShareScreen = true,
  cameraEnabled = false,
  microphoneEnabled = false,
  screenShareEnabled = false,
  mediaBusy = false,
  locale = defaultLocale,
  onLeave,
  onToggleMicrophone,
  onToggleCamera,
  onToggleScreenShare,
  endAction,
}: Readonly<SessionToolbarProps>) {
  const text = messagesFor(locale).sessions;
  const collaboration = messagesFor(locale).collaboration;
  const unavailable = roomCopy(locale).mediaUnavailable;
  const controls = [
    {
      available: canPublishAudio,
      label: microphoneEnabled ? text.muteMicrophone : text.unmuteMicrophone,
      icon: microphoneEnabled ? Mic : MicOff,
      pressed: microphoneEnabled,
      onClick: onToggleMicrophone,
    },
    {
      available: canPublishVideo,
      label: cameraEnabled ? text.turnCameraOff : text.turnCameraOn,
      icon: cameraEnabled ? Video : VideoOff,
      pressed: cameraEnabled,
      onClick: onToggleCamera,
    },
    {
      available: canShareScreen,
      label: text.shareScreen,
      icon: MonitorUp,
      pressed: screenShareEnabled,
      onClick: onToggleScreenShare,
    },
  ].filter(({ available }) => available);

  return (
    <TooltipProvider>
      <div className="flex w-fit max-w-[calc(100vw-2rem)] items-center gap-1 overflow-x-auto rounded-2xl border border-slate-800 bg-slate-950 p-1.5 shadow-[0_12px_32px_-12px_rgba(15,23,42,0.65)]">
        {controls.map(({ label, icon: Icon, pressed, onClick }) => (
          <Tooltip key={label}>
            <TooltipTrigger asChild>
              <Button
                aria-label={label}
                aria-pressed={pressed}
                disabled={!onClick || mediaBusy}
                onClick={onClick}
                className={`size-10 rounded-xl transition-all duration-200 active:scale-95 ${
                  pressed
                    ? "bg-slate-800 text-white hover:bg-slate-700"
                    : "text-slate-400 hover:bg-slate-800 hover:text-white"
                }`}
                size="icon"
                variant="ghost"
              >
                <Icon className="size-[1.125rem]" strokeWidth={1.8} />
              </Button>
            </TooltipTrigger>
            <TooltipContent side="top" sideOffset={10}>
              {onClick ? label : unavailable}
            </TooltipContent>
          </Tooltip>
        ))}
        <div className="mx-1 h-6 w-px bg-slate-800" />
        {(
          [
            { surface: "participants", label: text.participants, icon: UsersRound },
            { surface: "notes", label: collaboration.notes, icon: NotebookPen },
            { surface: "whiteboard", label: collaboration.whiteboard, icon: PencilRuler },
            { surface: "presentation", label: collaboration.presentation, icon: Presentation },
          ] as const
        ).map(({ surface, label, icon: Icon }) => (
          <Tooltip key={surface}>
            <TooltipTrigger asChild>
              <Button
                aria-label={label}
                aria-pressed={activeSurface === surface}
                className={`size-10 rounded-xl ${activeSurface === surface ? "bg-slate-800 text-white" : "text-slate-400 hover:bg-slate-800 hover:text-white"}`}
                onClick={() => onSelectSurface?.(surface)}
                size="icon"
                variant="ghost"
              >
                <Icon className="size-[1.125rem]" />
              </Button>
            </TooltipTrigger>
            <TooltipContent side="top">{label}</TooltipContent>
          </Tooltip>
        ))}
        <div className="mx-1 h-6 w-px bg-slate-800" />
        <Tooltip>
          <TooltipTrigger asChild>
            <Button
              aria-label={text.leaveSession}
              data-testid="leave-session"
              onClick={onLeave}
              className="size-10 rounded-xl bg-rose-500 text-white transition-all duration-200 hover:bg-rose-400 active:scale-95"
              size="icon"
              variant="destructive"
            >
              <PhoneOff className="size-[1.125rem]" strokeWidth={1.8} />
            </Button>
          </TooltipTrigger>
          <TooltipContent side="top" sideOffset={10}>
            {text.leaveSession}
          </TooltipContent>
        </Tooltip>
        {endAction ? (
          <>
            <div className="mx-1 h-6 w-px bg-slate-800" />
            {endAction}
          </>
        ) : null}
      </div>
    </TooltipProvider>
  );
}
