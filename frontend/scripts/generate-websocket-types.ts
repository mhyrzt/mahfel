import { mkdir, readFile, writeFile } from "node:fs/promises";
import { compile, type JSONSchema } from "json-schema-to-typescript";

type Message = { payload: JSONSchema };
type Operation = { messages: Array<{ $ref: string }> };
type AsyncApiDocument = {
  asyncapi: string;
  components: {
    messages: Record<string, Message>;
    schemas?: Record<string, JSONSchema>;
  };
  operations: Record<string, Operation>;
};

const source = new URL("../../specs/asyncapi.json", import.meta.url);
const target = new URL("../lib/websocket/generated/messages.ts", import.meta.url);
const document = JSON.parse(await readFile(source, "utf8")) as AsyncApiDocument;

if (document.asyncapi !== "3.0.0" || !document.components?.messages) {
  throw new Error("Expected an AsyncAPI 3 document with message components");
}

const messages = document.components.messages;
const names = Object.keys(messages).sort();
const models = await Promise.all(
  names.map((name) => {
    const payload = messages[name]?.payload;
    if (!payload) {
      throw new Error(`Missing payload for ${name}`);
    }
    return compile(removeDescriptions(inlineReferences(payload)) as JSONSchema, name, {
      additionalProperties: false,
      bannerComment: "",
      format: false,
    });
  }),
);

const clientNames = operationMessages("receiveClientMessage");
const serverNames = operationMessages("sendServerMessage");
const output = [
  ...models,
  `export type ClientMessage = ${clientNames.join(" | ")};`,
  `export type ServerMessage = ${serverNames.join(" | ")};`,
  "",
].join("\n");

await mkdir(new URL(".", target), { recursive: true });
await writeFile(target, output);

function operationMessages(operationName: string): string[] {
  const references = document.operations?.[operationName]?.messages;
  if (!Array.isArray(references) || references.length === 0) {
    throw new Error(`Missing messages for ${operationName}`);
  }
  return references.map(({ $ref }) => {
    const name = $ref?.match(/^#\/components\/messages\/([^/]+)$/u)?.[1];
    if (!name || !messages[name]) {
      throw new Error(`Unknown message reference: ${$ref}`);
    }
    return name;
  });
}

function removeDescriptions(value: unknown): unknown {
  if (Array.isArray(value)) {
    return value.map(removeDescriptions);
  }
  if (value !== null && typeof value === "object") {
    return Object.fromEntries(
      Object.entries(value).flatMap(([key, child]) =>
        key === "description" ? [] : [[key, removeDescriptions(child)]],
      ),
    );
  }
  return value;
}

function inlineReferences(value: unknown, seen = new Set<string>()): unknown {
  if (Array.isArray(value)) {
    return value.map((item) => inlineReferences(item, seen));
  }
  if (value !== null && typeof value === "object") {
    const object = value as Record<string, unknown>;
    const reference = object.$ref;
    if (typeof reference === "string") {
      const name = reference.match(/^#\/components\/schemas\/([^/]+)$/u)?.[1];
      const schema = name && document.components.schemas?.[name];
      if (!name || !schema || seen.has(name)) {
        throw new Error(`Unknown or recursive schema reference: ${reference}`);
      }
      const next = new Set(seen);
      next.add(name);
      return inlineReferences(schema, next);
    }
    return Object.fromEntries(
      Object.entries(object).map(([key, child]) => [key, inlineReferences(child, seen)]),
    );
  }
  return value;
}
