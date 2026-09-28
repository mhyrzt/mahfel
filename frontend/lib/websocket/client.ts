"use client";

import axios from "axios";

import { publicApiBaseUrl } from "@/lib/api/config";
import { browserApiClient } from "@/lib/api/browser";
import type { ClientMessage, ServerMessage } from "./generated/messages";

interface WebSocketTicketResponse {
  ticket: string;
  expires_in: number;
}

const INITIAL_RECONNECT_DELAY_MS = 500;
const MAX_RECONNECT_DELAY_MS = 15_000;
const AUTH_TIMEOUT_MS = 10_000;
const REQUEST_TIMEOUT_MS = 12_000;

export type SignalingStatus =
  "connecting" | "connected" | "reconnecting" | "unauthorized" | "unavailable" | "closed";

export class SignalingClient {
  private socket: WebSocket | null = null;
  private reconnectTimer: ReturnType<typeof setTimeout> | null = null;
  private reconnectAttempt = 0;
  private stopped = false;
  private hasConnected = false;
  private pending = new Map<
    string,
    {
      resolve: (message: ServerMessage) => void;
      reject: (error: Error) => void;
      timer: ReturnType<typeof setTimeout>;
    }
  >();

  constructor(
    private readonly sessionId: string | null,
    private readonly onMessage: (message: ServerMessage) => void,
    private readonly onStatus: (status: SignalingStatus) => void = () => undefined,
  ) {}

  connect(): void {
    if (this.socket && this.socket.readyState < WebSocket.CLOSING) {
      return;
    }
    if (this.stopped) {
      this.stopped = false;
    }
    void this.open(true);
  }

  send(message: ClientMessage): void {
    if (!this.socket || this.socket.readyState !== WebSocket.OPEN) {
      throw new Error("WebSocket is not connected.");
    }
    this.socket.send(JSON.stringify({ v: 1, ...message }));
  }

  request(message: ClientMessage): Promise<ServerMessage> {
    const id = crypto.randomUUID();
    return new Promise((resolve, reject) => {
      const timer = setTimeout(() => {
        this.pending.delete(id);
        reject(new Error("The media request timed out."));
      }, REQUEST_TIMEOUT_MS);
      this.pending.set(id, { resolve, reject, timer });
      try {
        this.send({ ...message, id });
      } catch (error) {
        clearTimeout(timer);
        this.pending.delete(id);
        reject(
          error instanceof Error ? error : new Error("The session connection is unavailable."),
        );
      }
    });
  }

  leave(): void {
    if (this.socket?.readyState === WebSocket.OPEN) {
      this.socket.send(JSON.stringify({ v: 1, type: "leave" }));
    }
  }

  close(): void {
    this.stopped = true;
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer);
    }
    this.reconnectTimer = null;
    this.socket?.close(1000, "client closed");
    this.socket = null;
    this.rejectPending();
    this.onStatus("closed");
  }

  private async open(initial: boolean): Promise<void> {
    if (this.stopped) {
      return;
    }
    this.onStatus(initial ? "connecting" : "reconnecting");

    try {
      const { data } = await browserApiClient.post<WebSocketTicketResponse>("/v1/ws/tickets");
      if (this.stopped) {
        return;
      }

      const socket = new WebSocket(webSocketUrl());
      this.socket = socket;
      let welcomed = false;
      const authTimeout = setTimeout(() => {
        if (!welcomed) {
          socket.close(4401, "authentication timeout");
        }
      }, AUTH_TIMEOUT_MS);

      socket.onopen = () => {
        socket.send(JSON.stringify({ v: 1, type: "auth", ticket: data.ticket }));
      };
      socket.onmessage = (event: MessageEvent<string>) => {
        const message = decodeServerMessage(event.data);
        if (!message) {
          return;
        }
        if (message.id) {
          const pending = this.pending.get(message.id);
          if (pending) {
            clearTimeout(pending.timer);
            this.pending.delete(message.id);
            if (message.type === "error") {
              pending.reject(new Error(message.message));
            } else {
              pending.resolve(message);
            }
          }
        }
        if (message.type === "welcome" && !welcomed) {
          welcomed = true;
          clearTimeout(authTimeout);
          this.hasConnected = true;
          this.reconnectAttempt = 0;
          this.onStatus("connected");
          if (this.sessionId) {
            socket.send(JSON.stringify({ v: 1, type: "join", session_id: this.sessionId }));
          }
        }
        this.onMessage(message);
      };
      socket.onerror = () => {
        if (!welcomed) {
          socket.close();
        }
      };
      socket.onclose = (event: CloseEvent) => {
        this.rejectPending();
        clearTimeout(authTimeout);
        if (this.socket === socket) {
          this.socket = null;
        }
        if (this.stopped) {
          return;
        }
        if (event.code === 4401) {
          this.stopped = true;
          this.onStatus("unauthorized");
          return;
        }
        if (!this.hasConnected) {
          this.scheduleReconnect();
          return;
        }
        this.scheduleReconnect();
      };
    } catch (error) {
      if (this.stopped) {
        return;
      }
      if (axios.isAxiosError(error) && [401, 403].includes(error.response?.status ?? 0)) {
        this.onStatus("unauthorized");
        return;
      }
      if (initial && !this.hasConnected) {
        this.onStatus("unavailable");
      }
      this.scheduleReconnect();
    }
  }

  private scheduleReconnect(): void {
    if (this.stopped || this.reconnectTimer) {
      return;
    }
    this.onStatus("reconnecting");
    const delay = Math.min(
      INITIAL_RECONNECT_DELAY_MS * 2 ** this.reconnectAttempt,
      MAX_RECONNECT_DELAY_MS,
    );
    this.reconnectAttempt += 1;
    this.reconnectTimer = setTimeout(() => {
      this.reconnectTimer = null;
      void this.open(false);
    }, delay);
  }

  private rejectPending(): void {
    for (const pending of this.pending.values()) {
      clearTimeout(pending.timer);
      pending.reject(new Error("The session connection was interrupted."));
    }
    this.pending.clear();
  }
}

function webSocketUrl(): string {
  const url = new URL("/v1/ws", publicApiBaseUrl);
  url.protocol = url.protocol === "https:" ? "wss:" : "ws:";
  return url.toString();
}

function decodeServerMessage(data: unknown): ServerMessage | null {
  if (typeof data !== "string") {
    return null;
  }
  try {
    const value: unknown = JSON.parse(data);
    if (typeof value === "object" && value !== null && "type" in value) {
      return value as ServerMessage;
    }
  } catch {}
  return null;
}
