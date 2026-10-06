import type { Meta, StoryObj } from "@storybook/nextjs";
import { fn } from "storybook/test";
import { DirectionProvider } from "@/components/ui/direction";
import { ActivityCard } from "../activity-card";
import { ActivityComposer } from "../activity-composer";
import { ActivitiesPanel } from "../activities-panel";
import { HandQueue } from "../hand-queue";
import { poll, pollResults, quiz, quizClosed, snapshot } from "./fixtures";

const meta = {
  title: "Sessions/Interactions",
  component: ActivityCard,
  parameters: { layout: "centered" },
  decorators: [
    (Story, context) => (
      <DirectionProvider dir={context.args.locale === "fa" ? "rtl" : "ltr"}>
        <div
          dir={context.args.locale === "fa" ? "rtl" : "ltr"}
          className="w-[min(24rem,calc(100vw-2rem))]"
        >
          <Story />
        </div>
      </DirectionProvider>
    ),
  ],
  args: {
    activity: poll,
    sessionId: "33333333-3333-4333-8333-333333333333",
    locale: "fa",
    busy: false,
    canManage: false,
    canRespond: true,
    hasOpenActivity: true,
    send: fn(async () => true),
    onEdit: fn(),
  },
} satisfies Meta<typeof ActivityCard>;
export default meta;
type Story = StoryObj<typeof meta>;
export const PollVoting: Story = {};
export const PollResults: Story = { args: { activity: pollResults } };
export const QuizSubmitted: Story = { args: { activity: quiz } };
export const QuizResults: Story = { args: { activity: quizClosed } };
export const EnglishPoll: Story = {
  args: {
    locale: "en",
    activity: {
      ...poll,
      question: "Which topics should we discuss next?",
      options: ["WebRTC", "Accessible interfaces", "Project questions"],
    },
  },
};
export const Composer: Story = {
  render: (args) => (
    <div className="flex h-[70dvh] min-h-0 flex-col">
      <ActivityComposer locale={args.locale} onClose={fn()} busy={false} send={args.send} />
    </div>
  ),
};
export const HostPanel: Story = {
  render: (args) => (
    <ActivitiesPanel
      sessionId={args.sessionId}
      locale={args.locale}
      snapshot={snapshot}
      busy={false}
      error={null}
      send={args.send}
      onRetry={fn()}
    />
  ),
};
export const RaisedHands: Story = {
  render: (args) => (
    <HandQueue
      hands={snapshot.hands}
      version={snapshot.version}
      locale={args.locale}
      busy={false}
      canModerate
      send={args.send}
    />
  ),
};
export const LongMixedText: Story = {
  args: {
    activity: {
      ...poll,
      question:
        "در این پرسش دربارهٔ طراحی رابط کاربری RTL و پیاده‌سازی گزینه‌های بسیار طولانی در نمایشگر کوچک کدام راهکار را انتخاب می‌کنید؟",
      options: [
        "این گزینهٔ طولانی شامل توضیح فارسی و عبارت انگلیسی WebRTC API v2 است و باید در چند خط بدون بیرون‌زدگی نمایش داده شود.",
        "نمونهٔ دوم برای بررسی فاصله‌گذاری، متن و جایگاه نشان انتخاب در حالت راست‌به‌چپ",
        "https://example.test/verylongunbrokenmixedtext12345678901234567890123456789012345678901234567890",
      ],
    },
  },
};
