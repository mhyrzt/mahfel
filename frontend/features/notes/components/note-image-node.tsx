import { createContext, useContext, useEffect, useState, type ReactNode } from "react";
import Image from "next/image";
import {
  $applyNodeReplacement,
  DecoratorNode,
  type NodeKey,
  type SerializedLexicalNode,
} from "lexical";

import { fetchCollaborationImage } from "@/lib/collaboration/images";
import type { Locale } from "@/lib/i18n/config";
import { messagesFor } from "@/lib/i18n/messages";

export const NotesImageLocaleContext = createContext<Locale>("en");

type SerializedNoteImage = SerializedLexicalNode & {
  fileId: string;
  altText: string;
};

function NoteImage({ fileId, altText }: Readonly<{ fileId: string; altText: string }>) {
  const text = messagesFor(useContext(NotesImageLocaleContext)).collaboration;
  const [url, setUrl] = useState<string | null>(null);
  const [failed, setFailed] = useState(false);
  useEffect(() => {
    let active = true;
    let objectUrl: string | null = null;
    void fetchCollaborationImage(fileId)
      .then((blob) => {
        objectUrl = URL.createObjectURL(blob);
        if (active) {
          setUrl(objectUrl);
        } else {
          URL.revokeObjectURL(objectUrl);
        }
      })
      .catch(() => {
        if (active) {
          setFailed(true);
        }
      });
    return () => {
      active = false;
      if (objectUrl) {
        URL.revokeObjectURL(objectUrl);
      }
    };
  }, [fileId]);
  if (failed) {
    return <span role="alert">{text.imageFailed}</span>;
  }
  if (!url) {
    return <span role="status">{text.loading}</span>;
  }
  return (
    <Image
      alt={altText}
      className="my-3 h-auto max-w-full rounded-md"
      data-testid="notes-embedded-image"
      height={800}
      src={url}
      unoptimized
      width={1200}
    />
  );
}

export class NoteImageNode extends DecoratorNode<ReactNode> {
  __fileId: string;
  __altText: string;

  static getType(): string {
    return "collaboration-image";
  }
  static clone(node: NoteImageNode): NoteImageNode {
    return new NoteImageNode(node.__fileId, node.__altText, node.__key);
  }
  static importJSON(value: SerializedNoteImage): NoteImageNode {
    return new NoteImageNode(value.fileId, value.altText);
  }
  constructor(fileId: string, altText: string, key?: NodeKey) {
    super(key);
    this.__fileId = fileId;
    this.__altText = altText;
  }
  exportJSON(): SerializedNoteImage {
    return {
      ...super.exportJSON(),
      fileId: this.__fileId,
      altText: this.__altText,
      type: "collaboration-image",
      version: 1,
    };
  }
  createDOM(): HTMLElement {
    return document.createElement("span");
  }
  updateDOM(): false {
    return false;
  }
  decorate(): ReactNode {
    return <NoteImage fileId={this.__fileId} altText={this.__altText} />;
  }
}

export function $createNoteImageNode(fileId: string, altText: string): NoteImageNode {
  return $applyNodeReplacement(new NoteImageNode(fileId, altText));
}
