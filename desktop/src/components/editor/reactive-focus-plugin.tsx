import { useLexicalComposerContext } from "@lexical/react/LexicalComposerContext";
import { useEffect } from "react";
import { useEditorFocusStore } from "~/store/editor-focus-state";

export function ReactiveFocusPlugin({ id }: { id: string }) {
	const [editor] = useLexicalComposerContext();

	const focusedId = useEditorFocusStore((s) => s.focusedId);
	const clearFocus = useEditorFocusStore((s) => s.clearFocus);

	useEffect(() => {
		if (focusedId !== id) return;

		// ensure editor is ready
		requestAnimationFrame(() => {
			editor.focus();
			clearFocus(); // optional: one-shot behavior
		});
	}, [focusedId, id, editor, clearFocus]);

	return null;
}
