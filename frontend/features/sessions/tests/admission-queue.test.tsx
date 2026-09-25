import { fireEvent, render, screen } from "@testing-library/react";
import { expect, test, vi } from "vitest";

import { AdmissionQueue } from "../components/admission-queue";

const request = {
  id: "request-1",
  actorId: "actor-1",
  name: "Roya",
  initials: "R",
  requestedAt: "2026-09-25T00:00:00Z",
};

test("routes admit and reject actions to the selected waiting principal", () => {
  const onAdmit = vi.fn();
  const onReject = vi.fn();
  render(<AdmissionQueue requests={[request]} locale="en" onAdmit={onAdmit} onReject={onReject} />);

  fireEvent.click(screen.getByRole("button", { name: "Admit Roya" }));
  fireEvent.click(screen.getByRole("button", { name: "Reject Roya" }));

  expect(onAdmit).toHaveBeenCalledWith(request);
  expect(onReject).toHaveBeenCalledWith(request);
});

test("does not expose a working admission control without an actor id", () => {
  render(<AdmissionQueue requests={[{ ...request, actorId: undefined }]} locale="en" />);
  expect(screen.getByRole("button", { name: "Admit Roya" })).toBeDisabled();
});
