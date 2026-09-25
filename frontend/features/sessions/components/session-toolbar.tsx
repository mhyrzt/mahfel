import { Mic, MicOff, MonitorUp, PhoneOff, Video, VideoOff } from "lucide-react";

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
  cameraEnabled?: boolean;
  microphoneEnabled?: boolean;
  screenShareEnabled?: boolean;
  locale?: Locale;
  onLeave?: () => void;
  onToggleMicrophone?: () => void;
  onToggleCamera?: () => void;
  onToggleScreenShare?: () => void;
}

export function SessionToolbar({
  canPublishAudio = true,
  canPublishVideo = true,
  canShareScreen = true,
  cameraEnabled = false,
  microphoneEnabled = false,
  screenShareEnabled = false,
  locale = defaultLocale,
  onLeave,
  onToggleMicrophone,
  onToggleCamera,
  onToggleScreenShare,
}: Readonly<SessionToolbarProps>) {
  const text = messagesFor(locale).sessions;
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
      <div className="flex w-fit items-center gap-1 rounded-2xl border border-slate-800 bg-slate-950 p-1.5 shadow-[0_12px_32px_-12px_rgba(15,23,42,0.65)]">
        {controls.map(({ label, icon: Icon, pressed, onClick }) => (
          <Tooltip key={label}>
            <TooltipTrigger asChild>
              <Button
                aria-label={label}
                aria-pressed={pressed}
                disabled={!onClick}
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
      </div>
    </TooltipProvider>
  );
}
