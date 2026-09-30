import { createEditor, DecoratorNode, type NodeKey, type SerializedLexicalNode } from "lexical";
import { HeadingNode, QuoteNode } from "@lexical/rich-text";
import { ListItemNode, ListNode } from "@lexical/list";
import { LinkNode } from "@lexical/link";
import { createYjsBinding, syncYjsChangesToLexical, type Provider } from "@lexical/yjs";
import * as Y from "yjs";

let resolveAsset: (id: string) => string = () => "";
type ImageData = SerializedLexicalNode & { fileId: string; altText: string };
class ArchivedImageNode extends DecoratorNode<null> {
  __fileId: string;
  __altText: string;
  constructor(fileId = "", altText = "", key?: NodeKey) {
    super(key);
    this.__fileId = fileId;
    this.__altText = altText;
  }
  static getType(): string {
    return "collaboration-image";
  }
  static clone(node: ArchivedImageNode): ArchivedImageNode {
    return new ArchivedImageNode(node.__fileId, node.__altText, node.__key);
  }
  static importJSON(value: ImageData): ArchivedImageNode {
    return new ArchivedImageNode(value.fileId, value.altText);
  }
  exportJSON(): ImageData {
    return {
      ...super.exportJSON(),
      type: "collaboration-image",
      version: 1,
      fileId: this.__fileId,
      altText: this.__altText,
    };
  }
  createDOM(): HTMLElement {
    const url = resolveAsset(this.__fileId);
    if (!url) {
      const placeholder = document.createElement("span");
      placeholder.textContent = `${this.__altText || "Image"} (unavailable)`;
      return placeholder;
    }
    const image = document.createElement("img");
    image.src = url;
    image.alt = this.__altText;
    image.style.maxWidth = "100%";
    return image;
  }
  updateDOM(previous: ArchivedImageNode): boolean {
    return previous.__fileId !== this.__fileId || previous.__altText !== this.__altText;
  }
  decorate(): null {
    return null;
  }
}

const provider: Provider = {
  awareness: {
    getLocalState: () => null,
    getStates: () => new Map(),
    on() {},
    off() {},
    setLocalState() {},
    setLocalStateField() {},
  },
  connect() {},
  disconnect() {},
  on() {},
  off() {},
};

export class ArchivedNotes {
  private readonly doc = new Y.Doc();
  private readonly editor;
  private readonly binding;
  private readonly observer;
  constructor(element: HTMLElement, assets: (id: string) => string) {
    resolveAsset = assets;
    this.editor = createEditor({
      namespace: "recording-notes",
      editable: false,
      nodes: [HeadingNode, QuoteNode, ListNode, ListItemNode, LinkNode, ArchivedImageNode],
      onError(error) {
        throw error;
      },
    });
    this.editor.setRootElement(element);
    this.binding = createYjsBinding({
      editor: this.editor,
      id: "recording-notes",
      doc: this.doc,
      docMap: new Map([["recording-notes", this.doc]]),
    });
    this.observer = (events: Y.YEvent<Y.Text>[]) =>
      syncYjsChangesToLexical(this.binding, provider, events, false, () => {});
    this.doc.get("root", Y.XmlText).observeDeep(this.observer);
  }
  update(source: Y.Doc): void {
    Y.applyUpdate(this.doc, Y.encodeStateAsUpdate(source, Y.encodeStateVector(this.doc)));
  }
  destroy(): void {
    this.doc.get("root", Y.XmlText).unobserveDeep(this.observer);
    this.editor.setRootElement(null);
    this.doc.destroy();
  }
}
