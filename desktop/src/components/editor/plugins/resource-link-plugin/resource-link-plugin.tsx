import { useLexicalComposerContext } from "@lexical/react/LexicalComposerContext";
import { useEffect } from "react";
import { ResourceLinkAutocomplete } from "./resource-link-autocomplete";
import { ResourceLinkNode } from "./resource-link-node";

export function ResourceLinkPlugin() {
	const [editor] = useLexicalComposerContext();

	useEffect(() => {
		if (!editor.hasNodes([ResourceLinkNode])) {
			throw new Error(
				"ResourceLinkPlugin: ResourceLinkNode is not registered on editor",
			);
		}
	}, [editor]);

	return <ResourceLinkAutocomplete />;
}
