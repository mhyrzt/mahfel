import { join } from "node:path";

const directory = process.argv[2];
if (!directory) {
  throw new Error("Fixture directory required");
}
const origin = new URL(
  process.env.MAHFEL_RECORDING_FIXTURE_ASSET_ORIGIN ?? "http://127.0.0.1:3011",
);
if (origin.hostname !== "127.0.0.1") {
  throw new Error("Fixture assets must bind to loopback");
}
const server = Bun.serve({
  hostname: "127.0.0.1",
  port: Number(origin.port),
  fetch(request) {
    const path = new URL(request.url).pathname;
    if (request.method !== "GET" || !["/image.png", "/deck.pdf"].includes(path)) {
      return new Response(null, { status: 404 });
    }
    return new Response(Bun.file(join(directory, path.slice(1))));
  },
});
process.stdout.write(`${server.url}\n`);
