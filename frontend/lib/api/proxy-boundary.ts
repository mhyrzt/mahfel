import { isAllowedBackendPath } from "./allowlist";

export const MAX_BODY_BYTES = 2 * 1024 * 1024;

const HOP_HEADERS = new Set([
  "connection",
  "keep-alive",
  "proxy-authenticate",
  "proxy-authorization",
  "proxy-connection",
  "te",
  "trailer",
  "transfer-encoding",
  "upgrade",
]);

export class PayloadTooLarge extends Error {}

export function filteredProxyHeaders(source: Headers, dropped: ReadonlySet<string>): Headers {
  const nominated = new Set(
    (source.get("connection") ?? "").split(",").map((name) => name.trim().toLowerCase()),
  );
  const headers = new Headers();
  source.forEach((value, name) => {
    const normalized = name.toLowerCase();
    if (
      !dropped.has(normalized) &&
      !HOP_HEADERS.has(normalized) &&
      !nominated.has(normalized) &&
      normalized !== "forwarded" &&
      normalized !== "x-real-ip" &&
      !normalized.startsWith("x-forwarded-") &&
      !normalized.startsWith("sec-fetch-")
    ) {
      headers.set(name, value);
    }
  });
  return headers;
}

export function containedUpstreamUrl(
  request: Request,
  path: string[],
  baseUrl: string,
): URL | null {
  if (
    path.some(
      (segment) =>
        !segment || segment === "." || segment === ".." || /[%\\/?#\u0000-\u0020]/.test(segment),
    )
  ) {
    return null;
  }
  const joined = path.join("/");
  if (!isAllowedBackendPath(joined)) {
    return null;
  }
  const base = new URL(baseUrl);
  const encoded = path.map((segment) => encodeURIComponent(segment)).join("/");
  const url = new URL(`${baseUrl}/${encoded}${new URL(request.url).search}`);
  if (
    url.origin !== base.origin ||
    !url.pathname.startsWith("/v1/") ||
    !isAllowedBackendPath(url.pathname.slice(1))
  ) {
    return null;
  }
  return url;
}

export async function boundedRequestBody(
  request: Request,
  signal: AbortSignal,
): Promise<Uint8Array<ArrayBuffer> | undefined> {
  if (!request.body) {
    return undefined;
  }
  const chunks: Uint8Array[] = [];
  let total = 0;
  await request.body.pipeTo(
    new WritableStream<Uint8Array>({
      write(chunk) {
        total += chunk.byteLength;
        if (total > MAX_BODY_BYTES) {
          throw new PayloadTooLarge();
        }
        chunks.push(chunk);
      },
    }),
    { signal },
  );
  const body = new Uint8Array(total);
  let offset = 0;
  for (const chunk of chunks) {
    body.set(chunk, offset);
    offset += chunk.byteLength;
  }
  return body;
}
