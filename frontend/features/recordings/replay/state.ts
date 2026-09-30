import * as Y from "yjs";
import {
  integer,
  nanoseconds,
  row,
  text,
  type Presentation,
  type ReplayInput,
  type Row,
  type Surface,
  type Toast,
} from "./types";

export function postgresBytes(value: unknown): Uint8Array {
  const encoded = text(value);
  if (!/^\\x(?:[0-9a-fA-F]{2})*$/.test(encoded)) {
    throw new Error("Invalid recorded document bytes");
  }
  return Uint8Array.from(encoded.slice(2).match(/../g) ?? [], (item) => Number.parseInt(item, 16));
}

export class ReplayState {
  readonly documents = new Map<
    string,
    { kind: "notes" | "whiteboard"; doc: Y.Doc; revision: number }
  >();
  readonly viewports = new Map<Surface, Row>();
  readonly toasts: Toast[] = [];
  presentation: Presentation | null = null;
  surface: Surface | null = null;
  generation: number | null = null;
  slate: "paused" | "outage" | null = null;
  private cursor = 0;
  private lastNs = -BigInt("1");
  private workspaceSurface: "notes" | "whiteboard" | null = null;
  private readonly events;

  constructor(readonly input: ReplayInput) {
    this.events = [...input.events].sort((a, b) => {
      const delta = nanoseconds(a.at_ns) - nanoseconds(b.at_ns);
      if (delta !== BigInt("0")) {
        return delta < BigInt("0") ? -1 : 1;
      }
      const sequenceA = Number(a.payload.event_sequence ?? a.payload.event_seq ?? 0);
      const sequenceB = Number(b.payload.event_sequence ?? b.payload.event_seq ?? 0);
      return sequenceA - sequenceB || a.source_id.localeCompare(b.source_id);
    });
  }

  advance(outputNs: bigint): void {
    if (outputNs < BigInt("0") || outputNs > nanoseconds(this.input.duration_ns)) {
      throw new Error("Replay timestamp outside recording");
    }
    const atNs = nanoseconds(this.input.start_ns) + outputNs;
    if (atNs < this.lastNs) {
      this.clear();
      this.cursor = 0;
      this.generation = null;
    }
    this.lastNs = atNs;
    const interval = this.input.intervals.find(
      (item) => nanoseconds(item.start_ns) <= atNs && atNs < nanoseconds(item.end_ns),
    );
    if (!interval) {
      const previous = [...this.input.intervals]
        .reverse()
        .find((item) => nanoseconds(item.end_ns) <= atNs);
      this.slate = !previous || previous.loss_reason ? "outage" : "paused";
      this.toasts.splice(0);
      return;
    }
    this.slate = null;
    if (interval.generation !== this.generation) {
      this.clear();
      this.generation = interval.generation;
      const snapshots = this.input.snapshots.filter(
        (item) => item.generation === interval.generation,
      );
      for (const snapshot of snapshots.filter((item) => item.snapshot_kind === "document")) {
        if (
          snapshot.payload.schema_version !== undefined &&
          snapshot.payload.schema_version !== 1
        ) {
          throw new Error("Unsupported recorded document schema");
        }
        const kind = snapshot.payload.kind;
        if (kind === "notes" || kind === "whiteboard") {
          const document: { kind: "notes" | "whiteboard"; doc: Y.Doc; revision: number } = {
            kind,
            doc: new Y.Doc(),
            revision: 0,
          };
          document.doc.on("update", () => {
            document.revision += 1;
          });
          this.documents.set(snapshot.source_id, document);
        }
      }
      for (const snapshot of snapshots.filter((item) => item.snapshot_kind === "checkpoint")) {
        this.applyUpdate(snapshot.payload);
      }
      for (const snapshot of snapshots
        .filter((item) => item.snapshot_kind === "document_update")
        .sort((a, b) => Number(a.payload.event_sequence) - Number(b.payload.event_sequence))) {
        this.applyUpdate(snapshot.payload);
      }
      for (const snapshot of snapshots
        .filter((item) => item.snapshot_kind === "viewport")
        .sort((a, b) => Number(a.payload.event_sequence) - Number(b.payload.event_sequence))) {
        this.viewport(row(snapshot.payload.payload));
      }
      const presentation = snapshots.find((item) => item.snapshot_kind === "presentation");
      if (presentation?.payload.selection_event_id) {
        const assets = snapshots
          .filter((item) => item.snapshot_kind === "presentation_asset")
          .sort((a, b) => Number(a.payload.ordinal) - Number(b.payload.ordinal));
        this.presentation = {
          fileIds: assets.map((item) => text(item.payload.file_id)),
          page: Number(presentation.payload.current_page ?? 1),
          positionMs: Number(presentation.payload.playback_position_ms ?? 0),
          paused: presentation.payload.playback_paused !== false,
          rate: Number(presentation.payload.playback_rate ?? 1),
          anchorNs:
            typeof presentation.payload.playback_anchor_ns === "string"
              ? nanoseconds(presentation.payload.playback_anchor_ns)
              : nanoseconds(interval.start_ns),
        };
        this.surface = "presentation";
      }
      if (!this.surface) {
        if (this.document("whiteboard")?.getMap("elements").size) {
          this.surface = "whiteboard";
        } else if (this.document("notes")?.get("root", Y.XmlText).length) {
          this.surface = "notes";
        }
      }
    }
    if (!this.workspaceSurface && (this.surface === "notes" || this.surface === "whiteboard")) {
      this.workspaceSurface = this.surface;
    }
    while (
      this.cursor < this.events.length &&
      nanoseconds(this.events[this.cursor].at_ns) <= atNs
    ) {
      const event = this.events[this.cursor++];
      if (event.generation !== this.generation) {
        continue;
      }
      if (event.event_kind === "collaboration") {
        const payload = event.payload;
        if (payload.kind === "document_update") {
          this.applyUpdate(payload);
          const document = this.documents.get(text(payload.document_id));
          if (document) {
            this.surface = document.kind;
            this.workspaceSurface = document.kind;
          }
        } else if (payload.kind === "viewport") {
          this.viewport(row(payload.payload));
        } else if (payload.kind === "presentation") {
          this.applyPresentation(row(payload.payload), nanoseconds(event.at_ns));
        }
      } else if (event.event_kind === "chat") {
        this.chat(event.payload, nanoseconds(event.at_ns));
      }
    }
    for (let i = this.toasts.length - 1; i >= 0; i--) {
      if (this.toasts[i].expiresNs <= atNs) {
        this.toasts.splice(i, 1);
      }
    }
  }

  document(kind: "notes" | "whiteboard"): Y.Doc | undefined {
    return [...this.documents.values()].find((item) => item.kind === kind)?.doc;
  }
  presentationPosition(atNs: bigint): number {
    const state = this.presentation;
    return state
      ? Math.max(
          0,
          state.positionMs +
            (state.paused ? 0 : (Number(atNs - state.anchorNs) / 1000000) * state.rate),
        )
      : 0;
  }
  destroy(): void {
    this.clear();
  }
  private clear(): void {
    for (const value of this.documents.values()) {
      value.doc.destroy();
    }
    this.documents.clear();
    this.viewports.clear();
    this.toasts.splice(0);
    this.presentation = null;
    this.surface = null;
    this.workspaceSurface = null;
  }
  private applyUpdate(payload: Row): void {
    const document = this.documents.get(text(payload.document_id));
    if (!document) {
      throw new Error("Recording document missing");
    }
    Y.applyUpdate(document.doc, postgresBytes(payload.update_bytes));
  }
  private viewport(payload: Row): void {
    if (payload.surface === "notes" || payload.surface === "whiteboard") {
      this.viewports.set(payload.surface, payload);
      this.surface = payload.surface;
      this.workspaceSurface = payload.surface;
    }
  }
  private applyPresentation(payload: Row, atNs: bigint): void {
    if (payload.action === "stop") {
      this.presentation = null;
      this.surface = this.workspaceSurface;
      return;
    }
    if (payload.action === "select") {
      if (!Array.isArray(payload.file_ids)) {
        throw new Error("Invalid recorded presentation");
      }
      this.presentation = {
        fileIds: payload.file_ids.map(text),
        page: 1,
        positionMs: 0,
        paused: true,
        rate: 1,
        anchorNs: atNs,
      };
    } else if (this.presentation && payload.action === "page") {
      this.presentation.page = integer(payload.page);
    } else if (this.presentation && payload.action === "playback") {
      this.presentation.positionMs = integer(payload.position_ms);
      this.presentation.paused = payload.paused === true;
      this.presentation.rate = Number(payload.rate);
      this.presentation.anchorNs = atNs;
    }
    if (this.presentation) {
      this.surface = "presentation";
    }
  }
  private chat(payload: Row, atNs: bigint): void {
    const id = text(payload.message_id);
    const existing = this.toasts.findIndex((item) => item.id === id);
    if (payload.kind === "deleted") {
      if (existing >= 0) {
        this.toasts.splice(existing, 1);
      }
    } else if (payload.kind === "edited") {
      if (existing >= 0) {
        this.toasts[existing].body = text(payload.body);
      }
    } else if (payload.kind === "created") {
      this.toasts.push({
        id,
        author: this.input.actors[text(payload.author_actor_id)] ?? "Participant",
        body: text(payload.body),
        expiresNs: atNs + BigInt("6000000000"),
      });
      while (this.toasts.length > 3) {
        this.toasts.shift();
      }
    }
  }
}
