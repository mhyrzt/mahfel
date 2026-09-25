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
    name: "Yuji Itadori",
    initials: "YI",
    role: "moderator" as const,
    speaking: true,
    videoEnabled: true,
  },
  {
    id: "participant-2",
    name: "Megumi Fushiguro",
    initials: "MF",
    role: "member" as const,
    muted: true,
    videoEnabled: false,
  },
  { id: "participant-3", name: "Nobara Kugisaki", initials: "NK", role: "member" as const },
  { id: "participant-4", name: "Satoru Gojo", initials: "SG", role: "moderator" as const },
  { id: "participant-5", name: "Maki Zenin", initials: "MZ", role: "member" as const },
  {
    id: "participant-6",
    name: "Toge Inumaki",
    initials: "TI",
    role: "member" as const,
    muted: true,
  },
  { id: "participant-7", name: "Monkey D. Luffy", initials: "ML", role: "member" as const },
  {
    id: "participant-8",
    name: "Roronoa Zoro",
    initials: "RZ",
    role: "member" as const,
    muted: true,
  },
  { id: "participant-9", name: "Nami", initials: "NA", role: "member" as const },
  { id: "participant-10", name: "Sanji", initials: "SA", role: "member" as const },
  {
    id: "participant-11",
    name: "Naruto Uzumaki",
    initials: "NU",
    role: "member" as const,
    speaking: true,
  },
  {
    id: "participant-12",
    name: "Sasuke Uchiha",
    initials: "SU",
    role: "member" as const,
    muted: true,
  },
  { id: "participant-13", name: "Sakura Haruno", initials: "SH", role: "member" as const },
  { id: "participant-14", name: "Kakashi Hatake", initials: "KH", role: "moderator" as const },
];

const admissionRequests = [
  { id: "request-1", name: "Maki Zenin", initials: "MZ", requestedAt: minutesAgo(0) },
  { id: "request-2", name: "Toge Inumaki", initials: "TI", requestedAt: minutesAgo(1) },
  { id: "request-3", name: "Panda", initials: "PA", requestedAt: minutesAgo(2) },
  { id: "request-4", name: "Kento Nanami", initials: "KN", requestedAt: minutesAgo(2) },
  { id: "request-5", name: "Tony Tony Chopper", initials: "TC", requestedAt: minutesAgo(3) },
  { id: "request-6", name: "Usopp", initials: "US", requestedAt: minutesAgo(4) },
  { id: "request-7", name: "Hinata Hyuga", initials: "HH", requestedAt: minutesAgo(5) },
  { id: "request-8", name: "Shikamaru Nara", initials: "SN", requestedAt: minutesAgo(6) },
];

function minutesAgo(minutes: number) {
  return new Date(Date.now() - minutes * 60_000).toISOString();
}

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
const observedAt = new Date().toISOString();
const liveTiming = {
  status: "available" as const,
  startedAt: new Date(Date.now() - 52 * 60_000).toISOString(),
  endedAt: null,
  observedAt,
};

const meta = {
  title: "Sessions/Session Screen",
  component: SessionScreen,
  args: {
    title: "Weekly research discussion",
    state: liveState,
    sessionTiming: liveTiming,
    admissionRequests,
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
  args: {
    state: endedState,
    admissionRequests: [],
    sessionTiming: { ...liveTiming, endedAt: observedAt },
  },
};

export const Rejected: Story = {
  args: { state: rejectedState, admissionRequests: [] },
};

export const Unavailable: Story = {
  args: { state: unavailableState, admissionRequests: [] },
};
