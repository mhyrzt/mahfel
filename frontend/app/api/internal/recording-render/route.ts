import type { NextRequest, NextResponse } from "next/server";
import { authenticateRender } from "@/features/recordings/server-render/auth";
import { withRenderCapacity } from "@/features/recordings/server-render/capacity";
import { readRenderRequest } from "@/features/recordings/server-render/request";
import { renderImage } from "@/features/recordings/server-render/render";
import { imageResponse, renderErrorResponse } from "./responses";

export const runtime = "nodejs";

export async function POST(request: NextRequest): Promise<NextResponse> {
  try {
    authenticateRender(request);
    return await withRenderCapacity(async () => {
      const signal = AbortSignal.any([request.signal, AbortSignal.timeout(30000)]);
      const input = await readRenderRequest(request, signal);
      const image = await renderImage(input, signal);
      return imageResponse(image.bytes, image.mime);
    });
  } catch (error) {
    return renderErrorResponse(error);
  }
}
