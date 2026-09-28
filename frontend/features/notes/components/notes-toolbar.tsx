import { INSERT_ORDERED_LIST_COMMAND, INSERT_UNORDERED_LIST_COMMAND } from "@lexical/list";
import { useLexicalComposerContext } from "@lexical/react/LexicalComposerContext";
import { $setBlocksType } from "@lexical/selection";
import { $createHeadingNode } from "@lexical/rich-text";
import {
  $createParagraphNode,
  $getSelection,
  $insertNodes,
  FORMAT_TEXT_COMMAND,
  $isRangeSelection,
  REDO_COMMAND,
  UNDO_COMMAND,
} from "lexical";
import {
  Bold,
  Heading2,
  ImagePlus,
  Italic,
  List,
  ListOrdered,
  Redo2,
  Underline,
  Undo2,
  Pilcrow,
} from "lucide-react";
import { useRef, useState } from "react";

import { Button } from "@/components/ui/button";
import type { Locale } from "@/lib/i18n/config";
import { messagesFor } from "@/lib/i18n/messages";
import { registerCollaborationImage } from "@/lib/collaboration/images";

import { $createNoteImageNode } from "./note-image-node";

export function NotesToolbar({
  disabled,
  locale,
  sessionId,
}: Readonly<{ disabled: boolean; locale: Locale; sessionId: string }>) {
  const [editor] = useLexicalComposerContext();
  const imageInput = useRef<HTMLInputElement>(null);
  const [uploadingImage, setUploadingImage] = useState(false);
  const [imageError, setImageError] = useState(false);
  const text = messagesFor(locale).collaboration;
  const heading = () =>
    editor.update(() => {
      const selection = $getSelection();
      if ($isRangeSelection(selection)) {
        $setBlocksType(selection, () => $createHeadingNode("h2"));
      }
    });
  const paragraph = () =>
    editor.update(() => {
      const selection = $getSelection();
      if ($isRangeSelection(selection)) {
        $setBlocksType(selection, () => $createParagraphNode());
      }
    });
  const insertImage = async (file: File) => {
    setUploadingImage(true);
    setImageError(false);
    try {
      const fileId = await registerCollaborationImage(file, sessionId, "notes");
      editor.update(() => $insertNodes([$createNoteImageNode(fileId, file.name)]));
    } catch {
      setImageError(true);
    } finally {
      setUploadingImage(false);
      if (imageInput.current) {
        imageInput.current.value = "";
      }
    }
  };
  return (
    <div
      className="flex flex-wrap items-center gap-1 border-b bg-muted/30 p-2"
      role="toolbar"
      aria-label={text.noteFormatting}
    >
      <Button
        type="button"
        size="icon"
        variant="ghost"
        aria-label={text.undo}
        title={text.undo}
        disabled={disabled}
        onClick={() => editor.dispatchCommand(UNDO_COMMAND, undefined)}
      >
        <Undo2 className="size-4" />
      </Button>
      <Button
        type="button"
        size="icon"
        variant="ghost"
        aria-label={text.redo}
        title={text.redo}
        disabled={disabled}
        onClick={() => editor.dispatchCommand(REDO_COMMAND, undefined)}
      >
        <Redo2 className="size-4" />
      </Button>
      <span className="mx-1 h-6 w-px bg-border" aria-hidden="true" />
      <Button
        type="button"
        size="icon"
        variant="ghost"
        aria-label={text.paragraph}
        title={text.paragraph}
        disabled={disabled}
        onClick={paragraph}
      >
        <Pilcrow className="size-4" />
      </Button>
      <Button
        type="button"
        size="icon"
        variant="ghost"
        aria-label={text.heading}
        title={text.heading}
        disabled={disabled}
        onClick={heading}
      >
        <Heading2 className="size-4" />
      </Button>
      <span className="mx-1 h-6 w-px bg-border" aria-hidden="true" />
      <Button
        type="button"
        size="icon"
        variant="ghost"
        aria-label={text.bold}
        title={text.bold}
        disabled={disabled}
        onClick={() => editor.dispatchCommand(FORMAT_TEXT_COMMAND, "bold")}
      >
        <Bold className="size-4" />
      </Button>
      <Button
        type="button"
        size="icon"
        variant="ghost"
        aria-label={text.italic}
        title={text.italic}
        disabled={disabled}
        onClick={() => editor.dispatchCommand(FORMAT_TEXT_COMMAND, "italic")}
      >
        <Italic className="size-4" />
      </Button>
      <Button
        type="button"
        size="icon"
        variant="ghost"
        aria-label={text.underline}
        title={text.underline}
        disabled={disabled}
        onClick={() => editor.dispatchCommand(FORMAT_TEXT_COMMAND, "underline")}
      >
        <Underline className="size-4" />
      </Button>
      <span className="mx-1 h-6 w-px bg-border" aria-hidden="true" />
      <Button
        type="button"
        size="icon"
        variant="ghost"
        aria-label={text.bullets}
        title={text.bullets}
        disabled={disabled}
        onClick={() => editor.dispatchCommand(INSERT_UNORDERED_LIST_COMMAND, undefined)}
      >
        <List className="size-4" />
      </Button>
      <Button
        type="button"
        size="icon"
        variant="ghost"
        aria-label={text.numbers}
        title={text.numbers}
        disabled={disabled}
        onClick={() => editor.dispatchCommand(INSERT_ORDERED_LIST_COMMAND, undefined)}
      >
        <ListOrdered className="size-4" />
      </Button>
      <Button
        type="button"
        size="icon"
        variant="ghost"
        aria-label={text.insertImage}
        title={text.insertImage}
        data-testid="notes-insert-image"
        disabled={disabled || uploadingImage}
        onClick={() => imageInput.current?.click()}
      >
        {uploadingImage ? text.loading : <ImagePlus className="size-4" />}
      </Button>
      <input
        ref={imageInput}
        className="sr-only"
        type="file"
        accept="image/png,image/jpeg"
        aria-label={text.insertImage}
        disabled={disabled || uploadingImage}
        onChange={(event) => {
          const file = event.target.files?.[0];
          if (file) {
            void insertImage(file);
          }
        }}
      />
      {imageError ? (
        <span className="text-xs text-destructive" role="alert">
          {text.imageFailed}
        </span>
      ) : null}
    </div>
  );
}
