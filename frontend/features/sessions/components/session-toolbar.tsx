import { Mic, MicOff, MonitorUp, PhoneOff, Video, VideoOff } from "lucide-react";

import { Button } from "@/components/ui/button";
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "@/components/ui/tooltip";

interface SessionToolbarProps {
  cameraEnabled?: boolean;
  microphoneEnabled?: boolean;
  screenShareEnabled?: boolean;
}

export function SessionToolbar({
  cameraEnabled = true,
  microphoneEnabled = true,
  screenShareEnabled = false,
}: Readonly<SessionToolbarProps>) {
  const controls = [
    {
      label: microphoneEnabled ? "Mute microphone" : "Unmute microphone",
      icon: microphoneEnabled ? Mic : MicOff,
      pressed: microphoneEnabled,
    },
    {
      label: cameraEnabled ? "Turn camera off" : "Turn camera on",
      icon: cameraEnabled ? Video : VideoOff,
      pressed: cameraEnabled,
    },
    { label: "Share screen", icon: MonitorUp, pressed: screenShareEnabled },
  ];

  return (
    <TooltipProvider>
      <div className="flex w-fit items-center gap-1 rounded-2xl border border-slate-800 bg-slate-950 p-1.5 shadow-[0_12px_32px_-12px_rgba(15,23,42,0.65)]">
        {controls.map(({ label, icon: Icon, pressed }) => (
          <Tooltip key={label}>
            <TooltipTrigger asChild>
              <Button
                aria-label={label}
                aria-pressed={pressed}
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
              {label}
            </TooltipContent>
          </Tooltip>
        ))}
        <div className="mx-1 h-6 w-px bg-slate-800" />
        <Tooltip>
          <TooltipTrigger asChild>
            <Button
              aria-label="Leave session"
              className="size-10 rounded-xl bg-rose-500 text-white transition-all duration-200 hover:bg-rose-400 active:scale-95"
              size="icon"
              variant="destructive"
            >
              <PhoneOff className="size-[1.125rem]" strokeWidth={1.8} />
            </Button>
          </TooltipTrigger>
          <TooltipContent side="top" sideOffset={10}>
            Leave session
          </TooltipContent>
        </Tooltip>
      </div>
    </TooltipProvider>
  );
}
