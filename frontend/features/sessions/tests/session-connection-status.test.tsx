import { fireEvent, render, screen, within } from "@testing-library/react";
import { expect, test } from "vitest";

import { SessionConnectionStatus } from "../components/session-connection-status";
import { emptyConnectionMetrics } from "@/lib/rtc/connection-stats";

test("Persian metric units stay ms and numeric values retain LTR order", () => {
  render(
    <SessionConnectionStatus
      locale="fa"
      connection={{
        signaling: "connected",
        offline: false,
        media: "connected",
        metrics: { latencyMs: 42, jitterMs: 12, packetLossPercent: 0.5 },
      }}
    />,
  );
  fireEvent.click(screen.getByRole("button", { name: "جزئیات اتصال: خوب" }));
  const card = screen.getByRole("region", { name: "جزئیات اتصال" });
  expect(within(card).getByText("۴۲ ms")).toHaveAttribute("dir", "ltr");
  expect(within(card).getByText("۱۲ ms")).toHaveAttribute("dir", "ltr");
  expect(within(card).queryByText(/میلی‌ثانیه/)).not.toBeInTheDocument();
});

test("keyboard focus opens details and Escape closes them", async () => {
  render(
    <SessionConnectionStatus
      locale="en"
      connection={{
        signaling: "reconnecting",
        offline: false,
        media: "idle",
        metrics: emptyConnectionMetrics,
      }}
    />,
  );
  const trigger = screen.getByRole("button", { name: "Connection details: Reconnecting" });
  fireEvent.focus(trigger);
  expect(await screen.findByRole("region", { name: "Connection details" })).toBeInTheDocument();
  fireEvent.keyDown(trigger, { key: "Escape" });
  expect(screen.queryByRole("region")).not.toBeInTheDocument();
  expect(trigger).toHaveAttribute("aria-expanded", "false");
});

test("opens real measurements and an estimated quality explanation", () => {
  render(
    <SessionConnectionStatus
      locale="en"
      connection={{
        signaling: "connected",
        offline: false,
        media: "connected",
        metrics: { latencyMs: 42, jitterMs: 12, packetLossPercent: 0.5 },
      }}
    />,
  );
  fireEvent.click(screen.getByRole("button", { name: "Connection details: Good" }));
  const dialog = screen.getByRole("region", { name: "Connection details" });
  expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
  expect(within(dialog).getByText("42 ms")).toBeInTheDocument();
  expect(within(dialog).getByText("12 ms")).toBeInTheDocument();
  expect(within(dialog).getByText("0.5%")).toBeInTheDocument();
  expect(within(dialog).getByText(/Quality is an estimate/)).toBeInTheDocument();
});

test("idle media does not show invented quality or zero ping", () => {
  render(
    <SessionConnectionStatus
      locale="en"
      connection={{
        signaling: "connected",
        offline: false,
        media: "idle",
        metrics: emptyConnectionMetrics,
      }}
    />,
  );
  fireEvent.click(screen.getByRole("button", { name: "Connection details: Connected" }));
  const dialog = screen.getByRole("region", { name: "Connection details" });
  expect(within(dialog).getByText("Not measured")).toBeInTheDocument();
  expect(within(dialog).getAllByText("Not available")).toHaveLength(3);
  expect(within(dialog).getByText(/when you send or receive/)).toBeInTheDocument();
});

test("offline Persian state takes precedence over previously good measurements", () => {
  render(
    <SessionConnectionStatus
      locale="fa"
      connection={{
        signaling: "connected",
        offline: true,
        media: "connected",
        metrics: { latencyMs: 42, jitterMs: 12, packetLossPercent: 0 },
      }}
    />,
  );
  fireEvent.click(screen.getByRole("button", { name: "جزئیات اتصال: آفلاین" }));
  const dialog = screen.getByRole("region", { name: "جزئیات اتصال" });
  expect(dialog).toHaveAttribute("dir", "rtl");
  expect(within(dialog).getAllByText("آفلاین").length).toBeGreaterThan(0);
  expect(within(dialog).getByText("اندازه‌گیری نشده")).toBeInTheDocument();
});
