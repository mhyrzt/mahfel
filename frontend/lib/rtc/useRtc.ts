"use client";

import { useCallback, useState } from "react";
import { createDevice, isDeviceSupported } from "./device";

export type PermissionState = "unknown" | "granted" | "denied" | "unsupported";

export function useRtc() {
  const [permission, setPermission] = useState<PermissionState>("unknown");
  const [error, setError] = useState<string | null>(null);
  const [deviceReady, setDeviceReady] = useState(false);

  const initDevice = useCallback(async (routerRtpCapabilities: unknown) => {
    if (!isDeviceSupported()) {
      setPermission("unsupported");
      setError("Browser does not support WebRTC");
      return null;
    }
    try {
      const device = await createDevice(routerRtpCapabilities);
      setDeviceReady(true);
      setPermission("granted");
      setError(null);
      return device;
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to load device");
      setPermission("denied");
      return null;
    }
  }, []);

  const requestMedia = useCallback(async (constraints: MediaStreamConstraints) => {
    try {
      const stream = await navigator.mediaDevices.getUserMedia(constraints);
      setPermission("granted");
      setError(null);
      return stream;
    } catch (err) {
      let message = "Failed to access media devices";
      if (err instanceof DOMException && err.name === "NotAllowedError") {
        message =
          "Microphone/camera permission denied. Allow access in browser settings and retry.";
      } else if (err instanceof Error) {
        message = err.message;
      }
      setPermission("denied");
      setError(message);
      return null;
    }
  }, []);

  const requestDisplay = useCallback(async () => {
    try {
      const stream = await navigator.mediaDevices.getDisplayMedia({
        video: true,
        audio: false,
      });
      const [track] = stream.getVideoTracks();
      track.addEventListener("ended", () => {
        stream.getTracks().forEach((t) => t.stop());
      });
      setError(null);
      return stream;
    } catch (err) {
      let message = "Failed to start screen share";
      if (err instanceof DOMException && err.name === "NotAllowedError") {
        message = "Screen share permission denied. Allow and retry.";
      } else if (err instanceof Error) {
        message = err.message;
      }
      setError(message);
      return null;
    }
  }, []);

  return { permission, error, deviceReady, initDevice, requestMedia, requestDisplay };
}
