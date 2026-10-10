import type { Provider } from "@lexical/yjs";
import axios from "axios";
import * as awarenessProtocol from "y-protocols/awareness";
import type * as Y from "yjs";

import { browserApiClient } from "@/lib/api/browser";
import { publicApiBaseUrl } from "@/lib/api/config";

import { awarenessFrame, receiveFrame, syncStepOne, updateFrame } from "./protocol";

type Status = "connecting" | "connected" | "reconnecting" | "unauthorized" | "closed";
type EventName = "sync" | "status" | "update" | "reload";
type Listener = (...args: never[]) => void;

interface TicketResponse {
  ticket: string;
}

interface ReadyFrame {
  type: "collaboration_ready";
  can_edit: boolean;
  live: boolean;
}

export interface CollaborationEventFrame {
  type: "collaboration_event";
  sequence: number;
  kind: string;
  data: Record<string, unknown>;
}

export class MahfelCollaborationProvider implements Provider {
  readonly awareness: Provider["awareness"];
  private readonly rawAwareness: awarenessProtocol.Awareness;
  private socket: WebSocket | null = null;
  private timer: ReturnType<typeof setTimeout> | null = null;
  private stopped = true;
  private generation = 0;
  private authTimer: ReturnType<typeof setTimeout> | null = null;
  private attempt = 0;
  private synced = false;
  private serverCanEdit = false;
  private listeners = new Map<EventName, Set<Listener>>();

  constructor(
    readonly sessionId: string,
    readonly kind: "notes" | "whiteboard",
    readonly doc: Y.Doc,
    private readonly onPermission: (canEdit: boolean) => void,
    private readonly onEvent: (event: CollaborationEventFrame) => void,
  ) {
    this.rawAwareness = new awarenessProtocol.Awareness(doc);
    this.awareness = this.rawAwareness as unknown as Provider["awareness"];
    doc.on("update", this.onDocumentUpdate);
    this.rawAwareness.on("update", this.onAwarenessUpdate);
  }

  on(type: "sync", cb: (isSynced: boolean) => void): void;
  on(type: "status", cb: (value: { status: string }) => void): void;
  on(type: "update", cb: (value: unknown) => void): void;
  on(type: "reload", cb: (doc: Y.Doc) => void): void;
  on(type: EventName, cb: Listener): void {
    const listeners = this.listeners.get(type) ?? new Set<Listener>();
    listeners.add(cb);
    this.listeners.set(type, listeners);
  }

  off(type: "sync", cb: (isSynced: boolean) => void): void;
  off(type: "status", cb: (value: { status: string }) => void): void;
  off(type: "update", cb: (value: unknown) => void): void;
  off(type: "reload", cb: (doc: Y.Doc) => void): void;
  off(type: EventName, cb: Listener): void {
    this.listeners.get(type)?.delete(cb);
  }

  connect(): void {
    if (!this.stopped || this.socket) {
      return;
    }
    this.stopped = false;
    void this.open();
  }

  disconnect(): void {
    this.stopped = true;
    this.generation += 1;
    if (this.authTimer) {
      clearTimeout(this.authTimer);
    }
    this.authTimer = null;
    if (this.timer) {
      clearTimeout(this.timer);
    }
    this.timer = null;
    this.rawAwareness.setLocalState(null);
    this.socket?.close();
    this.socket = null;
    this.synced = false;
    this.emit("sync", false);
    this.emit("status", { status: "closed" satisfies Status });
  }

  destroy(): void {
    this.disconnect();
    this.doc.off("update", this.onDocumentUpdate);
    this.rawAwareness.off("update", this.onAwarenessUpdate);
    this.rawAwareness.destroy();
    this.listeners.clear();
  }

  private emit(type: EventName, value: unknown): void {
    this.listeners.get(type)?.forEach((listener) => listener(value as never));
  }

  private send(bytes: Uint8Array): void {
    if (this.socket?.readyState === WebSocket.OPEN) {
      this.socket.send(bytes);
    }
  }

  private readonly onDocumentUpdate = (update: Uint8Array, origin: unknown): void => {
    if (origin !== this) {
      this.send(updateFrame(update));
    }
  };

  private readonly onAwarenessUpdate = (
    {
      added,
      updated,
      removed,
    }: {
      added: number[];
      updated: number[];
      removed: number[];
    },
    origin: unknown,
  ): void => {
    if (origin !== this) {
      this.send(awarenessFrame(this.rawAwareness, [...added, ...updated, ...removed]));
    }
  };

  private async open(): Promise<void> {
    if (this.stopped) {
      return;
    }
    const generation = ++this.generation;
    this.emit("status", {
      status: this.attempt ? "reconnecting" : ("connecting" satisfies Status),
    });
    try {
      const { data } = await browserApiClient.post<TicketResponse>("/v1/ws/tickets");
      if (this.stopped || generation !== this.generation) {
        return;
      }
      const url = new URL(
        `/v1/ws/collaboration/${encodeURIComponent(this.sessionId)}/${this.kind}`,
        publicApiBaseUrl,
      );
      url.protocol = url.protocol === "https:" ? "wss:" : "ws:";
      const socket = new WebSocket(url);
      socket.binaryType = "arraybuffer";
      this.socket = socket;
      const authTimer = setTimeout(() => socket.close(4401, "authentication timeout"), 10_000);
      this.authTimer = authTimer;
      socket.onopen = () => {
        if (generation === this.generation && !this.stopped) {
          socket.send(JSON.stringify({ type: "auth", ticket: data.ticket }));
        }
      };
      socket.onmessage = (message) => {
        if (generation !== this.generation || this.stopped) {
          return;
        }
        if (typeof message.data === "string") {
          let frame: unknown;
          try {
            frame = JSON.parse(message.data);
          } catch {
            socket.close(1002, "invalid collaboration frame");
            return;
          }
          if (isReady(frame)) {
            clearTimeout(authTimer);
            this.authTimer = null;
            this.attempt = 0;
            if (this.serverCanEdit && !frame.can_edit) {
              window.location.reload();
              return;
            }
            this.serverCanEdit = frame.can_edit && frame.live;
            this.onPermission(frame.can_edit && frame.live);
            this.emit("status", { status: "connected" satisfies Status });
            this.send(syncStepOne(this.doc));
            this.send(awarenessFrame(this.rawAwareness, [this.doc.clientID]));
          } else if (isPermission(frame)) {
            if (this.serverCanEdit && !frame.can_edit) {
              window.location.reload();
              return;
            }
            this.serverCanEdit = frame.can_edit;
            this.onPermission(frame.can_edit);
          } else if (isEvent(frame)) {
            this.onEvent(frame);
          }
          return;
        }
        if (!(message.data instanceof ArrayBuffer)) {
          return;
        }
        try {
          const result = receiveFrame(
            new Uint8Array(message.data),
            this.doc,
            this.rawAwareness,
            this,
          );
          if (result.reply) {
            this.send(result.reply);
          }
          if (result.synced && !this.synced) {
            this.synced = true;
            this.emit("sync", true);
          }
        } catch {
          socket.close();
        }
      };
      socket.onclose = (event) => {
        clearTimeout(authTimer);
        if (generation !== this.generation) {
          return;
        }
        this.authTimer = null;
        if (this.socket === socket) {
          this.socket = null;
        }
        if (!this.stopped && event.code === 4401) {
          this.stopped = true;
          this.onPermission(false);
          this.emit("status", { status: "unauthorized" satisfies Status });
          return;
        }
        if (!this.stopped) {
          this.reconnect();
        }
      };
    } catch (error) {
      if (this.stopped || generation !== this.generation) {
        return;
      }
      if (axios.isAxiosError(error) && [401, 403].includes(error.response?.status ?? 0)) {
        this.emit("status", { status: "unauthorized" satisfies Status });
        this.stopped = true;
        return;
      }
      this.reconnect();
    }
  }

  private reconnect(): void {
    if (this.timer) {
      return;
    }
    this.onPermission(false);
    this.synced = false;
    this.emit("sync", false);
    this.emit("status", { status: "reconnecting" satisfies Status });
    const delay = Math.min(500 * 2 ** this.attempt, 15_000);
    this.attempt += 1;
    this.timer = setTimeout(() => {
      this.timer = null;
      void this.open();
    }, delay);
  }
}

function isReady(value: unknown): value is ReadyFrame {
  return (
    typeof value === "object" &&
    value !== null &&
    "type" in value &&
    value.type === "collaboration_ready" &&
    "can_edit" in value &&
    typeof value.can_edit === "boolean" &&
    "live" in value &&
    typeof value.live === "boolean"
  );
}

function isPermission(value: unknown): value is { type: "permission_changed"; can_edit: boolean } {
  return (
    typeof value === "object" &&
    value !== null &&
    "type" in value &&
    value.type === "permission_changed" &&
    "can_edit" in value &&
    typeof value.can_edit === "boolean"
  );
}

function isEvent(value: unknown): value is CollaborationEventFrame {
  return (
    typeof value === "object" &&
    value !== null &&
    "type" in value &&
    value.type === "collaboration_event" &&
    "sequence" in value &&
    typeof value.sequence === "number" &&
    "kind" in value &&
    typeof value.kind === "string" &&
    "data" in value &&
    typeof value.data === "object" &&
    value.data !== null
  );
}
