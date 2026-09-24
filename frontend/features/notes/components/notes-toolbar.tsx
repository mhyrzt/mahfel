import { INSERT_ORDERED_LIST_COMMAND, INSERT_UNORDERED_LIST_COMMAND } from "@lexical/list";
import { useLexicalComposerContext } from "@lexical/react/LexicalComposerContext";
import { $setBlocksType } from "@lexical/selection";
import { $createHeadingNode } from "@lexical/rich-text";
import { $getSelection, FORMAT_TEXT_COMMAND, $isRangeSelection } from "lexical";

import { Button } from "@/components/ui/button";
import type { Locale } from "@/lib/i18n/config";
import { messagesFor } from "@/lib/i18n/messages";

export function NotesToolbar({
  disabled,
  locale,
}: Readonly<{ disabled: boolean; locale: Locale }>) {
  const [editor] = useLexicalComposerContext();
  const text = messagesFor(locale).collaboration;
  const heading = () =>
    editor.update(() => {
      const selection = $getSelection();
      if ($isRangeSelection(selection)) {
        $setBlocksType(selection, () => $createHeadingNode("h2"));
      }
    });
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
    </div>
  );
}
