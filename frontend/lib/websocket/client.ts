"use client";

export type WsRtcMessage =
  | { type: "createTransport"; room_id: string; id?: string }
  | {
      type: "connectTransport";
      room_id: string;
      transport_id: string;
      dtls_parameters: string;
      id?: string;
    }
  | {
      type: "produce";
      room_id: string;
      transport_id: string;
      kind: string;
      rtp_parameters: string;
      id?: string;
    }
  | {
      type: "consume";
      room_id: string;
      transport_id: string;
      producer_id: string;
      rtp_capabilities: string;
      id?: string;
    };

export class RtcWsClient {
  private ws: WebSocket | null = null;
  private pending = new Map<string, (data: unknown) => void>();

  constructor(private url: string) {}

  connect(token?: string): Promise<void> {
    return new Promise((resolve, reject) => {
      const ws = new WebSocket(this.url);
      ws.onopen = () => {
        if (token) {
          ws.send(JSON.stringify({ v: 1, type: "auth", token }));
        }
        this.ws = ws;
        ws.onmessage = (ev) => this.handleMessage(ev.data as string);
        resolve();
      };
      ws.onerror = () => reject(new Error("WebSocket failed"));
    });
  }

  private handleMessage(data: string): void {
    try {
      const msg = JSON.parse(data) as { id?: string; type: string };
      if (msg.id && this.pending.has(msg.id)) {
        const cb = this.pending.get(msg.id);
        if (cb) {
          cb(msg);
        }
        this.pending.delete(msg.id);
      }
    } catch {}
  }

  sendWithId(msg: WsRtcMessage): Promise<unknown> {
    const id = Math.random().toString(36).slice(2);
    const payload = { v: 1, id, ...msg };
    return new Promise((resolve) => {
      this.pending.set(id, resolve);
      this.ws?.send(JSON.stringify(payload));
      setTimeout(() => {
        if (this.pending.has(id)) {
          this.pending.delete(id);
          resolve(null);
        }
      }, 5000);
    });
  }

  close(): void {
    this.ws?.close();
    this.ws = null;
  }
}
