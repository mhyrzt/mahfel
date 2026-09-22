import { useEffect, useState, type ComponentProps, type CSSProperties } from "react";
import type { Meta, StoryObj } from "@storybook/nextjs";

import { ParticipantTile } from "../components/participant-tile";

const participant = {
  id: "participant-1",
  name: "Ritsuko Akagi",
  initials: "RA",
  role: "moderator" as const,
  videoEnabled: false,
};

const meta = {
  title: "Sessions/Participant Tile",
  component: ParticipantTile,
  parameters: { layout: "centered" },
  args: { participant },
  decorators: [
    (Story) => (
      <div
        className="w-[min(90vw,48rem)]"
        style={
          {
            "--stage-grid-rows": 1,
            "--stage-tile-max-width": "min(100%, 48rem)",
          } as CSSProperties
        }
      >
        <Story />
      </div>
    ),
  ],
} satisfies Meta<typeof ParticipantTile>;

export default meta;

type Story = StoryObj<typeof meta>;
type TileArgs = ComponentProps<typeof ParticipantTile>;

export const AvatarOnly: Story = {};

export const CameraOnly: Story = {
  args: { participant: { ...participant, videoEnabled: true } },
  render: (args) => <MockMediaTile args={args} camera />,
};

export const ScreenShareOnly: Story = {
  args: { participant: { ...participant, screenShareEnabled: true } },
  render: (args) => <MockMediaTile args={args} screenShare />,
};

export const ScreenShareAndCamera: Story = {
  args: {
    participant: { ...participant, videoEnabled: true, screenShareEnabled: true },
  },
  render: (args) => <MockMediaTile args={args} camera screenShare />,
};

function MockMediaTile({
  args,
  camera = false,
  screenShare = false,
}: Readonly<{ args: TileArgs; camera?: boolean; screenShare?: boolean }>) {
  const [cameraStream, setCameraStream] = useState<MediaStream>();
  const [screenShareStream, setScreenShareStream] = useState<MediaStream>();

  useEffect(() => {
    const nextCameraStream = camera ? createMockStream("camera") : undefined;
    const nextScreenShareStream = screenShare ? createMockStream("screen") : undefined;
    const timeout = window.setTimeout(() => {
      setCameraStream(nextCameraStream);
      setScreenShareStream(nextScreenShareStream);
    }, 0);

    return () => {
      window.clearTimeout(timeout);
      nextCameraStream?.getTracks().forEach((track) => track.stop());
      nextScreenShareStream?.getTracks().forEach((track) => track.stop());
    };
  }, [camera, screenShare]);

  return (
    <ParticipantTile {...args} cameraStream={cameraStream} screenShareStream={screenShareStream} />
  );
}

function createMockStream(kind: "camera" | "screen"): MediaStream | undefined {
  const canvas = document.createElement("canvas");
  canvas.width = 640;
  canvas.height = 360;
  const context = canvas.getContext("2d");
  if (!context || typeof canvas.captureStream !== "function") {
    return undefined;
  }

  if (kind === "camera") {
    const background = context.createLinearGradient(0, 0, 640, 360);
    background.addColorStop(0, "#1d4ed8");
    background.addColorStop(1, "#0f172a");
    context.fillStyle = background;
    context.fillRect(0, 0, 640, 360);
    context.fillStyle = "#f2c6a0";
    context.beginPath();
    context.arc(320, 135, 62, 0, Math.PI * 2);
    context.fill();
    context.fillStyle = "#312e81";
    context.beginPath();
    context.ellipse(320, 360, 145, 150, 0, Math.PI, Math.PI * 2);
    context.fill();
  } else {
    context.fillStyle = "#e2e8f0";
    context.fillRect(0, 0, 640, 360);
    context.fillStyle = "#4f46e5";
    context.fillRect(0, 0, 640, 28);
    context.fillStyle = "#0f172a";
    context.font = "600 28px sans-serif";
    context.fillText("Research seminar", 48, 100);
    context.fillStyle = "#94a3b8";
    context.fillRect(48, 130, 420, 12);
    context.fillRect(48, 158, 360, 12);
    context.fillRect(48, 186, 390, 12);
    context.fillStyle = "#c7d2fe";
    context.fillRect(48, 230, 220, 92);
  }

  return canvas.captureStream(12);
}
