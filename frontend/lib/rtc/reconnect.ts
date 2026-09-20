"use client";

export type ConnectionState =
  "new" | "connecting" | "connected" | "failed" | "disconnected" | "closed";

export interface ReconnectController {
  onDisconnected: () => void;
  onFailed: () => void;
  reset: () => void;
}

export function createReconnectController(onRejoin: () => Promise<void>): ReconnectController {
  let disconnectTimer: ReturnType<typeof setTimeout> | null = null;
  let retries = 0;

  const clear = () => {
    if (disconnectTimer) {
      clearTimeout(disconnectTimer);
      disconnectTimer = null;
    }
  };

  return {
    onDisconnected: () => {
      clear();
      disconnectTimer = setTimeout(async () => {
        if (retries < 3) {
          retries += 1;
          try {
            await onRejoin();
            retries = 0;
          } catch {}
        }
      }, 5000);
    },
    onFailed: () => {
      clear();
      void onRejoin().catch(() => {});
    },
    reset: () => {
      clear();
      retries = 0;
    },
  };
}
