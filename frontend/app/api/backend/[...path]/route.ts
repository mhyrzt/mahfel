import { NextResponse } from "next/server";
import { getServerAccessToken } from "@/lib/auth/server";
import { serverApiBaseUrl } from "@/lib/api/config";
import {
  boundedRequestBody,
  containedUpstreamUrl,
  filteredProxyHeaders,
  MAX_BODY_BYTES,
  PayloadTooLarge,
} from "@/lib/api/proxy-boundary";

const REQUEST_HEADERS_TO_DROP = new Set([
  "accept-encoding",
  "authorization",
  "connection",
  "content-length",
  "cookie",
  "host",
  "origin",
  "referer",
  "transfer-encoding",
  "x-forwarded-host",
  "x-forwarded-proto",
]);

const RESPONSE_HEADERS_TO_DROP = new Set([
  "access-control-allow-credentials",
  "access-control-allow-headers",
  "access-control-allow-methods",
  "access-control-allow-origin",
  "connection",
  "content-encoding",
  "content-length",
  "set-cookie",
  "transfer-encoding",
]);

const UPSTREAM_TIMEOUT_MS = 10_000;
interface Context {
  params: Promise<{ path: string[] }>;
}

async function handleBackendRequest(request: Request, context: Context): Promise<Response> {
  const accessToken = await getServerAccessToken();
  if (!accessToken) {
    return NextResponse.json({ error: "Unauthorized" }, { status: 401 });
  }
  const { path } = await context.params;
  const joinedPath = path.join("/");
  const upstream = containedUpstreamUrl(request, path, serverApiBaseUrl);
  if (!upstream) {
    return NextResponse.json({ error: "Not found" }, { status: 404 });
  }
  if (joinedPath === "v1/ws/tickets" && request.method !== "POST") {
    return NextResponse.json({ error: "Method not allowed" }, { status: 405 });
  }
  if (/^v1\/sessions\/.+\/roster$/.test(joinedPath) && request.method !== "GET") {
    return NextResponse.json({ error: "Method not allowed" }, { status: 405 });
  }
  const contentLength = request.headers.get("content-length");
  if (contentLength && Number(contentLength) > MAX_BODY_BYTES) {
    return NextResponse.json({ error: "Payload too large" }, { status: 413 });
  }
  if (contentLength) {
    const parsed = Number(contentLength);
    if (!Number.isFinite(parsed) || parsed < 0) {
      return NextResponse.json({ error: "Bad request" }, { status: 400 });
    }
  }
  const hasBody = request.method !== "GET" && request.method !== "HEAD";
  const timeoutSignal = AbortSignal.timeout(UPSTREAM_TIMEOUT_MS);
  const signal = AbortSignal.any([request.signal, timeoutSignal]);
  let response: Response;
  try {
    const body = hasBody ? await boundedRequestBody(request, signal) : undefined;
    const headers = filteredProxyHeaders(request.headers, REQUEST_HEADERS_TO_DROP);
    headers.set("authorization", `Bearer ${accessToken}`);
    response = await fetch(upstream, {
      method: request.method,
      headers,
      body,
      cache: "no-store",
      redirect: "manual",
      signal,
    });
  } catch (error) {
    if (error instanceof PayloadTooLarge) {
      return NextResponse.json({ error: "Payload too large" }, { status: 413 });
    }
    if (timeoutSignal.aborted) {
      return NextResponse.json({ error: { code: "UPSTREAM_TIMEOUT" } }, { status: 504 });
    }
    if (request.signal.aborted) {
      return new Response(null, { status: 499 });
    }
    throw error;
  }
  const downstream = new Response(response.status === 204 ? null : response.body, {
    status: response.status,
    statusText: response.statusText,
    headers: filteredProxyHeaders(response.headers, RESPONSE_HEADERS_TO_DROP),
  });
  return downstream;
}

export async function GET(request: Request, context: Context): Promise<Response> {
  return handleBackendRequest(request, context);
}

export async function POST(request: Request, context: Context): Promise<Response> {
  return handleBackendRequest(request, context);
}

export async function PUT(request: Request, context: Context): Promise<Response> {
  return handleBackendRequest(request, context);
}

export async function PATCH(request: Request, context: Context): Promise<Response> {
  return handleBackendRequest(request, context);
}

export async function DELETE(request: Request, context: Context): Promise<Response> {
  return handleBackendRequest(request, context);
}
