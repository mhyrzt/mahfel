import { name, version } from "@/package.json";

export const runtime = "nodejs";
export const dynamic = "force-dynamic";

export function GET() {
  return Response.json(
    {
      status: "ok",
      service: name,
      version,
      kind: "liveness",
      timestamp: new Date().toISOString(),
      uptimeSeconds: Math.floor(process.uptime()),
      environment: process.env.NODE_ENV ?? "unknown",
    },
    { headers: { "Cache-Control": "no-store" } },
  );
}
