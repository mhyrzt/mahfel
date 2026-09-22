import type { Meta, StoryObj } from "@storybook/nextjs";

import { initialSessionState, sessionReducer } from "../session-state";
import { SessionScreen } from "../components/session-screen";

const capabilities = {
  canPublishAudio: true,
  canPublishVideo: true,
  canShareScreen: true,
  canModerate: true,
};

const participants = [
  {
    id: "participant-1",
    name: "Ritsuko Akagi",
    initials: "RA",
    role: "moderator" as const,
    speaking: true,
    videoEnabled: true,
  },
  {
    id: "participant-2",
    name: "Asuka Langley",
    initials: "AL",
    role: "member" as const,
    muted: true,
    videoEnabled: false,
  },
];

const liveState = sessionReducer(initialSessionState, {
  type: "session-connected",
  sessionId: "session-demo",
  participantId: "participant-1",
  roomName: "Research methods",
  participants,
  capabilities,
});

const waitingState = sessionReducer(initialSessionState, {
  type: "admission-pending",
  roomName: "Research methods",
});

const endedState = sessionReducer(liveState, { type: "session-ended" });
const rejectedState = sessionReducer(initialSessionState, {
  type: "admission-rejected",
  message: "The host declined the request.",
});
const unavailableState = sessionReducer(liveState, {
  type: "connection-unavailable",
  message: "Connection lost.",
});

const meta = {
  title: "Sessions/Session Screen",
  component: SessionScreen,
  args: {
    title: "Weekly research discussion",
    state: liveState,
    admissionRequests: [
      { id: "request-1", name: "Misato Katsuragi", initials: "MK", requestedAt: "just now" },
    ],
  },
} satisfies Meta<typeof SessionScreen>;

export default meta;

type Story = StoryObj<typeof meta>;

export const EnglishLive: Story = {};

export const PersianLive: Story = {
  args: { locale: "fa" },
  decorators: [
    (Story) => (
      <div dir="rtl" lang="fa">
        <Story />
      </div>
    ),
  ],
};

export const Loading: Story = {
  args: { state: initialSessionState, admissionRequests: [] },
};

export const Waiting: Story = {
  args: { state: waitingState, admissionRequests: [] },
};

export const WaitingPersian: Story = {
  args: { state: waitingState, admissionRequests: [], locale: "fa" },
  decorators: [
    (Story) => (
      <div dir="rtl" lang="fa">
        <Story />
      </div>
    ),
  ],
};

export const Ended: Story = {
  args: { state: endedState, admissionRequests: [] },
};

export const Rejected: Story = {
  args: { state: rejectedState, admissionRequests: [] },
};

export const Unavailable: Story = {
  args: { state: unavailableState, admissionRequests: [] },
};
