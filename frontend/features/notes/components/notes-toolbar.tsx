import { INSERT_ORDERED_LIST_COMMAND, INSERT_UNORDERED_LIST_COMMAND } from "@lexical/list";
import { useLexicalComposerContext } from "@lexical/react/LexicalComposerContext";
import { $setBlocksType } from "@lexical/selection";
import { $createHeadingNode } from "@lexical/rich-text";
import { $getSelection, $insertNodes, FORMAT_TEXT_COMMAND, $isRangeSelection } from "lexical";
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
      className="flex flex-wrap gap-1 border-b p-2"
      role="toolbar"
      aria-label={text.noteFormatting}
    >
      <Button type="button" size="sm" variant="ghost" disabled={disabled} onClick={heading}>
        {text.heading}
      </Button>
      <Button
        type="button"
        size="sm"
        variant="ghost"
        disabled={disabled}
        onClick={() => editor.dispatchCommand(FORMAT_TEXT_COMMAND, "bold")}
      >
        {text.bold}
      </Button>
      <Button
        type="button"
        size="sm"
        variant="ghost"
        disabled={disabled}
        onClick={() => editor.dispatchCommand(FORMAT_TEXT_COMMAND, "italic")}
      >
        {text.italic}
      </Button>
      <Button
        type="button"
        size="sm"
        variant="ghost"
        disabled={disabled}
        onClick={() => editor.dispatchCommand(INSERT_UNORDERED_LIST_COMMAND, undefined)}
      >
        {text.bullets}
      </Button>
      <Button
        type="button"
        size="sm"
        variant="ghost"
        disabled={disabled}
        onClick={() => editor.dispatchCommand(INSERT_ORDERED_LIST_COMMAND, undefined)}
      >
        {text.numbers}
      </Button>
      <Button
        type="button"
        size="sm"
        variant="ghost"
        data-testid="notes-insert-image"
        disabled={disabled || uploadingImage}
        onClick={() => imageInput.current?.click()}
      >
        {uploadingImage ? text.loading : text.insertImage}
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
