import { expect, test } from "vitest";

import { isAllowedBackendPath } from "../allowlist";

const session = "89c42e43-48fa-4952-8507-b9ca3e14b001";
const actor = "c1fa6ded-3fb1-4e5a-bbb9-25168f5b189d";

test("allows the authenticated session routes used by the workspace", () => {
  expect(isAllowedBackendPath(`v1/sessions/${session}/start`)).toBe(true);
  expect(isAllowedBackendPath(`v1/sessions/${session}/lobby/${actor}/admit`)).toBe(true);
  expect(isAllowedBackendPath(`v1/sessions/${session}/collaboration/grants/${actor}`)).toBe(true);
  expect(isAllowedBackendPath(`v1/sessions/${session}/collaboration/notes/images`)).toBe(true);
  expect(isAllowedBackendPath(`v1/sessions/${session}/collaboration/whiteboard/images`)).toBe(true);
  expect(isAllowedBackendPath("v1/invite-links/redeem")).toBe(true);
});

test("does not forward arbitrary session or private operations paths", () => {
  expect(isAllowedBackendPath(`v1/sessions/${session}/internal`)).toBe(false);
  expect(isAllowedBackendPath(`v1/sessions/${session}/collaboration/recording/images`)).toBe(false);
  expect(isAllowedBackendPath("v1/admin/service-principals")).toBe(false);
  expect(isAllowedBackendPath("v1/ws")).toBe(false);
});
