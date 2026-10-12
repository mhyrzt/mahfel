export const activities = [
  "idle",
  "listening",
  "thinking",
  "speaking",
  "confused",
  "sleeping",
] as const;
export const emotions = ["neutral", "happy", "curious", "confused", "excited", "sad"] as const;
export const designs = [
  "faithful",
  "vector",
  "dimensional",
  "elastic",
  "magical",
  "minimal",
] as const;
export type PartoActivity = (typeof activities)[number];
export type PartoEmotion = (typeof emotions)[number];
export type PartoDesign = (typeof designs)[number];
export interface MotionState {
  activity: PartoActivity;
  emotion: PartoEmotion;
  energy: number;
  happiness: number;
  confusion: number;
  sleepiness: number;
  attention: number;
  audioLevel: number;
  gazeX: number;
  gazeY: number;
}
export const initialMotion: MotionState = {
  activity: "idle",
  emotion: "neutral",
  energy: 0.35,
  happiness: 0.5,
  confusion: 0,
  sleepiness: 0,
  attention: 0.5,
  audioLevel: 0,
  gazeX: 0,
  gazeY: 0,
};
export function normalizeMotion(
  patch: Partial<MotionState>,
  previous = initialMotion,
): MotionState {
  const next = { ...previous, ...patch };
  if (!activities.includes(next.activity) || !emotions.includes(next.emotion)) {
    throw new RangeError("Unknown Parto state");
  }
  for (const key of [
    "energy",
    "happiness",
    "confusion",
    "sleepiness",
    "attention",
    "audioLevel",
    "gazeX",
    "gazeY",
  ] as const) {
    if (!Number.isFinite(next[key])) {
      throw new RangeError(`${key} must be finite`);
    }
    next[key] = Math.max(key.startsWith("gaze") ? -1 : 0, Math.min(1, next[key]));
  }
  return next;
}
export class Spring {
  value = 0;
  velocity = 0;
  step(target: number, delta: number) {
    let remaining = Math.min(0.05, Math.max(0, delta));
    while (remaining > 0) {
      const dt = Math.min(remaining, 1 / 240);
      this.velocity += ((target - this.value) * 110 - this.velocity * 19) * dt;
      this.value += this.velocity * dt;
      remaining -= dt;
    }
    return this.value;
  }
}
export function confusedPose(time: number) {
  const knots = [
    [0, 0, 0, 0],
    [1, 0.25, -0.08, 0],
    [2.4, 0.8, 0.12, 0.1],
    [3.6, 1, -0.16, 1],
    [4.5, 0.9, 0.13, 1],
    [6.5, 0.75, -0.08, 0],
    [8, 0.3, 0.04, 0],
    [9, 0, 0, 0],
  ];
  const t = Math.max(0, Math.min(9, time));
  let index = 0;
  while (index < knots.length - 2 && t > knots[index + 1][0]) {
    index++;
  }
  const a = knots[index];
  const b = knots[index + 1];
  const u = (t - a[0]) / (b[0] - a[0]);
  const s = u * u * (3 - 2 * u);
  return {
    curl: a[1] + (b[1] - a[1]) * s,
    tilt: a[2] + (b[2] - a[2]) * s,
    squint: a[3] + (b[3] - a[3]) * s,
  };
}

export function delightPose(time: number) {
  const t = Math.max(0, time);
  const anticipation = Math.exp(-Math.pow((t - 0.18) / 0.12, 2));
  const jump = Math.exp(-Math.pow((t - 0.65) / 0.27, 2));
  const settle = Math.sin((t - 1) * 12) * Math.exp(-Math.max(0, t - 1) * 4);
  const landing = t > 1 ? settle : 0;
  const fade = Math.max(0, 1 - t / 2.8);
  return {
    lift: -22 * jump + 5 * anticipation + 3 * landing,
    squash: -0.08 * anticipation + 0.055 * jump - 0.035 * landing,
    tilt: Math.sin(t * 7) * 0.14 * fade,
    joy: Math.sin((Math.min(1, t / 0.3) * Math.PI) / 2) * fade,
    curl: jump * 0.55,
  };
}
export interface PlayfulPose {
  lift: number;
  squash: number;
  tilt: number;
  curl: number;
  joy: number;
  gazeX: number;
  gazeY: number;
  sleepy: number;
  speaking: number;
}
export function playfulPose(state: MotionState, phase: number): PlayfulPose {
  const happy = state.emotion === "happy" || state.emotion === "excited";
  const resting = state.activity === "sleeping";
  const focused = state.activity === "listening";
  const thinking = state.activity === "thinking";
  const joy = happy ? 0.45 + state.happiness * 0.5 : 0;
  const amplitude = (1.2 + state.energy * 2) * (focused ? 0.4 : 1);
  const drift = Math.sin(phase * 0.7) * Math.sin(phase * 1.1);
  return {
    lift: Math.sin(phase * (resting ? 0.45 : 1.1)) * amplitude,
    squash: Math.sin(phase * (resting ? 0.5 : 1.2)) * 0.004,
    tilt: drift * (focused ? 0.008 : 0.018) + state.gazeX * 0.035,
    curl: 0.18 + Math.sin(phase * 0.8) * 0.025 + (thinking ? 0.35 : 0),
    joy,
    gazeX: state.gazeX * 4 + (focused ? 0 : Math.sin(phase * 0.55) * 1.5),
    gazeY: state.gazeY * 3 + (thinking ? -3 : 0) + Math.sin(phase * 0.8) * 0.8,
    sleepy: Math.max(state.sleepiness, resting ? 1 : 0),
    speaking: state.activity === "speaking" ? state.audioLevel : 0,
  };
}
