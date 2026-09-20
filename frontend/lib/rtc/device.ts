"use client";

import { Device, detectDevice } from "mediasoup-client";

export type RtcDevice = Device;

export async function createDevice(routerRtpCapabilities: unknown): Promise<Device> {
  const device = new Device();
  await device.load({ routerRtpCapabilities: routerRtpCapabilities as never });
  return device;
}

export function isDeviceSupported(): boolean {
  return typeof navigator !== "undefined" && detectDevice() !== undefined;
}
