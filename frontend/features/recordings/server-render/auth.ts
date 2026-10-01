import { createHash, timingSafeEqual } from "node:crypto";
import type { NextRequest } from "next/server";
import { RenderError } from "./types";

export function authenticateRender(request: NextRequest): void {
  const key = process.env.MAHFEL_RENDER_API_KEY;
  if (!key) {
    throw new RenderError("renderer_not_configured", 503);
  }
  const supplied = request.headers.get("authorization") ?? "";
  const actual = createHash("sha256").update(supplied).digest();
  const expected = createHash("sha256").update(`Bearer ${key}`).digest();
  if (!timingSafeEqual(actual, expected)) {
    throw new RenderError("unauthorized", 401);
  }
}
