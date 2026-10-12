import { describe, expect, it } from "vitest";
import {
  delightPose,
  playfulPose,
  initialMotion,
  confusedPose,
  normalizeMotion,
  Spring,
} from "../core/motion";
import { fillFlame } from "../core/renderer";
describe("Parto motion", () => {
  it("bounds inputs and rejects nonfinite values", () => {
    expect(normalizeMotion({ energy: 4, gazeX: -4 })).toMatchObject({ energy: 1, gazeX: -1 });
    expect(() => normalizeMotion({ audioLevel: NaN })).toThrow(RangeError);
  });
  it("preserves independent activity and emotion", () => {
    expect(
      normalizeMotion({ activity: "listening" }, normalizeMotion({ emotion: "happy" })),
    ).toMatchObject({ activity: "listening", emotion: "happy" });
  });
  it("converges at different frame rates and bounds long delays", () => {
    for (const fps of [30, 60, 144]) {
      const spring = new Spring();
      for (let i = 0; i < fps * 3; i++) {
        spring.step(1, 1 / fps);
      }
      expect(spring.value).toBeCloseTo(1, 4);
    }
    const spring = new Spring();
    expect(spring.step(1, 300)).toBeLessThan(1);
  });
  it("recovers from peak confusion continuously", () => {
    expect(confusedPose(3.6).squint).toBe(1);
    expect(confusedPose(9)).toEqual({ curl: 0, tilt: 0, squint: 0 });
    expect(Math.abs(confusedPose(3.599).curl - confusedPose(3.601).curl)).toBeLessThan(0.01);
  });
  it("anchors flame flicker at the ribbon and freezes it for reduced motion", () => {
    const first = new Float32Array(128);
    const later = new Float32Array(128);
    fillFlame(first, 0.2, 0, 1);
    fillFlame(later, 0.2, 0.25, 1);
    expect((first[0] + first[2]) / 2).toBeCloseTo((later[0] + later[2]) / 2);
    expect((first[1] + first[3]) / 2).toBeCloseTo((later[1] + later[3]) / 2);
    let tipTravel = 0;
    for (let phase = 0; phase < 1; phase += 0.02) {
      fillFlame(later, 0.2, phase, 1);
      tipTravel = Math.max(tipTravel, Math.hypot(first[124] - later[124], first[125] - later[125]));
    }
    expect(tipTravel).toBeGreaterThan(1);
    expect(tipTravel).toBeLessThan(9);
    fillFlame(first, 0.2, 0, 0);
    fillFlame(later, 0.2, 10, 0);
    expect(first).toEqual(later);
    for (let phase = 0; phase < 12; phase += 0.1) {
      fillFlame(first, 1.8, phase, 1.4);
      expect(Array.from(first).every(Number.isFinite)).toBe(true);
    }
  });
  it("keeps the tapered mesh finite at extreme curls", () => {
    for (const curl of [0, 0.5, 1, 1.8]) {
      const vertices = new Float32Array(128);
      fillFlame(vertices, curl, 9);
      expect(Array.from(vertices).every(Number.isFinite)).toBe(true);
      expect(vertices[124]).toBeCloseTo(vertices[126]);
      expect(vertices[125]).toBeCloseTo(vertices[127]);
    }
  });
});

describe("playful choreography", () => {
  it("anticipates, jumps and settles back to neutral", () => {
    expect(delightPose(0.18).lift).toBeGreaterThan(0);
    expect(delightPose(0.65).lift).toBeLessThan(-15);
    expect(delightPose(3).joy).toBe(0);
    expect(Math.abs(delightPose(3).lift)).toBeLessThan(0.01);
  });
  it("keeps continuous floating calm without automatic hops and gives listening a quieter gaze", () => {
    const happy = { ...initialMotion, emotion: "excited" as const, energy: 1 };
    for (let time = 0; time < 30; time += 0.05) {
      const pose = playfulPose(happy, time);
      expect(Object.values(pose).every(Number.isFinite)).toBe(true);
      expect(Math.abs(pose.lift)).toBeLessThanOrEqual(3.2);
      expect(Math.abs(pose.squash)).toBeLessThanOrEqual(0.004);
    }
    expect(playfulPose({ ...initialMotion, activity: "listening" }, 2).gazeX).toBe(0);
  });
});
