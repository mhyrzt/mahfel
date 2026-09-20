import { NextResponse } from "next/server";
import { getServerAccessToken } from "@/lib/auth/server";
import { serverApiBaseUrl } from "@/lib/api/config";

const ALLOWLIST = /^v1\/(auth|rooms|echo|status|health)(\/.*)?$/;

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
const MAX_BODY_BYTES = 2 * 1024 * 1024;

function upstreamUrl(request: Request, path: string[]): URL {
  const source = new URL(request.url);
  const encodedPath = path.map((segment) => encodeURIComponent(segment)).join("/");
  return new URL(`${serverApiBaseUrl}/${encodedPath}${source.search}`);
}

function upstreamHeaders(request: Request, accessToken: string): Headers {
  const headers = new Headers();
  request.headers.forEach((value, name) => {
    const normalized = name.toLowerCase();
    if (!REQUEST_HEADERS_TO_DROP.has(normalized) && !normalized.startsWith("sec-fetch-")) {
      headers.set(name, value);
    }
  });
  headers.set("authorization", `Bearer ${accessToken}`);
  return headers;
}

function downstreamHeaders(upstream: Response): Headers {
  const headers = new Headers();
  upstream.headers.forEach((value, name) => {
    if (!RESPONSE_HEADERS_TO_DROP.has(name.toLowerCase())) {
      headers.set(name, value);
    }
  });
  return headers;
}

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
  if (!ALLOWLIST.test(joinedPath)) {
    return NextResponse.json({ error: "Not found" }, { status: 404 });
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
  const init: RequestInit & { duplex?: "half" } = {
    method: request.method,
    headers: upstreamHeaders(request, accessToken),
    body: hasBody ? request.body : undefined,
    cache: "no-store",
    redirect: "manual",
    signal,
  };
  if (hasBody) {
    init.duplex = "half";
  }
  let response: Response;
  try {
    response = await fetch(upstreamUrl(request, path), init);
  } catch (error) {
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
    headers: downstreamHeaders(response),
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
