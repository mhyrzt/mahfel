import type { Activity, InteractionsSnapshot } from "../types";

export const poll: Activity = {
  id: "11111111-1111-4111-8111-111111111111",
  kind: "poll",
  status: "open",
  question: "برای جلسهٔ بعد کدام موضوع‌ها را پیشنهاد می‌کنید؟",
  options: ["آشنایی با WebRTC", "طراحی رابط کاربری و دسترس‌پذیری", "پرسش و پاسخ دربارهٔ پروژه"],
  multiple: true,
  anonymous: true,
  selected: [],
  vote_version: 0,
  submitted: false,
  participation_count: null,
  draft_correct_options: null,
  draft_explanation: null,
  results: null,
};
export const pollResults: Activity = {
  ...poll,
  selected: [0, 1],
  vote_version: 1,
  submitted: true,
  results: {
    respondents: 10,
    counts: [8, 6, 2],
    correct_options: null,
    correct: null,
    explanation: null,
  },
};
export const quiz: Activity = {
  ...poll,
  id: "22222222-2222-4222-8222-222222222222",
  kind: "quiz",
  question: "کدام عددها اول هستند؟",
  options: ["۲", "۳", "۴"],
  selected: [0],
  submitted: true,
  vote_version: 1,
};
export const quizClosed: Activity = {
  ...quiz,
  status: "closed",
  results: {
    respondents: 10,
    counts: [8, 7, 2],
    correct_options: [0, 1],
    correct: false,
    explanation: "۲ و ۳ عدد اول هستند. هر دو گزینه باید انتخاب شوند.",
  },
};
export const snapshot: InteractionsSnapshot = {
  version: 4,
  live: true,
  can_manage: true,
  can_moderate: true,
  can_respond: true,
  activities: [pollResults, quizClosed],
  hands: [
    { actor_id: "actor-1", participant_id: "participant-1", name: "سارا احمدی", order: 1 },
    { actor_id: "actor-2", participant_id: "participant-2", name: "Ali رضایی", order: 2 },
  ],
};
