import { NextResponse } from "next/server";
import { RenderError } from "@/features/recordings/server-render/types";

export function imageResponse(bytes: Buffer, mime: string): NextResponse {
  return new NextResponse(new Uint8Array(bytes), {
    headers: {
      "Content-Type": mime,
      "Cache-Control": "no-store",
      "X-Renderer-Version": "native-v1",
      "X-Content-Type-Options": "nosniff",
    },
  });
}

export function renderErrorResponse(error: unknown): NextResponse {
  const known = error instanceof RenderError;
  const code = known ? error.code : "render_failed";
  const status = known ? error.status : 503;
  return NextResponse.json(
    {
      code,
      message:
        status === 401
          ? "A rendering API key is required."
          : "The rendering request could not be completed.",
    },
    { status, headers: code === "renderer_busy" ? { "Retry-After": "1" } : undefined },
  );
}
