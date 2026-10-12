# Parto / پرتو

Procedural PixiJS v8 mascot, explored in the existing Storybook. No new application or deployment boundary.

From `frontend/`: `bun install`, `bun run storybook`, then open **Brand / Parto** at http://localhost:6006. Storybook Controls change design, activity, emotion, gaze, audio, energy, confusion, speed and pause without restarting the renderer. The Confused Sequence story plays a nine-second semantic sequence once; the confused activity loops it. Reload the story to replay the one-shot sequence.

```tsx
import { Parto } from "@/features/parto/components/parto";

<Parto size={128} activity="speaking" emotion="happy" audioLevel={0.7} />;
```

For non-React use, await `createParto(host, options)` from `core/renderer`, then use `update`, `play("confused")`, `restart` and `destroy`. The host owns the returned controller. The React wrapper lazy-loads PixiJS and handles initialization races, errors, media-query changes and cleanup.

## Design

`assets/parto.png` is the canonical supplied image, preserved unchanged. A copy in `public/references` is used only by the reference story. Frame 01 establishes the asymmetric swept م ribbon: tapered left tail, broad rising arch, dark right inner fold, small cream oval tucked underneath. Frames 08–11 squeeze the eyes during peak curl. Frames 13–18 stretch the connected upper tip and release sparse particles. Frame 24 recovers toward the original arch.

The ribbon uses connected cubic Bézier boundaries, shaded with reusable GPU gradient textures. The crest and tapered left edge have restrained, slower flicker with traveling waves. The connected upper flame stretches and contracts with asynchronous pulses, with motion concentrated toward its tip; sleeping softens the flame and reduced motion freezes it. The upper extension is a tapered triangle strip with a curved centerline and local perpendicular widths, updated in place. The face is an oval behind the front ribbon. Palette starts at #8054E8, #A781FF, #43218B, #C7ABFF, #FFF1DC and #26164E; illustration colors are deliberately independent of UI theme tokens.

Faithful retains dimensional folds; vector flattens surfaces; dimensional adds restrained light; elastic amplifies motion; magical introduces sparse drifting lights; minimal removes the tip and effects for small avatars. These are shared-renderer interpretations, not six independent engines. Exact reference shading and more elaborate lighting remain opportunities for art-direction refinement.

## Motion and ownership

The Pixi ticker owns frame updates; React never receives per-frame state. Activity and emotion are separate from numeric parameters. Inputs clamp to normalized ranges and reject nonfinite values. Springs use bounded substeps, long frame gaps clamp to 50 ms, and phase persists through prop changes. Confusion blends semantic curl, tilt and squint targets through springs. `prefers-reduced-motion` freezes continuous movement and applies expression targets immediately, avoiding residual spring movement. Each mounted component owns one GPU context; do not mount unbounded grids of instances.

Destroy removes the canvas, ticker, children, mesh and owned textures/gradients. Shader programs are not recreated each frame. The main ribbon currently uses Pixi Graphics tessellation; the extension uses an explicit mesh. No sprite sheets, external artwork textures, image generation or CSS character animation.

## Validation

`bun run test features/parto/tests`, `bun run typecheck`, `bunx eslint features/parto --max-warnings=0`. Browser captures: from `e2e/`, `node parto-capture.mjs` against running Storybook; screenshots are saved under `e2e/artifacts/parto/`. This requires the existing Playwright Chromium installation. Reference fidelity requires inspecting those screenshots, not relying on unit tests.

## Playful motion

The **Playful** and **Excited** stories demonstrate happy eye arches, restrained ribbon movement, and calm floating. Explicit delight reactions retain anticipation, a little hop, and spring settling. Continuous floating and hair movement stay restrained in every emotion; hops occur only when explicitly activated. Listening settles into quieter motion, thinking curls upward, and sleeping slows the float. Gaze, joy, sleep and audio inputs blend through springs instead of switching shapes immediately.

Set `interactive` to let Parto follow the pointer and react to a tap, Enter or Space with a delight hop and sparse sparkles. Interaction is opt-in; ordinary avatars remain labeled images. Imperative clients can call `play("delight")`, and React can use `sequence="delight"`. Reduced motion suppresses hops, rotation, squash/stretch and delight sparkles. The reference silhouette remains the shape boundary throughout.

## Landing companion

`PartoCompanion` combines the mascot with an English/Persian greeting bubble. It follows pointer movement across the page through imperative updates, greets for 6.5 seconds, then settles into idle. The landing presentation has no replay or pause controls. It observes available width, cleans up its greeting timer and resize observer, and keeps all localization in the shared message catalog. This is a simulated welcome animation; it does not capture audio or call an AI service. The landing hero uses this companion in place of its large logo illustration.
