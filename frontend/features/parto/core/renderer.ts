import { Application, Container, FillGradient, Graphics, MeshSimple, Texture } from "pixi.js";
import {
  confusedPose,
  delightPose,
  playfulPose,
  initialMotion,
  normalizeMotion,
  Spring,
  type MotionState,
  type PartoDesign,
} from "./motion";

export interface PartoOptions extends Partial<MotionState> {
  design?: PartoDesign;
  size?: number;
  speed?: number;
  paused?: boolean;
  reducedMotion?: boolean;
}
export interface PartoController {
  update(options: PartoOptions): void;
  play(sequence: "confused" | "delight"): void;
  restart(): void;
  destroy(): void;
}
function gradient(colors: string[]) {
  return new FillGradient({
    type: "linear",
    textureSpace: "global",
    start: { x: 100, y: 65 },
    end: { x: 175, y: 175 },
    colorStops: colors.map((color, i) => ({ offset: i / (colors.length - 1), color })),
  });
}
export function fillFlame(vertices: Float32Array, curl: number, phase: number, intensity = 1) {
  const count = vertices.length / 4;
  for (let i = 0; i < count; i++) {
    const t = i / (count - 1);
    const angle = t * Math.PI * 1.9;
    const pulse = Math.sin(phase * 2.1) * 0.65 + Math.sin(phase * 3.5 + 1.2) * 0.35;
    const wave = phase * 2.4 - t * 5.5;
    const ripple = phase * 3.9 - t * 9;
    const flicker = Math.sin(wave) * 2.5 + Math.sin(ripple) * 0.8;
    const envelope = t * t * intensity;
    const height = 12 + curl * 64 + intensity * (21 + pulse * 3);
    const x = 148 - 20 * Math.sin(t * Math.PI) + curl * 13 * Math.sin(angle) + flicker * envelope;
    const y = 85 - t * height;
    const dx =
      -20 * Math.PI * Math.cos(t * Math.PI) +
      curl * 13 * Math.PI * 1.9 * Math.cos(angle) +
      2 * t * intensity * flicker -
      envelope * (Math.cos(wave) * 13.75 + Math.cos(ripple) * 7.2);
    const dy = -height;
    const length = Math.hypot(dx, dy);
    const width = 15 * Math.pow(1 - t, 1.6);
    vertices[i * 4] = x - (dy / length) * width;
    vertices[i * 4 + 1] = y + (dx / length) * width;
    vertices[i * 4 + 2] = x + (dy / length) * width;
    vertices[i * 4 + 3] = y - (dx / length) * width;
  }
}
export async function createParto(
  host: HTMLElement,
  options: PartoOptions = {},
): Promise<PartoController> {
  let state = normalizeMotion(options);
  let settings = {
    design: "faithful" as PartoDesign,
    size: 256,
    speed: 1,
    paused: false,
    reducedMotion: false,
    ...options,
  };
  if (!Number.isFinite(settings.size) || settings.size < 16 || settings.size > 1024) {
    throw new RangeError("Parto size must be between 16 and 1024");
  }
  if (!Number.isFinite(settings.speed) || settings.speed < 0 || settings.speed > 3) {
    throw new RangeError("Parto speed must be between 0 and 3");
  }
  const app = new Application();
  try {
    await app.init({
      width: settings.size,
      height: settings.size,
      backgroundAlpha: 0,
      antialias: true,
      resolution: Math.min(2, window.devicePixelRatio || 1),
      autoDensity: true,
      preference: "webgl",
    });
  } catch (error) {
    if (app.renderer) {
      app.destroy(true, { children: true });
    }
    throw error;
  }
  host.appendChild(app.canvas);
  const root = new Container();
  const character = new Container();
  root.scale.set(settings.size / 256);
  app.stage.addChild(root);
  const shadow = new Graphics();
  const back = new Graphics();
  const face = new Graphics();
  const front = new Graphics();
  const eyes = new Graphics();
  const effects = new Graphics();
  const surface = gradient(["#C4A4FF", "#9870F5", "#7548E4", "#8F64F5"]);
  const fold = gradient(["#281354", "#43218B", "#784BD9"]);
  const cream = new FillGradient({
    type: "radial",
    textureSpace: "global",
    center: { x: 160, y: 141 },
    outerCenter: { x: 151, y: 136 },
    innerRadius: 8,
    outerRadius: 48,
    colorStops: [
      { offset: 0, color: "#FFF8E6" },
      { offset: 0.65, color: "#FFF1DC" },
      { offset: 1, color: "#DCC3E8" },
    ],
  });
  const canvas = document.createElement("canvas");
  canvas.width = 64;
  canvas.height = 8;
  const context = canvas.getContext("2d");
  if (!context) {
    app.destroy(true, { children: true });
    throw new Error("Parto shading context unavailable");
  }
  const ramp = context.createLinearGradient(0, 0, 64, 0);
  ramp.addColorStop(0, "#7040D8");
  ramp.addColorStop(0.6, "#A781FF");
  ramp.addColorStop(1, "#C7ABFF");
  context.fillStyle = ramp;
  context.fillRect(0, 0, 64, 8);
  const texture = Texture.from(canvas);
  const samples = 32;
  const vertices = new Float32Array(samples * 4);
  const uvs = new Float32Array(samples * 4);
  const indices = new Uint32Array((samples - 1) * 6);
  for (let i = 0; i < samples; i++) {
    uvs.set([0, i / (samples - 1), 1, i / (samples - 1)], i * 4);
    if (i < samples - 1) {
      indices.set([i * 2, i * 2 + 1, i * 2 + 2, i * 2 + 1, i * 2 + 3, i * 2 + 2], i * 6);
    }
  }
  fillFlame(vertices, 0, 0);
  const flame = new MeshSimple({ texture, vertices, uvs, indices });
  root.addChild(shadow, character, effects);
  character.addChild(flame, back, face, front, eyes);
  const curlSpring = new Spring();
  const tiltSpring = new Spring();
  const squintSpring = new Spring();
  const motionSprings = {
    lift: new Spring(),
    squash: new Spring(),
    tilt: new Spring(),
    curl: new Spring(),
    joy: new Spring(),
    gazeX: new Spring(),
    gazeY: new Spring(),
    sleepy: new Spring(),
    speaking: new Spring(),
  };
  let delightTime = -1;
  let phase = 0;
  let sequenceTime = -1;
  let destroyed = false;
  function draw(delta: number) {
    if (destroyed) {
      return;
    }
    const dt = Math.min(0.05, Math.max(0, delta));
    if (!settings.paused && !settings.reducedMotion) {
      phase += dt * settings.speed;
      if (delightTime >= 0) {
        delightTime += dt * settings.speed;
        if (delightTime > 2.8) {
          delightTime = -1;
        }
      }
      if (sequenceTime >= 0) {
        sequenceTime += dt * settings.speed;
        if (sequenceTime > 9) {
          sequenceTime = -1;
        }
      }
    }
    const target = playfulPose(state, phase);
    const reaction = delightPose(delightTime >= 0 ? delightTime : 3);
    const poseDelta = settings.paused ? 0 : dt;
    const motion = settings.reducedMotion
      ? target
      : {
          lift: motionSprings.lift.step(target.lift + reaction.lift, poseDelta),
          squash: motionSprings.squash.step(target.squash + reaction.squash, poseDelta),
          tilt: motionSprings.tilt.step(target.tilt + reaction.tilt, poseDelta),
          curl: motionSprings.curl.step(target.curl + reaction.curl, poseDelta),
          joy: motionSprings.joy.step(Math.max(target.joy, reaction.joy), poseDelta),
          gazeX: motionSprings.gazeX.step(target.gazeX, poseDelta),
          gazeY: motionSprings.gazeY.step(target.gazeY, poseDelta),
          sleepy: motionSprings.sleepy.step(target.sleepy, poseDelta),
          speaking: motionSprings.speaking.step(target.speaking, poseDelta),
        };
    const confused = state.activity === "confused" || state.emotion === "confused";
    const pose = confusedPose(sequenceTime >= 0 ? sequenceTime : phase % 9);
    const amount = Math.max(state.confusion, confused ? 1 : 0);
    const active = sequenceTime >= 0 ? 1 : amount;
    const curl = settings.reducedMotion
      ? pose.curl * active + motion.curl
      : curlSpring.step(pose.curl * active + motion.curl, poseDelta);
    const tilt = settings.reducedMotion
      ? pose.tilt * active + motion.tilt
      : tiltSpring.step(pose.tilt * active + motion.tilt, poseDelta);
    const squint = settings.reducedMotion
      ? pose.squint * active
      : squintSpring.step(pose.squint * active, poseDelta);
    const elastic = settings.design === "elastic" ? 1.8 : 1;
    const float = settings.reducedMotion ? 0 : motion.lift;
    const squash = settings.reducedMotion ? 0 : motion.squash * elastic;
    character.pivot.set(140, 130);
    character.position.set(
      140 + (settings.reducedMotion ? 0 : Math.sin(phase * 0.8) * 2 + motion.gazeX * 0.8),
      130 + float,
    );
    character.rotation = settings.reducedMotion ? 0 : tilt * elastic;
    character.scale.set(1 - squash, 1 + squash + motion.speaking * 0.012);
    fillFlame(
      vertices,
      settings.reducedMotion ? 0.08 : curl * elastic,
      phase,
      settings.reducedMotion ? 0 : (1 - motion.sleepy * 0.93) * (0.85 + state.energy * 0.55),
    );
    flame.geometry.getBuffer("aPosition").update();
    flame.visible = settings.design !== "minimal";
    shadow.clear();
    for (let i = 8; i > 0; i--) {
      shadow
        .ellipse(143, 209, 20 + i * 2 - float * 0.3, 2 + i * 0.8)
        .fill({ color: 0x9164ed, alpha: 0.018 });
    }
    const fire = settings.reducedMotion ? 0 : (1 - motion.sleepy * 0.93) * elastic;
    const crest = fire * (Math.sin(phase * 2.1) * 1.5 + Math.sin(phase * 3.5 + 1.2) * 0.5);
    const tip = fire * (Math.sin(phase * 2.4 - 1) * 2 + Math.sin(phase * 3.9) * 0.6);
    const sway = settings.reducedMotion
      ? 0
      : Math.sin(phase * 0.9) * (0.8 + state.energy) * elastic + curl * 5;
    const tail = settings.reducedMotion
      ? 0
      : Math.sin(phase * 1.1 - 0.8) * (1 + state.energy) * elastic +
        fire * (Math.sin(phase * 2.4 - 2.8) * 1.5 + Math.sin(phase * 3.7) * 0.5);
    const flat = settings.design === "vector" || settings.design === "minimal";
    back
      .clear()
      .moveTo(157, 80)
      .bezierCurveTo(210, 68, 230, 138, 182, 173)
      .bezierCurveTo(160, 183, 122, 157, 131, 115)
      .bezierCurveTo(139, 98, 145, 88, 157, 80)
      .closePath()
      .fill(flat ? 0x43218b : fold);
    face
      .clear()
      .ellipse(151, 141, 44, 39)
      .fill(flat ? 0xfff1dc : cream);
    front
      .clear()
      .moveTo(48 - tail * 0.5, 148 + tail)
      .bezierCurveTo(80, 154 + sway, 91 + sway, 89, 127, 70 - sway)
      .bezierCurveTo(
        160 + sway + tip * 0.4,
        49 - sway - crest,
        193 + tip,
        66 - crest * 0.7,
        208 + tip,
        96 - crest * 0.4,
      )
      .bezierCurveTo(184 + tip * 0.7, 74 - crest * 0.5, 166, 94, 147, 117)
      .bezierCurveTo(117, 154 + tail * 0.3, 92, 184 + tail * 0.5, 62, 163 + tail)
      .quadraticCurveTo(52, 157 + tail, 48 - tail * 0.5, 148 + tail)
      .closePath()
      .fill(flat ? 0x8054e8 : surface);
    if (!flat) {
      front
        .moveTo(53 - tail * 0.4, 150 + tail)
        .bezierCurveTo(88, 159 + tail * 0.6, 100 + sway, 100, 135, 78 - sway * 0.5)
        .bezierCurveTo(
          162 + sway + tip * 0.3,
          61 - sway * 0.5 - crest,
          186 + tip * 0.8,
          72 - crest * 0.6,
          201 + tip,
          88 - crest * 0.4,
        )
        .stroke({ color: 0xc7abff, alpha: 0.22, width: 1.2 });
      front
        .moveTo(91, 151)
        .bezierCurveTo(122, 136, 135, 101, 166, 83)
        .stroke({ color: 0xb694ff, alpha: 0.12, width: 9 });
    }
    eyes.clear();
    const blinkPhase = (phase + Math.sin(phase * 0.37) * 0.35) % 4.7;
    const blink = blinkPhase > 4.48 ? Math.sin(((blinkPhase - 4.48) / 0.22) * Math.PI) : 0;
    const closed = Math.max(blink, squint, motion.sleepy);
    const gazeY = motion.gazeY + (state.emotion === "sad" ? 2 : 0);
    for (const x of [151, 177]) {
      const ex = x + motion.gazeX;
      const ey = 145 + gazeY;
      const joyBlend = Math.max(0, Math.min(1, motion.joy * 1.6));
      const sleepyBlend = Math.max(0, Math.min(1, (closed - 0.6) / 0.4));
      const openAlpha = (1 - squint) * (1 - sleepyBlend);
      eyes
        .ellipse(ex, ey, 4.1, Math.max(1, 7.5 * (1 - closed)))
        .fill({ color: 0x26164e, alpha: openAlpha * (1 - joyBlend) });
      eyes
        .moveTo(ex - 4, ey + 2)
        .quadraticCurveTo(ex, ey - motion.joy * 6, ex + 4, ey + 2)
        .stroke({ color: 0x26164e, width: 3, cap: "round", alpha: openAlpha * joyBlend });
      eyes
        .moveTo(ex - 4, ey)
        .quadraticCurveTo(ex, ey + 2, ex + 4, ey)
        .stroke({ color: 0x26164e, width: 2.5, cap: "round", alpha: sleepyBlend * (1 - squint) });
      const direction = x === 151 ? 1 : -1;
      eyes
        .moveTo(ex - direction * 4, ey - 3)
        .lineTo(ex + direction * 3, ey)
        .lineTo(ex - direction * 4, ey + 2)
        .stroke({ color: 0x26164e, width: 3, cap: "round", join: "round", alpha: squint });
    }

    if (motion.speaking > 0.02) {
      eyes
        .ellipse(163, 162, 2.5, 1 + motion.speaking * (1.5 + (Math.sin(phase * 11) + 1) * 1.5))
        .fill(0x43218b);
    } else if (motion.joy > 0.5) {
      eyes
        .moveTo(159, 160)
        .quadraticCurveTo(163, 163 + motion.joy * 2, 167, 160)
        .stroke({
          color: 0x43218b,
          width: 1.6,
          cap: "round",
          alpha: Math.min(1, (motion.joy - 0.5) * 2),
        });
    }
    effects.clear();
    if (active > 0.1 && curl > 0.25 && settings.design !== "minimal") {
      const qx = 213 + Math.sin(phase) * 4;
      const qy = 76 + Math.cos(phase * 1.3) * 6;
      effects
        .moveTo(qx - 5, qy - 5)
        .bezierCurveTo(qx - 5, qy - 15, qx + 10, qy - 15, qx + 8, qy - 5)
        .quadraticCurveTo(qx + 7, qy - 2, qx + 2, qy + 1)
        .lineTo(qx + 2, qy + 4)
        .stroke({ color: 0x9466ef, width: 4, cap: "round" });
      effects.circle(qx + 2, qy + 12, 2.3).fill(0x9466ef);
    }
    if (delightTime >= 0 && !settings.reducedMotion && settings.design !== "minimal") {
      for (let i = 0; i < 5; i++) {
        const travel = Math.min(1, delightTime / 1.5);
        const angle = i * 1.3 - 3;
        const x = 143 + Math.cos(angle) * (42 + travel * 35);
        const y = 125 + Math.sin(angle) * (38 + travel * 25) + travel * 14;
        const alpha = Math.max(0, 1 - delightTime / 2);
        const radius = 2 + Math.sin(delightTime * 5 + i) * 0.7;
        effects
          .moveTo(x - radius, y)
          .lineTo(x + radius, y)
          .moveTo(x, y - radius)
          .lineTo(x, y + radius)
          .stroke({ color: 0xc7abff, width: 1.5, alpha });
      }
    }
    if (settings.design === "magical" || active > 0.2 || state.activity === "thinking") {
      for (let i = 0; i < 3; i++) {
        const t = (phase * 0.25 + i / 3) % 1;
        effects
          .ellipse(95 + Math.sin(i * 3 + phase) * 35, 105 - t * 65, 2 + Math.sin(t * Math.PI), 4)
          .fill({ color: 0xa781ff, alpha: Math.sin(t * Math.PI) * 0.55 });
      }
    }
    if (settings.design === "dimensional" || settings.design === "magical") {
      effects.ellipse(132, 96 + float, 17, 4).fill({ color: 0xd8c1ff, alpha: 0.1 });
    }
  }
  draw(0);
  app.ticker.add((ticker) => draw(settings.paused ? 0 : ticker.deltaMS / 1000));
  return {
    update(next) {
      if (destroyed) {
        return;
      }
      if (next.activity !== undefined && next.activity !== state.activity) {
        sequenceTime = -1;
      }
      state = normalizeMotion(next, state);
      settings = { ...settings, ...next };
      if (!Number.isFinite(settings.speed) || settings.speed < 0 || settings.speed > 3) {
        settings.speed = 1;
      }
      if (next.size !== undefined && Number.isFinite(next.size)) {
        const size = Math.max(16, Math.min(1024, next.size));
        app.renderer.resize(size, size);
        root.scale.set(size / 256);
      }
    },
    play(sequence) {
      if (sequence === "delight") {
        delightTime = 0;
      } else {
        sequenceTime = 0;
      }
    },
    restart() {
      phase = 0;
      sequenceTime = -1;
      delightTime = -1;
    },
    destroy() {
      if (destroyed) {
        return;
      }
      destroyed = true;
      app.destroy(true, { children: true });
      texture.destroy(true);
      surface.destroy();
      fold.destroy();
      cream.destroy();
    },
  };
}
export { initialMotion };
