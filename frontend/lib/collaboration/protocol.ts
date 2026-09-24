import * as decoding from "lib0/decoding";
import * as encoding from "lib0/encoding";
import * as awarenessProtocol from "y-protocols/awareness";
import * as syncProtocol from "y-protocols/sync";
import type * as Y from "yjs";

import type { MahfelCollaborationProvider } from "./provider";

const SYNC = 0;
const AWARENESS = 1;

export function syncStepOne(doc: Y.Doc): Uint8Array {
  const encoder = encoding.createEncoder();
  encoding.writeVarUint(encoder, SYNC);
  syncProtocol.writeSyncStep1(encoder, doc);
  return encoding.toUint8Array(encoder);
}

export function updateFrame(update: Uint8Array): Uint8Array {
  const encoder = encoding.createEncoder();
  encoding.writeVarUint(encoder, SYNC);
  syncProtocol.writeUpdate(encoder, update);
  return encoding.toUint8Array(encoder);
}

export function awarenessFrame(
  awareness: awarenessProtocol.Awareness,
  clients: number[],
): Uint8Array {
  const encoder = encoding.createEncoder();
  encoding.writeVarUint(encoder, AWARENESS);
  encoding.writeVarUint8Array(encoder, awarenessProtocol.encodeAwarenessUpdate(awareness, clients));
  return encoding.toUint8Array(encoder);
}

export function receiveFrame(
  frame: Uint8Array,
  doc: Y.Doc,
  awareness: awarenessProtocol.Awareness,
  provider: MahfelCollaborationProvider,
): { reply?: Uint8Array; synced: boolean } {
  const decoder = decoding.createDecoder(frame);
  const kind = decoding.readVarUint(decoder);
  if (kind === SYNC) {
    const encoder = encoding.createEncoder();
    encoding.writeVarUint(encoder, SYNC);
    const messageKind = syncProtocol.readSyncMessage(decoder, encoder, doc, provider);
    const reply = encoding.toUint8Array(encoder);
    return {
      reply: reply.length > 1 ? reply : undefined,
      synced: messageKind === syncProtocol.messageYjsSyncStep2,
    };
  }
  if (kind === AWARENESS) {
    awarenessProtocol.applyAwarenessUpdate(
      awareness,
      decoding.readVarUint8Array(decoder),
      provider,
    );
    return { synced: false };
  }
  throw new Error("Unsupported collaboration frame");
}
