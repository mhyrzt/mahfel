export interface PlaybackState {
  positionMs: number;
  paused: boolean;
  rate: number;
}

export const DRIFT_THRESHOLD_MS = 1500;

export function shouldCorrectDrift(
  localMs: number,
  remoteMs: number,
  thresholdMs = DRIFT_THRESHOLD_MS,
): boolean {
  return Math.abs(localMs - remoteMs) > thresholdMs;
}
