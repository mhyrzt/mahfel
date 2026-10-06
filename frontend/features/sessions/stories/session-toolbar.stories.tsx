import { useState, type ComponentProps } from "react";
import type { Meta, StoryObj } from "@storybook/nextjs";
import { Button } from "@/components/ui/button";
import { SessionToolbar } from "../components/session-toolbar";
import { useInteractionPreview } from "./use-interaction-preview";

export function InteractiveToolbarPreview({
  disabled = false,
  initialRaised = false,
  mediaUnavailable = false,
  ...args
}: ComponentProps<typeof SessionToolbar> & {
  disabled?: boolean;
  initialRaised?: boolean;
  mediaUnavailable?: boolean;
}) {
  const locale = args.locale ?? "en";
  const interactions = useInteractionPreview(locale, disabled, initialRaised);
  const [microphone, setMicrophone] = useState(args.microphoneEnabled ?? false);
  const [camera, setCamera] = useState(args.cameraEnabled ?? false);
  const [share, setShare] = useState(args.screenShareEnabled ?? false);
  const [surface, setSurface] = useState(args.activeSurface ?? "participants");
  return (
    <div
      className="relative flex h-[min(28rem,70dvh)] min-h-60 items-end justify-center overflow-hidden rounded-xl bg-muted/40 p-4"
      dir={locale === "fa" ? "rtl" : "ltr"}
    >
      {interactions.reactionOverlay}
      <SessionToolbar
        {...args}
        activeSurface={surface}
        onSelectSurface={setSurface}
        microphoneEnabled={microphone}
        cameraEnabled={camera}
        screenShareEnabled={share}
        onToggleMicrophone={
          disabled || mediaUnavailable ? undefined : () => setMicrophone((value) => !value)
        }
        onToggleCamera={
          disabled || mediaUnavailable ? undefined : () => setCamera((value) => !value)
        }
        onToggleScreenShare={
          disabled || mediaUnavailable ? undefined : () => setShare((value) => !value)
        }
        handControl={interactions.handControl}
        reactionControl={interactions.reactionControl}
      />
    </div>
  );
}

const meta = {
  title: "Sessions/Toolbar",
  component: SessionToolbar,
  render: (args) => <InteractiveToolbarPreview {...args} />,
} satisfies Meta<typeof SessionToolbar>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Interactive: Story = {};
export const HandRaised: Story = {
  render: (args) => <InteractiveToolbarPreview {...args} initialRaised />,
};
export const MediaActive: Story = {
  args: { microphoneEnabled: true, cameraEnabled: true, screenShareEnabled: true },
};
export const MediaUnavailable: Story = {
  render: (args) => <InteractiveToolbarPreview {...args} mediaUnavailable />,
};
export const InteractionsUnavailable: Story = {
  render: (args) => <InteractiveToolbarPreview {...args} disabled />,
};
export const InteractionsLoading: Story = {
  render: (args) => <InteractiveToolbarPreview {...args} disabled />,
  args: { mediaBusy: true },
};
export const ListenOnly: Story = {
  args: { canPublishAudio: false, canPublishVideo: false, canShareScreen: false },
};
export const PollView: Story = { args: { activeSurface: "activities" } };
export const Host: Story = {
  args: {
    endAction: (
      <Button size="sm" variant="destructive">
        End session
      </Button>
    ),
  },
};
export const Persian: Story = { args: { locale: "fa" } };
export const NarrowPersian: Story = {
  args: { locale: "fa" },
  decorators: [
    (Story) => (
      <div className="max-w-sm">
        <Story />
      </div>
    ),
  ],
};
