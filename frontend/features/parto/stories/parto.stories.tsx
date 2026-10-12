import type { Meta, StoryObj } from "@storybook/nextjs";
import { PartoCompanion } from "../components/parto-companion";
import { Parto } from "../components/parto";
import { activities, designs, emotions } from "../core/motion";

const meta = {
  title: "Brand/Parto",
  component: Parto,
  parameters: { layout: "centered" },
  args: {
    design: "faithful",
    activity: "idle",
    emotion: "neutral",
    size: 320,
    energy: 0.35,
    confusion: 0,
    audioLevel: 0.4,
    gazeX: 0,
    gazeY: 0,
    speed: 1,
    paused: false,
  },
  argTypes: {
    design: { control: "select", options: designs },
    activity: { control: "select", options: activities },
    emotion: { control: "select", options: emotions },
    energy: { control: { type: "range", min: 0, max: 1, step: 0.01 } },
    confusion: { control: { type: "range", min: 0, max: 1, step: 0.01 } },
    audioLevel: { control: { type: "range", min: 0, max: 1, step: 0.01 } },
    gazeX: { control: { type: "range", min: -1, max: 1, step: 0.01 } },
    gazeY: { control: { type: "range", min: -1, max: 1, step: 0.01 } },
    speed: { control: { type: "range", min: 0, max: 3, step: 0.1 } },
  },
} satisfies Meta<typeof Parto>;
export default meta;
type Story = StoryObj<typeof meta>;
export const Faithful: Story = {};
export const TalkingCompanion: Story = {
  render: () => (
    <div className="w-[min(380px,calc(100vw-40px))]">
      <PartoCompanion locale="en" />
    </div>
  ),
};
export const PersianCompanion: Story = {
  render: () => (
    <div className="w-[min(380px,calc(100vw-40px))]" lang="fa">
      <PartoCompanion locale="fa" />
    </div>
  ),
};
export const Playful: Story = {
  args: { emotion: "happy", energy: 0.75, happiness: 0.9, interactive: true, size: 400 },
  decorators: [
    (Story) => (
      <div
        style={{
          background: "#1B1428",
          color: "#EDE1FF",
          borderRadius: 24,
          padding: "16px 36px 32px",
          textAlign: "center",
        }}
      >
        <Story />
        <h2 style={{ fontSize: 24 }}>Hello, curious mind.</h2>
        <p style={{ color: "#B9A5D3" }}>Move around to catch Parto’s eye. Tap for a little joy.</p>
      </div>
    ),
  ],
};
export const Excited: Story = {
  args: { emotion: "excited", energy: 1, happiness: 1, interactive: true },
};
export const DelightSequence: Story = { args: { sequence: "delight" } };
export const Listening: Story = {
  args: { activity: "listening", emotion: "curious", gazeX: 0.5, attention: 0.9 },
};
export const Thinking: Story = { args: { activity: "thinking", emotion: "curious" } };
export const Confused: Story = { args: { activity: "confused", emotion: "confused" } };
export const ConfusedSequence: Story = { args: { sequence: "confused" } };
export const Speaking: Story = {
  args: { activity: "speaking", emotion: "happy", audioLevel: 0.7 },
};
export const Sleeping: Story = { args: { activity: "sleeping" } };
export const ReducedMotion: Story = { args: { reducedMotion: true } };
const descriptions = [
  "Reference silhouette & folded satin",
  "Crisp, flat vector surfaces",
  "Soft depth & lavender highlights",
  "Springy tilt & expressive curls",
  "Sparse drifting light particles",
  "Quiet geometry for small avatars",
];
export const SixInterpretations: Story = {
  parameters: { layout: "fullscreen" },
  render: (args) => (
    <main style={{ background: "#14111D", color: "#F3EDFF", padding: 40, minHeight: "100vh" }}>
      <div style={{ maxWidth: 1000, margin: "auto" }}>
        <p style={{ color: "#A781FF", letterSpacing: 3, fontSize: 12 }}>
          MAHFEL · CHARACTER STUDIES
        </p>
        <h1 style={{ fontSize: 40, margin: "8px 0" }}>A little light. A curious mind.</h1>
        <p style={{ color: "#A69BB7", marginBottom: 32 }}>
          پرتو / Parto — six procedural interpretations of the same ribbon.
        </p>
        <div
          style={{
            display: "grid",
            gridTemplateColumns: "repeat(auto-fit, minmax(270px, 1fr))",
            gap: 16,
          }}
        >
          {designs.map((design, i) => (
            <section
              key={design}
              style={{
                border: "1px solid #30283D",
                borderRadius: 20,
                background: "#1D1729",
                padding: 24,
              }}
            >
              <div style={{ display: "flex", justifyContent: "center" }}>
                <Parto {...args} size={256} design={design} />
              </div>
              <h2 style={{ textTransform: "capitalize", fontSize: 18 }}>{design}</h2>
              <p style={{ fontSize: 13, color: "#A69BB7" }}>{descriptions[i]}</p>
            </section>
          ))}
        </div>
      </div>
    </main>
  ),
};
export const AvatarSizes: Story = {
  args: { emotion: "happy", happiness: 0.85, energy: 0.65, interactive: true },
  render: (args) => (
    <div style={{ display: "flex", flexDirection: "column", gap: 24 }}>
      {["#FAF7FF", "#191322"].map((background) => (
        <div
          key={background}
          style={{
            background,
            borderRadius: 16,
            padding: 24,
            display: "flex",
            flexWrap: "wrap",
            alignItems: "center",
            gap: 24,
          }}
        >
          {[32, 48, 64, 128, 256].map((size) => (
            <div key={size} style={{ textAlign: "center", color: "#A781FF" }}>
              <Parto {...args} size={size} design="minimal" />
              <small>{size}px</small>
            </div>
          ))}
        </div>
      ))}
    </div>
  ),
};
export const ReferenceComparison: Story = {
  render: (args) => (
    <div
      style={{
        display: "flex",
        flexWrap: "wrap",
        gap: 32,
        alignItems: "center",
        padding: 32,
        background: "#F6F1FF",
        borderRadius: 20,
      }}
    >
      <div>
        <div
          role="img"
          aria-label="Original Parto reference, frame 01"
          style={{
            width: 240,
            height: 200,
            backgroundImage: "url(/references/parto.png)",
            backgroundPosition: "-27px -150px",
            backgroundSize: "1536px 1024px",
            borderRadius: 12,
          }}
        />
        <p>Reference · Frame 01</p>
      </div>
      <div>
        <Parto {...args} size={240} />
        <p>Procedural reconstruction</p>
      </div>
    </div>
  ),
};
